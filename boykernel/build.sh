#!/bin/bash

set -e
RUSTFLAGS="-C relocation-model=static -C link-args=-no-pie"
cargo build -Zbuild-std=core,alloc --target x86_64-custom.json --release
