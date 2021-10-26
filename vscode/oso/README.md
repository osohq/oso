# Oso VSCode Extension

## Syntax Highlighting

Syntax highlighting for `.polar` files as used by [Oso](https://www.osohq.com).

## Language Server Protocol (LSP) Functionality

Coming soon.

### Known Issues

- The extension will not be alerted when a directory containing Polar files is
  deleted from outside the VSCode client. Note that running `rm -r` in VSCode's
  built-in terminal still counts as triggering the deletion from outside the
  VSCode client. This is due to [a limitation of VSCode's file watcher][60813].
  If you delete a directory some other way than through the right-click menu in
  VSCode's file tree, the simplest remediation is to restart the Oso extension.

[60813]: https://github.com/microsoft/vscode/issues/60813
