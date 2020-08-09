.PHONY: build-wasm install dev

build-wasm:
	$(MAKE) -C ../../polar-wasm-api build

install:
	yarn install

dev: build-wasm install
	yarn dev
