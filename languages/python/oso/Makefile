.PHONY: build test test-requirements fmt lint
build:
	rm -rf build
	rm -f _polar_lib.abi3.so
	python setup.py build
	python setup.py develop

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install pytest
	pip install -r requirements-test.txt
	touch $@

test: test-requirements
	pytest

parity-tests:
	cd tests/parity/policy && ln -sf ../../../../../../test/policies/* .
	cd tests/parity/spec && ln -sf ../../../../../../test/spec/* .

fmt:
	black .

typecheck:
	mypy oso polar tests

lint:
	flake8 .

package:
	python setup.py sdist bdist_wheel

repl:
	python -m oso

