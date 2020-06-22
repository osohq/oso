.PHONY: bundler install test

bundler:
	$(MAKE) -C ../../.. rust-build

install: bundler
	bundle install

test: install
	bundle exec rake spec
