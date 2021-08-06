import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import * as net from 'net';

import {
	Executable,
	integer,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	StreamInfo,
} from 'vscode-languageclient/node';

let client: LanguageClient;

type LspServerConfig = {
	mode: "embedded" | "server",
	port: integer,
}

export function activate(context: ExtensionContext) {
	const config = workspace.getConfiguration("polarAnalyzer");
	const lsp: LspServerConfig = config.get("lspServer");

	let serverOptions: ServerOptions | undefined;

	switch (lsp.mode) {
		case "embedded": {
			// The server is implemented in node
			const serverBinary = context.asAbsolutePath(
				path.join('..', '..', 'target', 'debug', 'polar-analyzer')
			);
			// // The debug options for the server
			// // --inspect=6009: runs the server in Node's Inspector mode so VS Code can attach to the server for debugging
			// const debugOptions = { execArgv: ['--nolazy', '--inspect=6009'] };

			// If the extension is launched in debug mode then the debug server options are used
			// Otherwise the run options are used
			serverOptions = {
				command: serverBinary,
				options: {
					detached: false,
				}
			};
			break;
		}
		case "server": {
			const connectionInfo = {
				port: lsp.port
			};
			serverOptions = () => {
				// Connect to language server via socket
				const socket = net.connect(connectionInfo);
				const result: StreamInfo = {
					writer: socket,
					reader: socket
				};
				return Promise.resolve(result);
			};
			break;
		}
	}

	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ language: 'polar' }],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		}
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'OsoLsp',
		'Oso Extension',
		serverOptions,
		clientOptions
	);

	// Start the client. This will also launch the server
	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
