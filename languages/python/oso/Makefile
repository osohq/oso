.PHONY: build test test-requirements fmt lint
build:
	rm -rf build
	rm -f _polar_lib.abi3.so
	pip install -r requirements.txt
	python setup.py build
	python setup.py develop

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install pytest
	pip install -r requirements-test.txt
	touch $@

test: test-requirements
	pytest

fmt:
	isort .
	black .

typecheck:
	mypy oso polar tests

lint:
	flake8 .

package:
	python setup.py sdist bdist_wheel

repl:
	python -m oso
