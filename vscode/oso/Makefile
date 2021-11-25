.PHONY: build package wasm fmtcheck lint test clean

build: clean wasm
	yarn compile

package: clean wasm
	yarn vsce package

CARGO_FLAGS ?= --dev

wasm:
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) -C ../../polar-language-server build

node_modules: package.json client/package.json server/package.json
	yarn install
	@touch $@

fmtcheck: clean
	yarn fmtcheck

lint: clean wasm
	yarn lint

test: clean wasm
	yarn test

clean: node_modules
	rm -rf client/out server/out
	mkdir -p client/out server/out
	cp client/node_modules/blake3-wasm/dist/wasm/nodejs/blake3_js_bg.wasm client/out
