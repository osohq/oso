# Expected location of osohq/oso source code
OSO_ROOT ?= $(abspath ../../)

# Language directories
JAVA_DIR := $(OSO_ROOT)/languages/java
JS_DIR := $(OSO_ROOT)/languages/js
RUBY_DIR := $(OSO_ROOT)/languages/ruby

# Needed for tests
JAVA_PACKAGE_JAR_PATH := $(JAVA_DIR)/oso/target/oso-0.26.4.jar

# Note: if you are using bundler in a sub-makefile (in a docs test for example),
# you need to add `unexport BUNDLE_GEMFILE` to that makefile. Otherwise this
# setting will take precedence over the working directory.
export BUNDLE_GEMFILE := $(RUBY_DIR)/Gemfile

submodules: .make.submodules.installed

.make.submodules.installed:
	$(MAKE) update-submodules
	touch $@

update-submodules:
	git submodule update --checkout --init --recursive --remote

.PHONY: submodules update-submodules java-test java-test-context java-test-rbac  python-test ruby-test nodejs-test test-quickstarts-release test-quickstarts-local test

$(JAVA_PACKAGE_JAR_PATH):
	$(MAKE) -C $(JAVA_DIR) package

java-test-rbac: $(JAVA_PACKAGE_JAR_PATH)
	cd rbac/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestApp.java && java -cp $(JAVA_PACKAGE_JAR_PATH):. TestApp

java-test-context: $(JAVA_PACKAGE_JAR_PATH)
	# The way this test works for context is pretty sketchy.
	# We can't set environment variables from within Java like the Python & Ruby
	# tests, so instead we call the test twice, expecting it to fail for production.
	cd context/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext.java && ENV="development" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext
	cd context/java && (ENV="production" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext && exit 1 || exit 0)
	@echo "Ignore that failure it actually worked!"

java-test: java-test-context

python-test:
	cd inheritance/python && pytest .
	cd context/python && pytest .
	cd user_types/python && pytest .
	$(MAKE) -C data_access/django test
	$(MAKE) -C data_access/sqlalchemy test

ruby-test:
	cd context/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd inheritance/ruby && bundle install --quiet && bundle exec rspec test_spec.rb
	cd user_types/ruby && bundle install --quiet && bundle exec rspec test_spec.rb

nodejs-test:
	$(MAKE) -C $(JS_DIR) build
	cd context/nodejs && yarn && yarn jest
	cd inheritance/nodejs && yarn && yarn jest
	cd user_types/nodejs && yarn && yarn jest

test-quickstarts-release: submodules
	$(MAKE) -C quickstart test-release

test-quickstarts-local: submodules
	$(MAKE) -C quickstart test-local

test:
	$(MAKE) -C add-to-your-application test
	$(MAKE) -C rbac test
	$(MAKE) python-test
	$(MAKE) ruby-test
	$(MAKE) java-test
	$(MAKE) nodejs-test
	# $(MAKE) test-quickstarts-local
