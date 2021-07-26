
/* Completions -- text snippets + helpers */

import { CompletionItem, CompletionItemKind, CompletionParams, InsertTextFormat } from 'vscode-languageserver';
import { log } from './server';


/** Get text completions
 * 
 * This is more like an index of possible completions.
 * The completion itself is handled in `resolveCompletions`
*/
function getCompletions(params: CompletionParams): CompletionItem[] {
	log(`getDocumentSymbols for ${params.textDocument.uri}`);

	return [
		{
			label: "allow",
			kind: CompletionItemKind.Function,
			data: 1
		}
	];
}

function resolveCompletions(item: CompletionItem): CompletionItem {
	switch (item.data) {
		case 1:
			item.detail = "allow rule";
			item.documentation = "allow actor to perform action on resource";
			item.insertText = "allow(${1:actor}, ${2:action}, ${3:resource}) if\n    $0;";
			item.insertTextFormat = InsertTextFormat.Snippet;
	}

	return item;
}

export {
	getCompletions, resolveCompletions
};