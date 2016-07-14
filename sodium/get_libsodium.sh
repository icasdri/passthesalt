#!/bin/bash

# Fail on any non-zero exit status
set -e

# This script must be executed in the directory it lives in.
# Here's quick check to a sister file
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
make DESTDIR=../dest install

cd ..
