.PHONY: build package wasm fmtcheck lint test clean

build: wasm
	yarn compile

package: wasm
	yarn vsce package

CARGO_FLAGS ?= --dev

wasm: clean
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../../polar-language-server build

node_modules: package.json
	yarn install
	@touch $@

fmtcheck: clean
	yarn fmtcheck

lint: wasm
	yarn lint

test: wasm
	yarn test

clean: node_modules
	rm -rf out
	mkdir -p out
