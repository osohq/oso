.PHONY: wasm install dev fmtcheck typecheck lint

wasm: clean install
	rm -f src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts
	$(MAKE) -C ../../polar-wasm-api build
	mkdir -p dist
	cp src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.d.ts dist

install:
	yarn install

clean:
	yarn clean

dev: wasm
	yarn tsc --watch

build: wasm
	yarn tsc

fmtcheck: install
	yarn fmtcheck

typecheck: install wasm
	yarn typecheck

lint: fmtcheck typecheck
