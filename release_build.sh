#!/bin/bash

set -x

cargo build --release
rm -rf languages/python/oso/build
rm -rf languages/python/oso/native
mkdir -p languages/python/oso/native
cp -r target/release/libpolar.a languages/python/oso/native/libpolar.a
cp -r polar-c-api/polar.h languages/python/oso/native/polar.h
cd languages/python/oso
pip uninstall oso -y
OSO_ENV=CI pip install -e .