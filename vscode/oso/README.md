# Oso VS Code Extension

## Syntax Highlighting

Syntax highlighting for `.polar` files as used by [Oso](https://www.osohq.com).

## Diagnostics (Errors & Warnings)

Diagnostics (errors & warnings) from an Oso policy in the current workspace
will be displayed inline in the editor and in the **Problems** pane.

### Known Issues

- The extension will not be alerted when a directory containing Polar files is
  deleted from outside the VS Code client. Note that running `rm -r` in VS
  Code's built-in terminal still counts as triggering the deletion from outside
  the VS Code client. This is due to [a limitation of VS Code's file
  watcher][60813]. If you delete a directory some other way than through the
  right-click menu in VS Code's file tree, the simplest remediation is to
  restart the Oso extension.

[60813]: https://github.com/microsoft/vscode/issues/60813

## Metrics

The extension collects **non-identifiable** metrics that we use to improve Oso.
We collect data into un-timestamped batches instead of sending it on every
policy load since we care about aggregate statistics, not tracking your
personal development behavior. **We will never sell this data**.

For more info on exactly what is tracked and why, see [this page][docs] in the
docs.

[docs]: https://docs.osohq.com/reference/tooling/ide/metrics.html
