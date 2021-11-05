.PHONY: build wasm fmtcheck lint test

build: wasm node_modules
	@yarn compile

wasm:
	@$(MAKE) -C ../../polar-language-server build

node_modules: package.json client/package.json server/package.json
	@yarn install
	@touch $@

fmtcheck: node_modules
	@yarn fmtcheck

lint: node_modules
	@yarn lint

test: node_modules
	@yarn test
