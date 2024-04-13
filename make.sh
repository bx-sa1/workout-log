#!/bin/bash

DIR="$(dirname "$0")"

if cargo "$@"; then
  [ -d "$DIR/target/debug" ] && \
    cp -r "$DIR/frontend/build*" "$DIR/target/debug/public/"
fi
