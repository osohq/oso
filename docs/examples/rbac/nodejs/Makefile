.PHONY: test

test: node_modules
	yarn jest

node_modules: package.json
	yarn
	@touch $@
