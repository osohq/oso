.PHONY: test rust-test rust-build python-build python-test

test: rust-test python-test ruby-test java-test

rust-test:
	cargo test

rust-build:
	cargo build

python-build: rust-build
	$(MAKE) -C languages/python build

python-test: python-build
	$(MAKE) -C languages/python test
	python examples/expenses-py/app.py
	cd test && python test.py

ruby-test:
	$(MAKE) -C languages/ruby test
	cd test && ruby test.rb

java-test:
	$(MAKE) -C languages/java test
	$(MAKE) -C languages/java package
	cd test && java -classpath "../languages/java/oso/target/*:." Test.java

docs-test: python-build
	$(MAKE) -C docs test

fmt:
	cargo fmt
	$(MAKE) -C languages/python fmt
