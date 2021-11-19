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
3. Restart VSCode (if it was already open).

### Steps to build a release version of the extension

1. Run `make CARGO_FLAGS=--release package` in the current directory (where
   this file lives).
2. The resulting `oso-X.Y.Z.vsix` file can be installed into any VSCode
   instance via: `code --install-extension oso-X.Y.Z.vsix`.

[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
