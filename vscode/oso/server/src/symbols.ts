/* Run Polar checks */

import { DocumentSymbolParams, DocumentUri, SymbolInformation, SymbolKind } from 'vscode-languageserver';
import { log, server } from './server';

export type RuleInfo = {
	symbol: string;
	signature: string;
	location: [string, number, number];
};


export type TermInfo = {
	name: string;
	location: [string, number, number];
	type: string;
	term: any;
	details?: string;
};


/** Get all symbols in the document (rules, variables, etc.) */
export function getDocumentSymbols(params: DocumentSymbolParams): SymbolInformation[] {
	log(`getDocumentSymbols for ${params.textDocument.uri}`);
	const doc = server.documents.get(params.textDocument.uri);
	const result: SymbolInformation[] = [];

	if (doc !== undefined) {
		const rules: {
			symbol: string,
			signature: string,
			location: [string, number, number]
		}[]
			= server.polar.getRuleInfo(params.textDocument.uri);


		rules.forEach((rule: RuleInfo) => {
			const currentDocUri: DocumentUri = rule.location[0];
			const symbolSummary: SymbolInformation = {
				name: rule.symbol,
				kind: SymbolKind.Method,
				location: {
					uri: currentDocUri,
					range: {
						start: doc.positionAt(rule.location[1]),
						end: doc.positionAt(rule.location[2])
					}
				}
			};
			result.push(symbolSummary);

		});
	}
	return result;
}
