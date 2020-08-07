#!/usr/bin/env sh
cd oso
apk add build-base libffi-dev rustup
rustup-init -y
$HOME/.cargo/bin/rustup target add x86_64-unknown-linux-musl
RUSTFLAGS='-C target-feature=-crt-static -C relocation-model=pic' $HOME/.cargo/bin/cargo build --target x86_64-unknown-linux-musl
mkdir -p languages/python/native
cp polar-c-api/polar.h languages/python/native/
cp target/x86_64-unknown-linux-musl/debug/libpolar.a languages/python/native/
cd languages/python
python -m pip install -r requirements.txt
python -m pip install -r requirements-tests.txt
python -m pip install wheel
python setup.py sdist bdist_wheel