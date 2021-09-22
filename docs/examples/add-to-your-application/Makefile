# By unexporting BUNDLE_GEMFILE here, we prevent bundler from using the
# Gemfile defined in `docs/examples/Makefile`.
unexport BUNDLE_GEMFILE

test: test-python test-policy-extensions test-ruby test-node

test-policy-extensions:
	pytest test_policy_extensions.py

test-python:
	pip install flask sqlalchemy
	pytest python/app/test.py

test-node:
	cd node; npm install
	cd node; node test.js

test-ruby:
	cd ruby; bundle install
	cd ruby; bundle exec ruby test.rb
