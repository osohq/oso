# By unexporting BUNDLE_GEMFILE here, we prevent bundler from using the
# Gemfile defined in `docs/examples/Makefile`.
unexport BUNDLE_GEMFILE

test: test-python test-policy-extensions test-ruby test-node

test-policy-extensions: venv
	. venv/bin/activate; pytest test_policy_extensions.py

test-python:
	$(MAKE) -C python test

test-node:
	cd node; npm install
	cd node; node test.js

test-ruby:
	cd ruby; bundle install
	cd ruby; bundle exec ruby test.rb

venv: venv/touchfile

venv/touchfile: requirements.txt
	test -d venv || python3 -m venv venv
	. venv/bin/activate; pip install -r requirements.txt
	touch $@

.PHONY: test test-policy-extensions test-python test-node test-ruby
