.PHONY: rust install test lint typecheck repl

rust:
	$(MAKE) -C ../.. rust-build

install: rust
	bundle install

test: install
	bundle exec rake spec

lint: install
	bundle exec rubocop

typecheck: install
	bundle exec solargraph typecheck

repl: install
	bundle exec oso
