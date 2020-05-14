.PHONY: test rust-test rust-build python-build python-test

test: rust-test python-test

rust-test:
	cargo test --release

rust-build:
	cargo build --release

python-build: rust-build
	$(MAKE) -C languages/python build

python-test: python-build
	$(MAKE) -C languages/python test
