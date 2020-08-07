#!/usr/bin/env sh
apk add build-base libffi-dev
cd oso/languages/python
python -m pip install -r requirements.txt
python -m pip install -r requirements-tests.txt
python -m pip install wheel
python setup.py sdist bdist_wheel