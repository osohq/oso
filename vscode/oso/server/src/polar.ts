/* Run Polar checks */

import { Diagnostic, DiagnosticSeverity, integer } from 'vscode-languageserver';
import { TextDocument } from 'vscode-languageserver-textdocument';
import { log, server } from './server';

type ErrorContext = {
	loc: integer,
}

type PolarError = {
	message: string,
	context?: ErrorContext,
	kind: any,
}

/** Convert a PolarError to a diagnostic error for the provided text document */
function errorToDiagnostic(textDocument: TextDocument, polarError: PolarError): Diagnostic {
	const loc = polarError.context?.loc || -1;
	return {
		severity: DiagnosticSeverity.Error,
		message: polarError.message,
		range: {
			start: textDocument.positionAt(loc),
			end: textDocument.positionAt(loc)
		},
		source: 'polar'
	};
}

/** 
 Attempt to load the policy file into the Polar knowledge base
 Any errors will be returned to the client.

 returns `false` if loading fails. In which case the knowledge
 base will be left unchanged
*/
function tryLoadFile(textDocument: TextDocument): { success: boolean, diagnostics: Diagnostic[] } {
	log(`tryLoadFile ${textDocument.uri}`);
	const polar = server.polar;
	const policy = textDocument.getText();
	const errors = polar.getParseErrors(policy);
	const diagnostics: Diagnostic[] = [];
	let success = false;
	if (errors.length === 0) {
		// no parse errors! Lets try loading the policy for real
		const filename = textDocument.uri;
		try {
			polar.load(policy, filename);
			success = true;
		} catch (error) {
			diagnostics.push(errorToDiagnostic(textDocument, error));
		}
	} else {
		// parse errors :( 
		// send them back to the client
		errors.forEach((error: PolarError) =>
			diagnostics.push(errorToDiagnostic(textDocument, error)));

	}
	return {
		success, diagnostics
	};
}

/** Run diagnostics for the text document */
function validateContents(textDocument: TextDocument): Diagnostic[] {
	log(`getDocumentSymbols for ${textDocument.uri}`);
	const polar = server.polar;
	const diagnostics: Diagnostic[] = [];
	const policy = textDocument.getText();
	const unused_rules = polar.getUnusedRules(policy);
	unused_rules.forEach(([ruleName, left, right]: [string, integer, integer]) => {
		const diagnostic: Diagnostic = {
			severity: DiagnosticSeverity.Warning,
			message: `Rule does not exist: ${ruleName}`,
			range: {
				start: textDocument.positionAt(left),
				end: textDocument.positionAt(right)
			},
			source: 'polar'
		};
		diagnostics.push(diagnostic);
	});
	return diagnostics;
}


export {
	tryLoadFile,
	validateContents,
};