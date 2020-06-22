.PHONY: install rust test

rust:
	$(MAKE) -C ../.. rust-build

install: rust
	bundle install

test: install
	bundle exec rake spec
