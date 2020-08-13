.PHONY: test rust-test rust-build python-build python-test ruby-test java-test docs-test fmt clippy lint wasm-build wasm-test

test: rust-test python-test ruby-test java-test python-flask-test wasm-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python/oso build

python-flask-build: python-build
	$(MAKE) -C languages/python/flask-oso build

python-test: python-build
	$(MAKE) -C languages/python/oso test
	python examples/expenses-py/app.py
	cd test && python test.py

python-flask-test: python-build python-flask-build
	$(MAKE) -C languages/python/flask-oso test

ruby-test:
	$(MAKE) -C languages/ruby test

java-test:
	$(MAKE) -C languages/java package
	cd test && \
		javac -classpath "../languages/java/oso/target/*:." Test.java && \
		java -classpath "../languages/java/oso/target/*:." -enableassertions Test

docs-test: python-build
	$(MAKE) -C docs test

fmt:
	cargo fmt
	$(MAKE) -C languages/python/oso fmt

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

lint: fmt clippy
	$(MAKE) -C languages/ruby lint typecheck

wasm-build:
	$(MAKE) -C polar-wasm-api build

wasm-test:
	$(MAKE) -C polar-wasm-api test
