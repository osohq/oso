import * as assert from 'assert';

import { Diagnostic, languages, Position, Range, Uri } from 'vscode';

function check([uri, diagnostics]: [Uri, Diagnostic[]], expected: string) {
  const filename = [...uri.toString().split('/')].pop();
  assert.strictEqual(filename, expected);

  assert.strictEqual(diagnostics.length, 1);
  assert.strictEqual(diagnostics[0].message, filename);
  assert(
    diagnostics[0].range.isEqual(
      new Range(new Position(0, 0), new Position(0, filename.length))
    )
  );
}

suite('Diagnostics', () => {
  test('We receive a diagnostic for each Polar file in the workspace', () => {
    const diagnostics = languages.getDiagnostics();
    assert.strictEqual(diagnostics.length, 2);
    check(diagnostics[0], 'apple.polar');
    check(diagnostics[1], 'banana.polar');
  });
});
