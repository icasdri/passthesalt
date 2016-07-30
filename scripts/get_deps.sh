#!/bin/bash

# Fail on any non-zero exit status
set -e

# Args:
#   1: directory of the root of the project
#   2: tag of the build if this is a tag build
#   3: os name, either 'linux' or 'osx'

[ "$#" -ge 3 ]
cd "$1/scripts"

if [ "$3" == "osx" ]; then
    brew update
    brew install gnupg
fi

./libsodium.sh "$1"
