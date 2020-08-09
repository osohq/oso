.PHONY: build test

build:
	wasm-pack build --dev --target nodejs
	rm pkg/.gitignore pkg/package.json
	mkdir -p ../languages/js/dist
	cp --recursive pkg/. ../languages/js/dist
	rm -rf pkg

test:
	wasm-pack test --node
