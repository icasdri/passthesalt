#!/bin/bash

# Fail on any non-zero exit status
set -e

# We take the root of the project (the main working dir -- where we live) as
# the first argument.
[ "$#" -ge 1 ]
cd "$1"

export SODIUM_LIB_DIR="$1/usr/local/lib" 
export SODIUM_STATIC=yes

if [ -z "$TRAVIS_TAG" ]; then
    echo "Doing a DEBUG (default) build..."
    cargo build --verbose
    cargo test --verbose
else
    echo "Doing a RELEASE build..."
    cargo build --release --verbose
    cargo test --release --verbose
fi
