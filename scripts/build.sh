#!/bin/bash

# Fail on any non-zero exit status
set -e

# Args:
#   1: directory of the root of the project
#   2: tag of the build if this is a tag build

[ "$#" -ge 2 ]
cd "$1"

export SODIUM_LIB_DIR="$1/usr/local/lib" 
export SODIUM_STATIC=yes

if [ -z "$2" ]; then
    echo "Doing a DEBUG (default) build..."
    cargo build --verbose
    cargo test --verbose
else
    echo "Doing a RELEASE build..."
    cargo build --release --verbose
    cargo test --release --verbose
fi
