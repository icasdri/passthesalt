#!/bin/bash

# Fail on any non-zero exit status
set -e

# Args:
#   1: directory of the root of the project

[ "$#" -ge 1 ]
cd "$1/scripts"

# Check for existence of key file (so we can do integrity check)
[ -f libsodium.asc ]

VERSION='1.0.10'
SOURCE="libsodium-${VERSION}"
SOURCE_FILE="${SOURCE}.tar.gz"
SIG_FILE="${SOURCE_FILE}.sig"

wget "https://download.libsodium.org/libsodium/releases/${SOURCE_FILE}"
wget "https://download.libsodium.org/libsodium/releases/${SIG_FILE}"

gpg --import libsodium.asc
gpg --verify "$SIG_FILE"

tar -zxvf "$SOURCE_FILE"

cd "$SOURCE"

./configure
make
make check
make DESTDIR="$1" install
