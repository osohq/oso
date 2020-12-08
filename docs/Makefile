# Minimal makefile for Sphinx documentation
#

# You can set these variables from the command line, and also
# from the environment for the first two.
SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

export BUNDLE_GEMFILE := $(abspath ../languages/ruby/Gemfile)
export RUBY_DIR := $(abspath ../languages/ruby)
export JAVA_DIR := $(abspath ../languages/java/oso)
export JS_DIR := $(abspath ../languages/js)


rwildcard=$(foreach d,$(wildcard $(1:=/*)),$(call rwildcard,$d,$2) $(filter $(subst *,%,$2),$d))

JAVA_PACKAGE_JAR_PATH := $(JAVA_DIR)/target/oso-0.9.0.jar

RUBY_FILES := $(call rwildcard,../languages/ruby/lib,*.rb)
JAVA_FILES := $(call rwildcard ../languages/java/oso/src, *.java)
JS_FILES := $(call rwildcard ../languages/js/src, *.ts)

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile html doctest test-quickstarts

_api_docs/ruby: $(RUBY_FILES)
	mkdir -p _api_docs
	cd "$(RUBY_DIR)" && yard doc
	rm -rf _api_docs/ruby
	cp -R $(RUBY_DIR)/doc _api_docs/ruby

_api_docs/java: $(JAVA_FILES)
	mkdir -p _api_docs
	cd "$(JAVA_DIR)" && mvn -q javadoc:javadoc
	rm -rf _api_docs/java
	cp -R $(JAVA_DIR)/target/site/apidocs _api_docs/java

_api_docs/js/node: $(JS_FILES)
	mkdir -p _api_docs/js
	make -C $(JS_DIR) docs
	rm -rf _api_docs/js/node
	cp -R $(JS_DIR)/docs _api_docs/js/node

_api_docs: _api_docs/ruby _api_docs/java _api_docs/js/node

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
html doctest spelling: Makefile deps _api_docs
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

build: Makefile deps _api_docs
	@DOCS_RELEASE=1 $(SPHINXBUILD) -M html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

clean:
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)
	rm -r _api_docs

livehtml: deps _api_docs
	@sphinx-autobuild -b html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

$(JAVA_PACKAGE_JAR_PATH):
	$(MAKE) -C ../languages/java package

java-test: $(JAVA_PACKAGE_JAR_PATH)
	cd examples/abac/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestAbac.java && java -cp $(JAVA_PACKAGE_JAR_PATH):. TestAbac
	# The way this test works for context is pretty sketchy.
	# We can't set environment variables from within Java like the Python & Ruby
	# tests, so instead we call the test twice, expecting it to fail for production.
	cd examples/context/java && javac -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext.java && ENV="development" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext
	cd examples/context/java && (ENV="production" java -cp $(JAVA_PACKAGE_JAR_PATH):. TestContext && exit 1 || exit 0)
	@echo "Ignore that failure it actually worked!"

python-test:
	cd examples/inheritance/python && pytest .
	cd examples/abac/python && pytest .
	cd examples/rbac/python && pytest .
	cd examples/context/python && pytest .
	cd examples/user_types/python && pytest .
	cd examples/list-filtering/sqlalchemy && pytest sqlalchemy_example
	$(MAKE) -C examples/list-filtering/django test

ruby-test:
	cd examples/abac/ruby && bundle exec rspec test_spec.rb
	cd examples/rbac/ruby && bundle exec rspec test_spec.rb
	cd examples/context/ruby && bundle exec rspec test_spec.rb
	cd examples/inheritance/ruby && bundle exec rspec test_spec.rb
	cd examples/user_types/ruby && bundle exec rspec test_spec.rb

nodejs-test:
	$(MAKE) -C ../languages/js build
	cd examples/rbac/nodejs && yarn && yarn jest
	cd examples/abac/nodejs && yarn && yarn jest
	cd examples/context/nodejs && yarn && yarn jest
	cd examples/inheritance/nodejs && yarn && yarn jest
	cd examples/user_types/nodejs && yarn && yarn jest

test-quickstarts:
	$(MAKE) -C examples/quickstart test

test: doctest
	$(MAKE) python-test
	$(MAKE) ruby-test
	$(MAKE) java-test
	$(MAKE) nodejs-test

deps:
	pip3 install -r requirements-docs.txt
