# Oso VSCode Extension

## Syntax Highlighting

Syntax highlighting for `.polar` files as used by [Oso](https://www.osohq.com).

## Language Server Protocol (LSP) Functionality

### Development

#### Requirements

- Latest stable version of Rust with `cargo` available on your system PATH.
- [`wasm-pack`][wasm-pack] 0.9.1+ installed and available on your system PATH.
- VSCode 1.52.0+.

#### Steps to test the extension out in VSCode

1. Run `make build` in the current directory (where this README lives).
2. Open the current directory in VSCode: `code .`.
3. Run the `Launch Client` launch configuration (defined in
   `.vscode/launch.json`).

[wasm-pack]: https://rustwasm.github.io/wasm-pack/installer/
