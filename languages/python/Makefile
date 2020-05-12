.PHONY: build
build:
	rm -rf build
	python setup.py build
	python setup.py develop

test:
	pip install pytest
	pytest
