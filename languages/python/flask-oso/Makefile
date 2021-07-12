.PHONY: build test test-requirements fmt lint
build:
	rm -rf build
	python setup.py develop

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install pytest
	pip install -r requirements-test.txt
	touch $@

test: test-requirements
	pytest tests

fmt:
	isort .
	black .

lint:
	flake8 .

package:
	python setup.py sdist bdist_wheel
