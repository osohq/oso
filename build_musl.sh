#!/usr/bin/env sh
cd oso
apk add build-base rustup
rustup-init -y
$HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C target-feature=-crt-static -C relocation-model=pic' $HOME/.cargo/bin/cargo build --target x86_64-unknown-linux-musl --release
python -m pip install cibuildwheel==1.4.2
mkdir -p languages/python/native
cp polar-c-api/polar.h langauges/python/native/
cp target/x86_64-unknown-linux-musl/release/libpolar.a languages/python/native/
python -m cibuildwheel --output-dir wheelhouse
