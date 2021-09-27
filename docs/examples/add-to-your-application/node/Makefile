.PHONY: test

test: node_modules
	@yarn link oso --silent
	@node test.js
	@yarn unlink oso --silent

node_modules: package.json
	@yarn
	@cd ../../../../languages/js; yarn link
	@touch $@
