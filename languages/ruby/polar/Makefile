.PHONY: bundler install test

bundler:
	$(MAKE) -C ../../.. rust-build
	gem install bundler

install: bundler
	bundle install

test: install
	bundle exec rake spec
