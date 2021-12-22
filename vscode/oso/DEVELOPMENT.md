# Development

## Language Server Protocol (LSP) Functionality

### Requirements

- Latest stable version of Rust with `cargo` available on your system PATH.
- [`wasm-pack`][wasm-pack] 0.9.1+ installed and available on your system PATH.
- VS Code 1.52.0+.

### Steps to test the extension out in VS Code

1. Run `make package` in the current directory (where this file lives).
2. Install the output `.vsix` file into your local VS Code instance: `code
   --install-extension oso-X.Y.Z.vsix`.
3. Run the **Developer: Reload Window** (`workbench.action.reloadWindow`)
   command.

### Steps to build a release version of the extension

1. Run `make CARGO_FLAGS=--release package` in the current directory (where
   this file lives).
2. The resulting `oso-X.Y.Z.vsix` file can be installed into any VS Code
   instance via: `code --install-extension oso-X.Y.Z.vsix`.

### Running tests

#### Server

Tests for the server live in the `polar-language-server` crate. Run `make -C
polar-language-server test` from the root of this repository.

#### Client

Tests for the client live in the `./test` directory relative to this file. Run
`make test` in the current directory (where this file lives), which invokes
`yarn test`, which:

- Calls `tsc --build test` to build the TypeScript project in the `./test`
  directory.
- Calls `yarn esbuild-all` to build the `./out/client.js` and `./out/server.js`
  files the same way we do for a release so we're running the end-to-end VS
  Code integration tests against the same code we'll be releasing.
- Invokes `node ./out/test/src/runTest.js` to run the end-to-end tests.

[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
