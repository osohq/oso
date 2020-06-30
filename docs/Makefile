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

RUBY_FILES := $(wildcard ../languages/ruby/**/*.rb)

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile api-docs html doctest

_api_docs/ruby: $(RUBY_FILES)
	mkdir -p _api_docs
	cd "$(RUBY_DIR)" && yard doc
	rm -rf _api_docs/ruby
	cp -R $(RUBY_DIR)/doc _api_docs/ruby

api-docs: _api_docs/ruby

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
html: Makefile deps api-docs
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

doctest: Makefile deps api-docs
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

livehtml: deps api-docs
	@sphinx-autobuild -b html "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

test: doctest
	python3 run_doc_tests.py learn-queries.polar
	python3 run_doc_tests.py externals-example/company.py externals-example/company.polar
	cd externals-example && python3 -m pytest

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
