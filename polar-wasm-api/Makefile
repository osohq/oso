.PHONY: build test

CARGO_FLAGS=--dev
build:
	wasm-pack build $(CARGO_FLAGS) --target nodejs
	rm pkg/.gitignore pkg/package.json
	cp -r pkg/. ../languages/js/src
	rm -rf pkg

test:
	wasm-pack test --node
