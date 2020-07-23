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

RUBY_FILES := $(wildcard ../languages/ruby/**/*.rb)
JAVA_FILES := $(wildcard ../languages/java/**/*.java)

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile api-docs html doctest

_api_docs/ruby: $(RUBY_FILES)
	mkdir -p _api_docs
	cd "$(RUBY_DIR)" && yard doc
	rm -rf _api_docs/ruby
	cp -R $(RUBY_DIR)/doc _api_docs/ruby

_api_docs/java: $(JAVA_FILES)
	cd "$(JAVA_DIR)" && mvn javadoc:javadoc
	rm -rf _api_docs/java
	cp -R $(JAVA_DIR)/target/site/apidocs _api_docs/java

api-docs: _api_docs/ruby _api_docs/java

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
html doctest spelling: Makefile deps api-docs
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

clean:
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)
	rm -r _api_docs

livehtml: deps api-docs
	@sphinx-autobuild -b html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O) -i *.tmp

test: doctest
	cd examples/inheritance/python && pytest .
	cd examples/abac/python && pytest .
	cd examples/rbac/python && pytest .
	cd examples/context/python && pytest .

	cd examples/abac/ruby && bundle exec rspec test_spec.rb
	cd examples/rbac/ruby && bundle exec rspec test_spec.rb
	cd examples/context/ruby && bundle exec rspec test_spec.rb
	cd examples/inheritance/ruby && bundle exec rspec test_spec.rb

deps:
	pip3 install -r requirements-docs.txt
