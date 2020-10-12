.PHONY: wasm install clean dev build fmtcheck typecheck lint parity test

build: wasm
	yarn build

wasm: clean
	rm -f src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.wasm.d.ts
	$(MAKE) -C ../../polar-wasm-api build
	mkdir -p dist/src
	cp src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.wasm.d.ts dist/src/

install:
	yarn install --network-timeout 100000

clean: install
	yarn clean

dev: wasm
	yarn build --watch

fmt: clean
	yarn fmt

fmtcheck: clean
	yarn fmtcheck

typecheck: wasm
	yarn tsc

lint: fmtcheck typecheck

parity: build
	yarn ts-node test/parity.ts

test: build
	yarn test

repl: build
	./bin/repl.js

docs: install
	yarn docs-build
