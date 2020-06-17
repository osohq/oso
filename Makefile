.PHONY: test rust-test rust-build python-build python-test

PYTHON_POLAR_WHEEL := compat_testing/oso-0.0.4-py3-none-any.whl

test: rust-test python-test ruby-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python build

python-test: python-build
	$(MAKE) -C languages/python test
	python examples/expenses-py/app.py

ruby-test:
	$(MAKE) -C languages/ruby/polar test

docs-test: python-build
	$(MAKE) -C languages/python/docs test

# Ensure that parity tests are still compatible with old code.
test_compat:
	pip install --force-reinstall $(PYTHON_POLAR_WHEEL)[dev]
	EXPECT_XFAIL_PASS=1 OSO_COMPAT=1 pytest -rf languages/python/tests/parity
	OSO_COMPAT=1 python examples/expenses-py/app.py
