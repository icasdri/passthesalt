#!/bin/bash

# Fail on any non-zero exit status
set -e

# We take the root of the project (the main working dir -- where we live) as
# the first argument.
[ "$#" -ge 1 ]
cd "$1"

if [ -z "$TRAVIS_TAG" ]; then
    cargo build --verbose
    cargo test --verbose
else
    cargo build --release --verbose
    cargo test --release --verbose
fi
