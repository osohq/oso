import * as vscode from 'vscode';
import * as assert from 'assert';
import { getDocUri, activate } from './helper';

suite('Should get diagnostics', () => {
	const missingRules = getDocUri('diag-missing-rule.polar');
	const parserError = getDocUri('diag-parser-error.polar');

	test('Diagnoses parser errors', async () => {
		await testDiagnostics(parserError, [
			{ message: 'hit the end of the file unexpectedly. Did you forget a semi-colon at line 1, column 39 in file file:///home/sam/work/oso/oso/vscode/oso/client/testFixture/diag-parser-error.polar', range: toRange(0, 38, 0, 38), severity: vscode.DiagnosticSeverity.Error, source: 'polar' },
		]);
	});

	test('Diagnoses missing rules', async () => {
		await testDiagnostics(missingRules, [
			{ message: 'Rule does not exist: There are no rules matching the format:\n  f(3)\nFound:\n  f(1);\n  f(2);\n', range: toRange(3, 9, 3, 13), severity: vscode.DiagnosticSeverity.Warning, source: 'ex' },
		]);
	});
});

function toRange(sLine: number, sChar: number, eLine: number, eChar: number) {
	const start = new vscode.Position(sLine, sChar);
	const end = new vscode.Position(eLine, eChar);
	return new vscode.Range(start, end);
}

async function testDiagnostics(docUri: vscode.Uri, expectedDiagnostics: vscode.Diagnostic[]) {
	await activate(docUri);

	const actualDiagnostics = vscode.languages.getDiagnostics(docUri);

	assert.strictEqual(actualDiagnostics.length, expectedDiagnostics.length);

	expectedDiagnostics.forEach((expectedDiagnostic, i) => {
		const actualDiagnostic = actualDiagnostics[i];
		assert.strictEqual(actualDiagnostic.message, expectedDiagnostic.message);
		assert.deepStrictEqual(actualDiagnostic.range, expectedDiagnostic.range);
		assert.strictEqual(actualDiagnostic.severity, expectedDiagnostic.severity);
	});
}