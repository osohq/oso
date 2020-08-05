.PHONY: build

build:
	wasm-pack build --dev --target nodejs
	rm pkg/.gitignore pkg/package.json
	cp --recursive pkg/. ../languages/js/lib
	rm -rf pkg
