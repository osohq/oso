#!/usr/bin/env sh
apk add build-base libffi-dev
cd oso
pip install oso==$OSO_VERSION -f musl-wheel
cd test
python test.py
echo "tests passed"