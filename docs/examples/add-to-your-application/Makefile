# By unexporting BUNDLE_GEMFILE here, we prevent bundler from using the
# Gemfile defined in `docs/examples/Makefile`.
unexport BUNDLE_GEMFILE

test: test-python test-policy-extensions test-ruby test-node test-go

test-policy-extensions: venv
	. venv/bin/activate; pytest test_policy_extensions.py

test-python:
	$(MAKE) -C python test

test-node:
	$(MAKE) -C node test

test-ruby:
	$(MAKE) -C ruby test

test-go:
	$(MAKE) -C go test

venv: venv/touchfile

venv/touchfile: requirements.txt
	test -d venv || python3 -m venv venv
	. venv/bin/activate; pip install -r requirements.txt
	touch $@

.PHONY: test test-policy-extensions test-python test-node test-ruby test-go
