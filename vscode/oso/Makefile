.PHONY: build

build:
	@$(MAKE) -C ../../polar-language-server build
	@wasm-bindgen --out-dir server/out --target nodejs --debug ../../target/wasm32-unknown-unknown/debug/polar_language_server.wasm --keep-debug
