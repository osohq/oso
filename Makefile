.PHONY: test rust-test rust-build python-build python-test

test: rust-test python-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python build

python-test: python-build
	$(MAKE) -C languages/python test
	python examples/expenses-py/app.py

docs-test: python-build
	$(MAKE) -C languages/python/docs test
