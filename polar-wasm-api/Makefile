.PHONY: build clean test nodejs bundler web

CARGO_FLAGS ?= --dev

build: nodejs

clean:
	rm -rf pkg

nodejs bundler web: clean
	wasm-pack build $(CARGO_FLAGS) --target $@ -d pkg
	rm pkg/.gitignore pkg/package.json
	cp pkg/polar_wasm_api.d.ts ../languages/js/src/polar_wasm_api.d.ts


test:
	wasm-pack test --node
