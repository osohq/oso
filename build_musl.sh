#!/usr/bin/env bash

apk add musl-dev rustup
rustup-init -y
source $HOME/.cargo/env
rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C target-feature=-crt-static -C relocation-model=pic' cargo build cargo build --target x86_64-unknown-linux-musl