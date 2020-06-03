#!/usr/bin/env bash
# Build osx packages.

brew install pyenv
pyenv install 3.6.10
pyenv exec python --version

#cargo build --release