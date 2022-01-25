.PHONY: a b

a: node_modules
	yarn a

b: node_modules
	yarn b

node_modules: package.json
	yarn
	cd ../../../../languages/js; yarn link
	yarn link oso
	@touch $@