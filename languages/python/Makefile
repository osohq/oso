.PHONY: build
build:
	rm -r build
	python setup.py build
	python setup.py develop

test:
	pip install pytest
	pytest
