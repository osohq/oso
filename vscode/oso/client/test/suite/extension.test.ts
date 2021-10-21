import * as assert from 'assert';

import { Diagnostic, languages, Uri, workspace } from 'vscode';

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
    const polarFiles = await workspace.findFiles('*.polar');
    const diagnostics = await getDiagnostics(polarFiles.length);
    for (const [uri, [diagnostic]] of diagnostics) {
      const { line, character } = diagnostic.range.start;
      assert.strictEqual(
        diagnostic.message,
        `hit the end of the file unexpectedly. Did you forget a semi-colon at line ${
          line + 1
        }, column ${character + 1} in file ${uri.toString()}`
      );
    }
  });
});
