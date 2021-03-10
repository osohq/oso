.PHONY: build test parity lint fmt fmtcheck typecheck docs repl clean install wasm

# Build the WASM lib; then build the TS lib.
build: install wasm
	yarn build

# Run the TS lib's test suite.
test: install wasm
	yarn test

# Run the parity tests.
parity: install wasm
	yarn ts-node test/parity.ts

# Check formatting and types.
lint: fmtcheck typecheck

# Run the formatter, updating offending files.
fmt: install
	yarn fmt

# Run the formatter without updating offending files.
fmtcheck: install
	yarn fmtcheck

# Run the TS compiler to typecheck the TS lib.
typecheck: install wasm
	yarn tsc

# Build the TS lib documentation.
docs: install wasm
	yarn docs-build

# Start a REPL session.
repl: build
	./bin/repl.js

# Remove existing build.
clean: install
	yarn clean

install: .make.deps.installed

# Only reinstall deps when package.json changes.
.make.deps.installed: package.json
	yarn install --network-timeout 100000
	@touch $@

wasm: src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.wasm.d.ts

# Rebuild the WASM core.
src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.wasm.d.ts:
	$(MAKE) -C ../../polar-wasm-api nodejs
	$(MAKE) -C ../../polar-wasm-api bundler
