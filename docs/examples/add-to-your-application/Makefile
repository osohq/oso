test: test-python test-policy-extensions test-ruby

test-policy-extensions:
	pytest test_policy_extensions.py

test-python:
	pytest python/app/test.py

test-ruby:
	cd ruby && bundle install
	cd ruby && bundle exec ruby test.rb
