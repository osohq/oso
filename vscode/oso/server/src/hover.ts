import { Hover, HoverParams, MarkupContent } from 'vscode-languageserver';
import { log, server } from './server';
import { TermInfo } from './symbols';


export function getHoverInfo(params: HoverParams): Hover | null {
	const filename = params.textDocument.uri;
	const document = server.documents.get(filename);
	if (document !== undefined) {
		const location = document?.offsetAt(params.position);
		const symbol: TermInfo | undefined = server.polar.getSymbolAt(filename, location);
		if (symbol !== undefined) {
			const [_, left, right] = symbol.location;
			const content: MarkupContent = {
				value: symbol.details || symbol.type,
				kind: 'markdown'
			};
			return {
				contents: content,
				range: {
					start: document?.positionAt(left),
					end: document.positionAt(right)
				}
			};
		}
	}
	return null;
}