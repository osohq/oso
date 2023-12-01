.PHONY: rust copy_lib install test lint typecheck repl

UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

rust:
	$(MAKE) -C ../.. rust-build

copy_lib:
	mkdir -p ext/oso-oso/lib/
ifeq ($(UNAME_S),Linux)
ifeq ($(UNAME_M),x86_64)
	cp ../../target/debug/libpolar.so ext/oso-oso/lib/libpolar-x86_64.so
else
	cp ../../target/debug/libpolar.so ext/oso-oso/lib/libpolar-aarch64.so
endif
endif
ifeq ($(UNAME_S),Darwin)
	cp ../../target/debug/libpolar.dylib ext/oso-oso/lib/
endif

install:
	bundle install

test: install rust copy_lib
	POLAR_IGNORE_NO_ALLOW_WARNING=1 bundle exec rake spec

lint: install
	bundle exec rubocop

typecheck: install
	bundle exec solargraph typecheck

repl: install rust copy_lib
	bundle exec oso
