.PHONY: build wasm

build: wasm node_modules
	@yarn compile

wasm:
	@$(MAKE) -C ../../polar-language-server build

node_modules: package.json client/package.json server/package.json
	@yarn install
	@touch $@
