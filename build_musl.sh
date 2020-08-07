#!/usr/bin/env sh
cd oso
apk add musl-dev rustup
rustup-init -y
$HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C target-feature=-crt-static -C relocation-model=pic' $HOME/.cargo/bin/cargo build --target x86_64-unknown-linux-musl
