.PHONY: build test

build:
	wasm-pack build --dev --target nodejs
	rm pkg/.gitignore pkg/package.json
	cp -r pkg/. ../languages/js/src
	rm -rf pkg

test:
	wasm-pack test --node
