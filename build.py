#!/usr/bin/env python3

import sys
import os
import shutil
import subprocess
from time import sleep


def print_e(*pos, **kwargs):
    print(*pos, file=sys.stderr, **kwargs)


class BuildError(Exception):
    def __init__(self, message):
        self.message = message


class Environment:
    sodium_version = '1.0.10'
    sodium_tar_expected_size = 1827211
    sodium_tar_hash = \
        '71b786a96dd03693672b0ca3eb77f4fb08430df307051c0d45df5353d22bc4be'

    os_type = None  # one of 'linux', 'macos', 'win'
    root_dir = None  # defaults os.getcwd() below
    sodium_lib_dir = None
    release_version = None
    release_build = False
    can_release = False
    github_repo_owner = 'icasdri'
    github_repo_name = 'passthesalt'
    github_release_api_token = None

    def init_environment(self):
        if 'TRAVIS' in os.environ:
            travis_os_name = os.getenv('TRAVIS_OS_NAME', 'linux')
            if travis_os_name == 'linux':
                self.os_type = 'linux'
            elif travis_os_name == 'osx':
                self.os_type = 'macos'
            else:
                raise BuildError('Unexpected TRAVIS_OS_NAME value: {}'
                                 .format(travis_os_name))
            self.root_dir = os.getenv('TRAVIS_BUILD_DIR', self.root_dir)

            if 'TRAVIS_TAG' in os.environ:
                tag = os.environ['TRAVIS_TAG']
                self.release_build = len(tag.strip()) > 0
            else:
                self.release_build = False

            self.release_version = os.getenv('TRAVIS_TAG', None)
            self.can_release = os.getenv('CAN_RELEASE', False)
            self.github_release_api_token = os.getenv('SEC_GH_API_KEY')
        elif 'APPVEYOR' in os.environ:
            self.os_type = 'win'
            self.root_dir = \
                os.getenv('APPVEYOR_BUILD_FOLDER', self.root_dir)
            self.release_build = \
                os.getenv('APPVEYOR_REPO_TAG', None) == 'true'
            self.release_version = os.getenv('APPVEYOR_REPO_TAG_NAME')
            self.can_release = os.getenv('CAN_RELEASE', False)
            self.github_release_api_token = os.getenv('SEC_GH_API_KEY')
        else:
            # TODO: prompt for os type
            self.os_type = 'linux'
            self.root_dir = os.getcwd()

    def cd_root(self):
        os.chdir(self.root_dir)

    def path(self, target):
        return os.path.join(self.root_dir, target)


ENV = Environment()


def run(args):
    p = subprocess.Popen(args,
                         stdout=subprocess.PIPE,
                         stderr=subprocess.STDOUT,
                         universal_newlines=True)

    stdout_lines = iter(p.stdout.readline, '')
    for line in stdout_lines:
        print(line, end='')

    while p.poll() is None:
        sleep(1)

    if p.returncode != 0:
        raise BuildError('Invocation of {} exitted with non-zero {} status.'
                         .format(args, p.returncode))


def get_sodium():
    ENV.cd_root()
    print_e('------ Dependency (libsodium) Build ------')

    from urllib.request import urlretrieve
    from hashlib import sha256
    import tarfile

    try:
        os.mkdir('libsodium')
    except FileExistsError:
        pass  # make sure it exists
    os.chdir('libsodium')

    if os.path.isfile('sodium.tar.gz'):
        print_e('Removing old libsodium sources...')
        os.remove('sodium.tar.gz')

    print_e('Retrieving libsodium sources...')
    urlretrieve(
        'https://download.libsodium.org/libsodium'
        '/releases/libsodium-{}.tar.gz'.format(ENV.sodium_version),
        'sodium.tar.gz'
    )

    print_e('Checking libsodium source integrity...')
    with open('sodium.tar.gz', 'rb') as sodium_source_tar:
        data = sodium_source_tar.read(ENV.sodium_tar_expected_size + 1)
        m = sha256()
        m.update(data)
        if m.hexdigest() != ENV.sodium_tar_hash:
            raise BuildError('Integrity verification for libsodium '
                             'sources failed')

    source_dir = 'libsodium-{}'.format(ENV.sodium_version)
    if os.path.isdir(source_dir):
        print_e('Removing old libsodium extracted sources...')
        shutil.rmtree(source_dir)

    print_e('Extracting libsodium sources...')
    tarfile.open('sodium.tar.gz', mode='r:gz').extractall()
    os.chdir(source_dir)

    print_e('Beginning libsodium build...')
    run(['./configure'])
    run(['make'])
    run(['make', 'check'])
    # This DESTDIR will make sodium available at libsodium/usr/local/lib
    run(['make', 'DESTDIR={}'.format(ENV.path('libsodium')), 'install'])

    ENV.sodium_lib_dir = ENV.path('libsodium/usr/local/lib')


def build():
    ENV.cd_root()
    print_e('------ Main Build Routine ------')

    os.environ['SODIUM_LIB_DIR'] = ENV.sodium_lib_dir
    os.environ['SODIUM_STATIC'] = 'yes'

    if ENV.release_build:
        print_e('Doing a RELEASE build...')
        run(['cargo', 'build', '--release'])
        run(['cargo', 'test', '--release'])
    else:
        print_e('Doing a DEBUG (normal) build...')
        run(['cargo', 'build'])
        run(['cargo', 'test'])


def consolidate_artifacts():
    ENV.cd_root()
    print_e('------ Artifact Consolidation Routine ------')

    if ENV.release_build:
        binary = ENV.path('target/release/passthesalt')
    else:
        binary = ENV.path('target/debug/passthesalt')

    if ENV.os_type == 'linux' or ENV.os_type == 'macos':
        pass  # binary path is correct
    elif ENV.os_type == 'windows':
        binary = binary + '.exe'
    else:
        raise BuildError('Unrecognized os type: {}'.format(ENV.os_type))

    if ENV.release_build:
        # Check that the version we have is correct
        print_e('Verifying built binary version and expected release version.')
        p = subprocess.Popen([binary, '--version'],
                             stdout=subprocess.PIPE,
                             universal_newlines=True)
        version_string = p.stdout.read().strip()
        if version_string.startswith('passthesalt '):
            binary_version = version_string[12:].strip()
        else:
            raise BuildError('Unexpected output from release binary. Release '
                             'binary outputted: ' + version_string)

            if binary_version != ENV.release_version:
                raise BuildError('Discrepancy between version from binary ({})'
                                 ' and expected version for release ({})'
                                 .format(binary_version, ENV.release_version))

        target_zip = 'passthesalt-{}-{}.zip'.format(ENV.release_version,
                                                    ENV.os_type)
    else:
        target_zip = 'passthesalt-debug-{}.zip'.format(ENV.os_type)

    if os.path.isfile(binary):
        from zipfile import ZipFile
        print_e('Zipping binary to ' + target_zip)
        with ZipFile(target_zip, 'w') as zipped:
            zipped.write(binary, arcname=os.path.basename(binary))
    else:
        raise BuildError('Failed to find binary: {}'
                         .format_map(binary))

    return target_zip


def deploy_release(target_zip):
    ENV.cd_root()
    print_e('------ Release Deployment Routine ------')

    if not ENV.can_release:
        print_e('This is not a build that can be released. '
                'Skipping release deployment.')
        return

    if not ENV.release_build:
        print_e('This is not a release build. '
                'Skipping release deployment.')
        return

    from urllib.request import urlopen, Request
    from urllib.error import HTTPError, URLError
    import json

    def repo_api(additional):
        return 'https://api.github.com/repos/{}/{}/{}?access_token={}'.format(
                ENV.github_repo_owner, ENV.github_repo_name, additional,
                ENV.github_release_api_token)

    release_obj = None
    try:
        print_e('Checking for existing GitHub Release in latest...')
        latest_response = urlopen(repo_api('releases/latest'))

        latest_obj = json.loads(latest_response.read())
        if latest_obj.get('tag_name', None) == ENV.release_version:
            print_e('Found existing GitHub Release.')
            release_obj = latest_obj
    except HTTPError as e:
        if e.code != 404:
            raise BuildError('Unexpected HTTP response from GitHub API: '
                             '{} {}'.format(e.getcode(), e.msg))

    if release_obj is None:
        print_e('Checking for existing GitHub Release across all releases...')
        all_releases_response = urlopen(repo_api('releases'))
        all_releases = json.loads(all_releases_response.read().decode())
        for release_i in all_releases:
            if release_i.get('tag_name', None) == ENV.release_version:
                print_e('Found existing GitHub Release.')
                release_obj = release_i
                break

    if release_obj is None:
        print_e('Existing GitHub Release not found. Creating new one...')
        new_release_data = json.dumps({
            'tag_name': ENV.release_version,
            'name': 'Pending Release',
            'body': 'Please wait while release builds finish and artifacts'
                    ' are uplaoded. This release will be available soon.',
            'draft': True
        }).encode()
        request = Request(repo_api('releases'), new_release_data,
                          {'Content-Type': 'application/json'})

        response = urlopen(request)
        release_obj = json.loads(response.read().decode())

    upload_url = None
    if 'upload_url' in release_obj:
        print_e('Building asset upload url...')
        upload_url = release_obj['upload_url']
        upload_url = upload_url[:upload_url.rfind('{')]  # chop off template
        upload_url = upload_url + '?name={}&access_token={}'.format(
                        target_zip, ENV.github_release_api_token)
    else:
        raise BuildError('Unexpected JSON response from GitHub API: '
                         'no upload_url in returned object!')

    with open(target_zip, 'rb') as target_zip_raw:
        print_e('Reading data for file to upload...')
        target_zip_data = target_zip_raw.read()

    request = Request(upload_url, target_zip_data,
                      {'Content-Type': 'application/zip'})
    try:
        print_e('Uploading release asset...')
        response = urlopen(request)
    except URLError as e:
        if 'Broken pipe' in e.reason:
            raise BuildError('Failed to upload release asset. Either a '
                             'connection problem was encountered or GitHub '
                             'API rejected it (possibly duplicate filename?).')

    if response.getcode() == 201:  # HTTP code for 'Created'
        print_e('Release asset upload successful.')
    else:
        raise BuildError('Failed to upload release asset. GitHub API '
                         'responded: {} {}'.format(response.get_code,
                                                   response.reason))


def main():
    ENV.init_environment()
    try:
        get_sodium()
        build()
        target_zip = consolidate_artifacts()
        deploy_release(target_zip)
    except BuildError as e:
        print_e('---- ERROR -------------------------')
        print_e(e.message)
        print_e('------------------------------------')
        raise e


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print_e('Exitting')
