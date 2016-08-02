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
            self.release_build = 'TRAVIS_TAG' in os.environ
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
    from urllib.request import urlretrieve
    from hashlib import sha256
    import tarfile

    ENV.cd_root()
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

    if ENV.os_type == 'linux' or ENV.os_type == 'macos':
        release_binary = ENV.path('target/release/passthesalt')
    elif ENV.os_type == 'windows':
        release_binary = ENV.path('target/release/passthesalt.exe')
    else:
        raise BuildError('Unrecognized os type: {}'.format(ENV.os_type))

    # Check that the version we have is correct
    result = subprocess.run([release_binary, '--version'],
                            universal_newlines=True)
    version_string = result.stdout.strip()
    if version_string.startswith('passthesalt '):
        binary_version = version_string[12:].strip()
    else:
        raise BuildError('Unexpected output from release binary. Release '
                         'binary outputted: ' + version_string)

        if binary_version != ENV.release_version:
            raise BuildError('Discrepancy between version from binary ({}) '
                             'and expected version for release ({})'
                             .format(binary_version, ENV.release_version))

    target_zip = 'passthesalt-{}-{}.zip'.format(ENV.release_version,
                                                ENV.os_type)

    if os.path.isfile(release_binary):
        from zipfile import ZipFile
        with ZipFile(target_zip, 'w') as zipped:
            zipped.write(release_binary)
    else:
        raise BuildError('Failed to find release binary: {}'
                         .format_map(release_binary))

    return target_zip


def deploy_release(target_zip):
    ENV.cd_root()

    if not ENV.can_release:
        print_e('This is not a build to-be-released. '
                'Skipping release deploymenet.')
        return

    from urllib.request import urlopen, Request
    import json

    def repo_api(additional):
        return 'https://api.github.com/repos/{}/{}/{}?access_token={}'.format(
                ENV.github_repo_owner, ENV.github_repo_name, additional,
                ENV.github_release_api_token)

    response = urlopen(repo_api('releases/tags/{}'
                                .format(ENV.release_version)))

    release_obj = None
    if response.get_code() == 200:
        print_e('Found existing GitHub Release.')
        release_obj = json.loads(response.read())
    elif response.get_code() == 404:
        print_e('Existing GitHub Release not found. Creating new one...')
        new_release_data = json.dumps({
            'tag_name': ENV.release_version,
            'name': 'Pending Release',
            'body': 'Please wait while release builds finish and artifacts '
                    'are uplaoded. This release will be available soon.',
            'draft': True
        })
        request = Request(repo_api('releases'), new_release_data,
                          {'Content-Type': 'application/json'})

        response = urlopen(request)
        release_obj = json.loads(response.read())
    else:
        raise BuildError('Unexpected HTTP response from GitHub API: '
                         '{} {}'.format(response.get_code(), response.reason))

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
    print_e('Uploading release asset...')
    response = urlopen(request)
    if response.get_code == 201:  # HTTP code for 'Created'
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
