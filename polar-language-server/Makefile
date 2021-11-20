.PHONY: build test

CARGO_FLAGS ?= --dev

build:
	wasm-pack --quiet build $(CARGO_FLAGS) --target nodejs --out-dir ../vscode/oso/server/out
	rm -f ../vscode/oso/server/out/.gitignore ../vscode/oso/server/out/package.json

test:
	wasm-pack test --node
