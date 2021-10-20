import * as assert from 'assert';

import { Diagnostic, languages, Position, Range, Uri, workspace } from 'vscode';

// TODO(gj): This is temporary and will be replaced when we have real
// diagnostics.
function check([uri, diagnostics]: [Uri, Diagnostic[]], expected: string) {
  const filename = [...uri.toString().split('/')].pop();
  assert.strictEqual(filename, expected);

  assert.strictEqual(diagnostics.length, 1);
  assert.strictEqual(diagnostics[0].message.trim(), filename);
  assert(
    diagnostics[0].range.isEqual(
      new Range(new Position(0, 0), new Position(0, filename.length))
    )
  );
}

// Spin until workspace is fully loaded. There might be a race condition
// where we call `languages.getDiagnostics()` before the extension has loaded
// all of the Polar files and emitted a diagnostic for each.
async function waitForWorkspaceToLoad() {
  for (;;) {
    const uris = await workspace.findFiles('*');
    switch (uris.length) {
      case 0:
      case 1:
      case 2:
        console.log(
          'uris =>',
          uris.map(u => [...u.toString().split('/')].pop())
        );
        continue;
      case 3:
        return;
      default:
        throw new Error();
    }
  }
}

suite('Diagnostics', () => {
  test('We receive a diagnostic for each Polar file in the workspace', async () => {
    await waitForWorkspaceToLoad();
    const diagnostics = languages.getDiagnostics();
    assert.strictEqual(diagnostics.length, 2);
    check(diagnostics[0], 'apple.polar');
    check(diagnostics[1], 'banana.polar');
  });
});
