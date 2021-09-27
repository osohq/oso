.PHONY: build test parity lint fmt fmtcheck typecheck docs repl clean install wasm

WASM_FILES = src/polar_wasm_api.js src/polar_wasm_api.d.ts src/polar_wasm_api_bg.wasm src/polar_wasm_api_bg.wasm.d.ts

# Build the WASM lib; then build the TS lib.
build: install wasm
	@yarn build

# Run the TS lib's test suite.
test: install wasm
	@yarn test

# Run the parity tests.
parity: install wasm
	@yarn ts-node test/parity.ts

# Check formatting and types and run ESLint.
lint: fmtcheck typecheck
	@yarn lint

# Tell ESLint to auto-fix some issues.
lint-fix: install
	@yarn fix

# Run the formatter, updating offending files.
fmt: install
	@yarn fmt

# Run the formatter without updating offending files.
fmtcheck: install
	@yarn fmtcheck

# Run the TS compiler to typecheck the TS lib.
typecheck: install wasm
	@yarn tsc

# Build the TS lib documentation.
docs: install wasm
	@yarn docs-build

# Start a REPL session.
repl: build
	@./bin/repl.js

# Remove existing build.
clean: install
	@yarn clean
	@rm -f $(WASM_FILES)

install: .make.deps.installed

# Only reinstall deps when package.json changes.
.make.deps.installed: package.json
	@yarn install --network-timeout 100000
	@touch $@

wasm: $(WASM_FILES)

# Rebuild the WASM core.
$(WASM_FILES):
	@$(MAKE) -C ../../polar-wasm-api nodejs
	@$(MAKE) -C ../../polar-wasm-api bundler
