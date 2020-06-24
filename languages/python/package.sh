#!/usr/bin/env bash

# Skip Python 2.7 and Python 3.5
export CIBW_SKIP="cp27-* cp35-* pp27-*"
 # 64-bit builds only
export CIBW_BUILD="*64"
# Used in build.py to find right files
export CIBW_ENVIRONMENT="ENV=CI"

make clean

cargo build --target x86_64-unknown-linux-musl --release
mkdir -p native
cp ../../polar/polar.h native/
cp ../../target/x86_64-unknown-linux-musl/release/libpolar.a native/
python -m cibuildwheel --output-dir wheelhouse --platform linux
