.PHONY: wasm install clean dev build fmtcheck typecheck lint parity test

build: wasm
	yarn build


wasm: clean
	mkdir -p dist/bundler
	mkdir -p dist/src
	$(MAKE) -C ../../polar-wasm-api nodejs
	cp -r ../../polar-wasm-api/pkg/* dist/src
	cp -r ../../polar-wasm-api/pkg/* src/
	$(MAKE) -C ../../polar-wasm-api bundler
	cp -r ../../polar-wasm-api/pkg/* dist/bundler/

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

jsbuildtest:
	yarn build
	yarn test

repl: build
	./bin/repl.js

docs: install
	yarn docs-build
