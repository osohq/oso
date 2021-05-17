.PHONY: build test test-requirements fmt lint requirements

build:
	rm -rf build
	pip install -e .[flask]

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt
	pip install pytest
	pip install -r requirements-test.txt
	touch $@

requirements: .make.requirements-install

.make.requirements-install: requirements.txt
	pip install -r requirements.txt
	touch $@

test: requirements test-requirements
	pytest tests

fmt:
	black .

lint:
	flake8 .

typecheck:
	mypy tests
	mypy sqlalchemy_oso

package:
	python setup.py sdist bdist_wheel
