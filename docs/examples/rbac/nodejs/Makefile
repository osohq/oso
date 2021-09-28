.PHONY: test

test: node_modules
	yarn jest

node_modules: package.json
	yarn
	cd ../../../../languages/js; yarn link
	yarn link oso
	@touch $@
