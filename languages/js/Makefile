.PHONY: wasm install clean dev build fmtcheck typecheck lint test

CARGO_FLAGS=
export CARGO_FLAGS
wasm: clean install
	rm -f src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts
	$(MAKE) -C ../../polar-wasm-api build
	mkdir -p dist/src
	cp src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts dist/src/

install:
	yarn install

clean:
	yarn clean

dev: wasm
	yarn tsc --project tsconfig.build.json --watch

build: wasm
	yarn tsc --project tsconfig.build.json

fmtcheck: install
	yarn fmtcheck

typecheck: install wasm
	yarn tsc

lint: fmtcheck typecheck

parity: install wasm
	yarn ts-node test/parity.ts

test: install wasm
	yarn test
