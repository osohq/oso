# Expected location of osohq/oso source code
OSO_ROOT ?= $(abspath ../../)

# Language directories
JAVA_DIR := $(OSO_ROOT)/languages/java
JS_DIR := $(OSO_ROOT)/languages/js
RUBY_DIR := $(OSO_ROOT)/languages/ruby

# Needed for tests
JAVA_PACKAGE_JAR_PATH := $(JAVA_DIR)/oso/target/oso-0.13.1.jar

export BUNDLE_GEMFILE := $(RUBY_DIR)/Gemfile

submodules: .make.submodules.installed

.make.submodules.installed:
	$(MAKE) update-submodules
	touch $@

update-submodules:
	git submodule update --checkout --init --recursive --remote
	cd roles/sqlalchemy/basic && git checkout basic_roles_complete && git pull

.PHONY: submodules update-submodules

$(JAVA_PACKAGE_JAR_PATH):
	$(MAKE) -C $(JAVA_DIR) package

java-test: $(JAVA_PACKAGE_JAR_PATH)
	cd abac/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestAbac.java && java -cp $(JAVA_PACKAGE_JAR_PATH):. TestAbac
	# The way this test works for context is pretty sketchy.
	# We can't set environment variables from within Java like the Python & Ruby
	# tests, so instead we call the test twice, expecting it to fail for production.
	cd context/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext.java && ENV="development" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext
	cd context/java && (ENV="production" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext && exit 1 || exit 0)
	@echo "Ignore that failure it actually worked!"

python-test:
	cd inheritance/python && pytest .
	cd abac/python && pytest .
	cd rbac/python && pytest .
	cd context/python && pytest .
	cd user_types/python && pytest .
	cd roles/sqlalchemy/basic && pip install -r requirements.txt && pytest .
	$(MAKE) -C data_access/django test
	$(MAKE) -C data_access/sqlalchemy test

ruby-test:
	cd abac/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd rbac/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd context/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd inheritance/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd user_types/ruby && bundle install --quiet && bundle exec rspec test_spec.rb

nodejs-test:
	$(MAKE) -C $(JS_DIR) build
	cd rbac/nodejs && yarn && yarn jest
	cd abac/nodejs && yarn && yarn jest
	cd context/nodejs && yarn && yarn jest
	cd inheritance/nodejs && yarn && yarn jest
	cd user_types/nodejs && yarn && yarn jest

test-quickstarts: submodules
	$(MAKE) -C quickstart test

test:
	$(MAKE) python-test
	$(MAKE) ruby-test
	$(MAKE) java-test
	$(MAKE) nodejs-test
