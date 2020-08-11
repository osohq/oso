.PHONY: build-wasm install dev checkfmt typecheck lint

build-wasm:
	$(MAKE) -C ../../polar-wasm-api build

install:
	yarn install

dev: build-wasm install
	yarn dev

checkfmt: install
	yarn checkfmt

typecheck: install build-wasm
	yarn compile

lint: checkfmt typecheck
