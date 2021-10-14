.PHONY: build wasm

build: wasm node_modules

wasm:
	@$(MAKE) -C ../../polar-language-server build
	@wasm-bindgen --out-dir server/out --target nodejs --debug ../../target/wasm32-unknown-unknown/debug/polar_language_server.wasm --keep-debug

node_modules: package.json client/package.json server/package.json
	@yarn
	@touch $@
