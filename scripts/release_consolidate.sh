#!/bin/bash

# Fail on any non-zero exit status
set -e

# We take the root of the project (the main working dir -- where we live) as
# the first argument.
[ "$#" -ge 1 ]
cd "$1"

release_binary="$1/target/release/passthesalt"
[ -f "$release_binary" ]
cp -- "$release_binary" .

name_with_version=$(./passthesalt --version | sed -e 's/ /-/')
case "$TRAVIS_OS_NAME" in
    osx)
        to_upload="${name_with_version}-macos.zip"
        ;;
    linux)
        to_upload="${name_with_version}-linux.zip"
        ;;
esac
zip -r "$to_upload" passthesalt > /dev/null
echo "$to_upload"
