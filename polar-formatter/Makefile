.PHONY: build

CARGO_FLAGS ?= --dev

build:
	wasm-pack --quiet build $(CARGO_FLAGS) --target nodejs --out-dir ../vscode/oso/out
	rm -f ../vscode/oso/out/.gitignore ../vscode/oso/out/package.json
