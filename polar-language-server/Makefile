.PHONY: build test require-out-dir

CARGO_FLAGS ?= --dev

build: require-out-dir
	wasm-pack --quiet build $(CARGO_FLAGS) --target nodejs --out-dir $(OUT_DIR)
	rm -f $(OUT_DIR)/.gitignore $(OUT_DIR)/package.json

test:
	wasm-pack test --node

require-out-dir:
ifndef OUT_DIR
	$(error OUT_DIR is not defined)
endif