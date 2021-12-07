.PHONY: build package wasm fmtcheck lint test clean

test: wasm
	yarn test

lint: wasm
	yarn lint

typecheck: wasm
	yarn typecheck

fmtcheck: clean
	yarn fmtcheck

package: wasm
	yarn package

CARGO_FLAGS ?= --dev

wasm: clean
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../../polar-language-server build
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../../polar-formatter build

node_modules: package.json
	yarn install
	@touch $@

clean: node_modules
	rm -rf out
	mkdir -p out
