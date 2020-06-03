#!/usr/bin/env bash
# Build osx packages.

cargo build --release

cd languages/python
ENV=RELEASE python setup.py build
ENV=RELEASE python setup.py sdist bdist_wheel