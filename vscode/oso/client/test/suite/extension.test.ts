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
      new Range(
        new Position(0, 0),
        new Position(0, diagnostics[0].message.length)
      )
    )
  );
}

// Helper that waits for `n` diagnostics to appear and then returns them.
async function getDiagnostics(n: number): Promise<[Uri, Diagnostic[]][]> {
  let diagnostics: [Uri, Diagnostic[]][] = [];
  for (;;) {
    diagnostics = languages.getDiagnostics();
    if (diagnostics.length === n) break;
    if (diagnostics.length > n) throw new Error('too many diagnostics');
    await new Promise(r => setTimeout(r, 0));
  }
  return diagnostics;
}

suite('Diagnostics', () => {
  test('We receive a diagnostic for each Polar file in the workspace', async () => {
    const polarFileCount = (await workspace.findFiles('*.polar')).length;
    const diagnostics = await getDiagnostics(polarFileCount);
    check(diagnostics[0], 'apple.polar');
    check(diagnostics[1], 'banana.polar');
  });
});
