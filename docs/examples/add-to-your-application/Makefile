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
	# TODO disabled because these won't pass CI for some reason
	# https://github.com/osohq/oso/pull/1157/checks?check_run_id=3592723372
	cd ruby; pwd; cat Gemfile; cat Gemfile.lock; bundle install --gemfile=./Gemfile
	cd ruby; bundle exec ruby test.rb
