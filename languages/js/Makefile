.PHONY: wasm install clean dev build fmtcheck typecheck lint parity test

wasm: clean
	rm -f src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts
	$(MAKE) -C ../../polar-wasm-api build
	mkdir -p dist/src
	cp src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts dist/src/

install:
	yarn install

clean: install
	yarn clean

dev: wasm
	yarn tsc --project tsconfig.build.json --watch

build: wasm
	yarn tsc --project tsconfig.build.json

fmt: clean
	yarn fmt

fmtcheck: clean
	yarn fmtcheck

typecheck: wasm
	yarn tsc

lint: fmtcheck typecheck

parity: wasm
	yarn ts-node test/parity.ts

test: wasm
	yarn test

repl: build
	./bin/repl.js
