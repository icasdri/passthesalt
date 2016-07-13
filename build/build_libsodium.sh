#!/bin/bash

# Fail on any non-zero exit status
set -e

VERSION='1.0.10'
TARGET="libsodium-${VERSION}"
TARGET_FILE="${TARGET}.tar.gz"
SIG_FILE="${TARGET_FILE}.sig"

wget "https://download.libsodium.org/libsodium/releases/${TARGET_FILE}"
wget "https://download.libsodium.org/libsodium/releases/${SIG_FILE}"

gpg --import libsodium.asc
gpg --verify "$SIG_FILE"

tar -zxvf "$TARGET_FILE"

cd "$TARGET"

./configure
make
make check

cd ..
ln -s "$TARGET" 'libsodium-root'
