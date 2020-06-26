# Minimal makefile for Sphinx documentation
#

# You can set these variables from the command line, and also
# from the environment for the first two.
SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = .
BUILDDIR      = _build

export BUNDLE_GEMFILE := $(abspath ../languages/ruby/Gemfile)

# Put it first so that "make" without argument is like "make help".
help:
	@$(SPHINXBUILD) -M help "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

.PHONY: help Makefile

# Catch-all target: route all unknown targets to Sphinx using the new
# "make mode" option.  $(O) is meant as a shortcut for $(SPHINXOPTS).
%: Makefile deps
	@$(SPHINXBUILD) -M $@ "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) $(O)

livehtml: deps
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

deps:
	pip3 install -r requirements-docs.txt
