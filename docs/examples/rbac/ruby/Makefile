# By unexporting BUNDLE_GEMFILE here, we prevent bundler from using the
# Gemfile defined in `docs/examples/Makefile`.
unexport BUNDLE_GEMFILE

.PHONY: test install

test: install
	bundle exec rspec .

install: .make.dependencies.installed

.make.dependencies.installed: Gemfile
	bundle
	@touch $@

