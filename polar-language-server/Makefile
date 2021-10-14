.PHONY: build

build:
	@wasm-pack --quiet build --dev --target nodejs --out-dir ../vscode/oso/server/out
	@rm -f ../vscode/oso/server/out/.gitignore ../vscode/oso/server/out/package.json
