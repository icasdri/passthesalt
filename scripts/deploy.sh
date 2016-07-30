#!/bin/bash

# Fail on any non-zero exit status
set -e

# Args:
#   1: directory of the root of the project
#   2: tag of the build if this is a tag build
#   3: os name, either 'linux' or 'osx'
#   4: GitHub API key with repo access for deployment

[ "$#" -ge 4 ]
cd "$1"

release_binary="$1/target/release/passthesalt"
>&2 echo 'Checking for release binary output...'
[ -f "$release_binary" ]
>&2 echo 'Copying release binary output...'
cp -- "$release_binary" .

name_with_version=$(./passthesalt --version | sed -e 's/ /-/')
case "$3" in
    osx)
        to_upload="${name_with_version}-macos.zip"
        ;;
    linux)
        to_upload="${name_with_version}-linux.zip"
        ;;
esac
>&2 echo "Zipping release as ${to_upload}..."
zip -r "$to_upload" passthesalt > /dev/null


# GitHub Release uploading by dpl
if [ -z "$2" ]; then
    >&2 echo "Not a tag build, skipping deployment."
else
    >&2 echo "Retrieving and invoking github-release."
    
    export GOPATH="$1"
    export PATH="$PATH:$GOPATH"

    go get github.com/aktau/github-release

    github-release upload --security-token "$4" --user icasdri --repo passthesalt --tag "$2" --file "$to_upload" --name "$to_upload"
fi
