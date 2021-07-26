import {
	createConnection,
	TextDocuments,
	ProposedFeatures,
	Connection,
	RenameFilesParams,
	DeleteFilesParams
} from 'vscode-languageserver/node';

import {
	TextDocument
} from 'vscode-languageserver-textdocument';
import { Polar } from './polar_analyzer';
import { getCapabilities, registerCapabilities, updateSettings } from './configuration';
import { tryLoadFile, validateContents } from './polar';
import { getCompletions, resolveCompletions } from './completion';
import { getDocumentSymbols } from './symbols';
import { getHoverInfo } from './hover';

export class Server {
	connection: Connection;
	documents: TextDocuments<TextDocument>;
	polar: Polar;

	constructor() {
		this.connection = createConnection(ProposedFeatures.all);
		this.documents = new TextDocuments(TextDocument);
		this.polar = new Polar();
	}
}

export function log(...msg: any) {
	server.connection.console.log(msg);
}

// Create a connection for the server, using Node's IPC as a transport.
// Also include all preview / proposed LSP features.
export const server = new Server();

const connection = server.connection;
const documents = server.documents;
const polar = server.polar;


/* ===== Register hooks ===== */


/* Configuration hooks */

connection.onInitialize(getCapabilities);
connection.onInitialized(() => {
	connection.window.showInformationMessage("Hello! Thank you for using the Oso integration :)");
});

// update settings when configuration changes
connection.onDidChangeConfiguration(updateSettings);

// revalidate everything
// connection.onDidChangeConfiguration(_ => {
// 	documents.all().forEach(validateContents);
// });

// Once initialized, need to set a few settings on the connection
registerCapabilities(connection);

/* Workspace functions */

// rename file (removes it and re-adds it)
connection.workspace.onDidRenameFiles((params: RenameFilesParams) => {
	params.files.forEach(file => {
		try {
			polar.rename(file.oldUri, file.newUri);
		} catch (error) {
			// ignore
		}
	});
});

// delete file
connection.workspace.onDidDeleteFiles((params: DeleteFilesParams) => {
	params.files.forEach(file => {
		polar.delete(file.uri);
	});
});



// The content of a text document has changed. This event is emitted
// when the text document first opened or when its content has changed.
documents.onDidOpen;
documents.onDidChangeContent(change => {
	log(`Document change: ${JSON.stringify(change)}`);
	const doc = change.document;

	const { success, diagnostics } = tryLoadFile(doc);
	connection.sendDiagnostics({ uri: doc.uri, diagnostics });
	if (success) {
		const diagnostics = validateContents(doc);
		connection.sendDiagnostics({ uri: doc.uri, diagnostics });
	}
});

/* Completion */
connection.onCompletion(getCompletions);
connection.onCompletionResolve(resolveCompletions);

/* Hover */
connection.onHover(getHoverInfo);

/* Symbols */
// Get all document symbols
connection.onDocumentSymbol(getDocumentSymbols);



/* Start the server */

// Make the text document manager listen on the connection
// for open, change and close text document events
documents.listen(connection);

// Listen on the connection
connection.listen();
