.PHONY: build test

CARGO_FLAGS ?= --dev

build:
	wasm-pack --quiet build $(CARGO_FLAGS) --target nodejs --out-dir $(OUT_DIR)
	rm -f $(OUT_DIR)/.gitignore $(OUT_DIR)/package.json

test:
	wasm-pack test --node
