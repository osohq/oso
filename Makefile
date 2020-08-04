.PHONY: test rust-test rust-build python-build python-test ruby-test java-test docs-test fmt clippy wasm-build

test: rust-test python-test ruby-test java-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python build

wasm-build:
	$(MAKE) -C polar-wasm-api build

python-test: python-build
	$(MAKE) -C languages/python test
	python examples/expenses-py/app.py
	cd test && python test.py

ruby-test:
	$(MAKE) -C languages/ruby test

java-test:
	$(MAKE) -C languages/java package
	cd test && \
		javac -classpath "../languages/java/oso/target/*:." Test.java && \
		java -classpath "../languages/java/oso/target/*:." Test


docs-test: python-build
	$(MAKE) -C docs test

fmt:
	cargo fmt
	$(MAKE) -C languages/python fmt

clippy:
	cargo clippy --all-features --all-targets -- -D warnings
