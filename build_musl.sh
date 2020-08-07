#!/usr/bin/env sh

apk add musl-dev rustup
rustup-init -y
PATH = $HOME/.cargo/bin:$PATH
rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C target-feature=-crt-static -C relocation-model=pic' cargo build cargo build --target x86_64-unknown-linux-musl