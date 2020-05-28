.PHONY: test rust-test rust-build python-build python-test

PYTHON_POLAR_WHEEL := compat_testing/oso-0.0.3-py3-none-any.whl

test: rust-test python-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python build

python-test: python-build
	$(MAKE) -C languages/python test

# Ensure that parity tests are still compatible with old code.
test_compat:
	pip install --force-reinstall $(PYTHON_POLAR_WHEEL)[dev]
	EXPECT_XFAIL_PASS=1 pytest -rf languages/python/tests/parity
