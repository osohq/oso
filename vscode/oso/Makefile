.PHONY: build package wasm fmtcheck lint test clean

build: clean wasm node_modules
	yarn compile

package: clean wasm node_modules
	yarn vsce package

CARGO_FLAGS ?= --dev

wasm:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../../polar-language-server build

node_modules: package.json client/package.json server/package.json
	yarn install
	@touch $@

fmtcheck: clean node_modules
	yarn fmtcheck

lint: clean wasm node_modules
	yarn lint

test: clean wasm node_modules
	yarn test

clean:
	rm -rf client/out server/out
