#!/usr/bin/env bash
# Build and package everything. Only runs on osx because it does everything. Yeah ya need docker.
# Prolly wanna break this in two to run on github actions or just not care.

rustup target add x86_64-unknown-linux-musl
cargo build --release
cargo build --target x86_64-unknown-linux-musl --release
cd languages/python

mkdir -p wheels
rm -rf wheels/*



rm -rf dist

#TODO: Why can't I build with python 3.6?

# osx 3.7
rm -rf build
env ENV=RELEASE python3.7 setup.py build
env ENV=RELEASE python3.7 setup.py sdist bdist_wheel

# osx 3.8
rm -rf build
env ENV=RELEASE python3.8 setup.py build
env ENV=RELEASE python3.8 setup.py sdist bdist_wheel

docker pull quay.io/pypa/manylinux1_x86_64
mkdir -p native
cp ../../target/x86_64-unknown-linux-musl/release/libpolar.a native/
cp ../../polar/polar.h native/
docker run -it -v $(pwd):/io quay.io/pypa/manylinux1_x86_64 io/package.sh