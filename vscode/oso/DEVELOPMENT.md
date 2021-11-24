# Development

## Language Server Protocol (LSP) Functionality

### Requirements

- Latest stable version of Rust with `cargo` available on your system PATH.
- [`wasm-pack`][wasm-pack] 0.9.1+ installed and available on your system PATH.
- VSCode 1.52.0+.

### Steps to test the extension out in VSCode

1. Run `make package` in the current directory (where this file lives).
2. Install the output `.vsix` file into your local VSCode instance: `code
   --install-extension oso-X.Y.Z.vsix`.
3. Run the **Developer: Reload Window** (`workbench.action.reloadWindow`)
   command.

### Steps to build a release version of the extension

1. Run `make CARGO_FLAGS=--release package` in the current directory (where
   this file lives).
2. The resulting `oso-X.Y.Z.vsix` file can be installed into any VSCode
   instance via: `code --install-extension oso-X.Y.Z.vsix`.

### Running tests

#### Server

Tests for the server live in the `polar-language-server` crate. Run `make -C
polar-language-server test` from the root of this repository.

#### Client

Tests for the client live in the `./client/test` directory relative to this
file. Run `make test` in the current directory (where this file lives), which
invokes `yarn test`, which:

- calls `yarn compile` to build the `client` and `server` TypeScript projects
  (including the `client/test` directory, which is really all we care about
  since `yarn esbuild` builds/bundles everything except the tests)
- calls `yarn esbuild` to build the `client/out/main.js` and
  `server/out/server.js` files the same way we do for a release so we're
  running the end-to-end VSCode integration tests against the same code we'll
  be releasing
- invokes `node ./client/out/test/runTest.js` to run the end-to-end tests.

[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
