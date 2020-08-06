.PHONY: rust install test lint typecheck repl

rust:
	$(MAKE) -C ../.. rust-build

install: rust
	bundle install

test: install
	bundle exec rake spec

lint:
	bundle exec rubocop

typecheck:
	bundle exec solargraph typecheck

repl: install
	bundle exec oso
