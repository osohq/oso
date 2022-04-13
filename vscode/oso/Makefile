.PHONY: test lint typecheck fmtcheck fmt package wasm clean

test: wasm
	yarn test

lint: wasm
	yarn lint

typecheck: wasm
	yarn typecheck

fmtcheck: clean
	yarn fmtcheck

fmt: clean
	yarn fmtwrite

package: wasm
	yarn package

CARGO_FLAGS ?= --dev

wasm: clean
	$(MAKE) CARGO_FLAGS=$(CARGO_FLAGS) OUT_DIR=$$(pwd)/out -C ../../polar-language-server build

node_modules: package.json
	yarn install
	@touch $@

clean: node_modules
	rm -rf out
	mkdir -p out
