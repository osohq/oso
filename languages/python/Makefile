.PHONY: build test test-requirements fmt
build: clean
	python setup.py build
	python setup.py develop

test-requirements: .make.test-requirements-install

.make.test-requirements-install: requirements-test.txt docs/requirements-docs.txt
	pip install pytest
	pip install -r requirements-test.txt
	pip install -r docs/requirements-docs.txt
	touch $@

test: test-requirements
	pytest tests
	pytest oso
	$(MAKE) -C docs test

fmt:
	black .

package:
	python setup.py sdist bdist_wheel

clean:
	rm -f _polar_lib.abi3.so
	rm -rf build
