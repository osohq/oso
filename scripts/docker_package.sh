#!/usr/bin/env bash
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl --release

docker pull quay.io/pypa/manylinux1_x86_64
mkdir -p native
cp ../../target/x86_64-unknown-linux-musl/release/libpolar.a native/
cp ../../polar/polar.h native/
docker run -v $(pwd):/io quay.io/pypa/manylinux1_x86_64 io/package.sh