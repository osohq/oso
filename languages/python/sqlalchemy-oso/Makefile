.PHONY: build test-requirements dev-requirements test fmt lint typecheck package tox

build:
	rm -rf build
	pip install -e .[flask]

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install -r requirements-test.txt
	touch $@

dev-requirements: .make.dev-requirements-install

.make.dev-requirements-install: requirements-dev.txt
	pip install -r requirements-dev.txt
	touch $@

test:
	$(MAKE) tox

fmt: dev-requirements
	isort .
	black .

lint: dev-requirements
	flake8 .

typecheck: dev-requirements
	mypy tests
	mypy sqlalchemy_oso

package:
	python setup.py sdist bdist_wheel

tox: dev-requirements
	tox
