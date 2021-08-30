import * as path from 'path';
import { window, workspace, ExtensionContext, commands, WorkspaceSymbolProvider, SymbolInformation, SymbolKind } from 'vscode';

import * as net from 'net';

import {
	ClientCapabilities,
	DocumentSelector,
	Executable,
	InitializeParams,
	integer,
	LanguageClient,
	LanguageClientOptions,
	RequestType,
	ServerCapabilities,
	ServerOptions,
	StaticFeature,
	StreamInfo,
	TextDocumentFeature,
} from 'vscode-languageclient/node';

let client: LanguageClient;

type LspServerConfig = {
	mode: "embedded" | "server",
	port: integer,
}

export async function activate(context: ExtensionContext) {
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

	const output = window.createOutputChannel("Oso Extension");

	const log = function (kind, msg) {
		output.appendLine(`[${kind}] ${msg}`);
	};
	const info = (msg) => log("info", msg);
	// const warn = (msg) => log("warn", msg);
	// const error = (msg) => log("error", msg);


	// Options to control the language client
	const clientOptions: LanguageClientOptions = {
		// Register the server for plain text documents
		documentSelector: [{ language: 'polar' }],
		synchronize: {
			// Notify the server about file changes to '.clientrc files contained in the workspace
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		},
		outputChannel: output
	};

	// Create the language client and start the client.
	client = new LanguageClient(
		'OsoLsp',
		'Oso Extension',
		serverOptions,
		clientOptions
	);

	// client.registerFeature(new SymbolLookupFeature());
	info("The channel works!");
	client.onReady().then(() => {
		info("Configuring the getAllSymbols handler");
		// the client handles a method for polar analyer, to find all workspace params
		client.onRequest(getAllSymbols, async (params) => {
			info("Got a request to get all workspace symbols!");
			info(`Requested symbols: ${params.names}`);
			const symbols = [];
			for (const symbol in params.names) {
				const matches: SymbolInformation[] = await commands.executeCommand('vscode.executeWorkspaceSymbolProvider', symbol);
				info(`Returned symbols: ${matches.map(m => m.name)}`);
				symbols.push(...matches);
			}
			const global: SymbolInformation[] = await commands.executeCommand('vscode.executeWorkspaceSymbolProvider', "");
			info(`Returned symbols: ${global.map(m => m.name)}`);
			symbols.push(...global);
			info(`Returned symbols: ${symbols.map(s => s.toString()).join(",")}`);
			return {
				"classes": symbols.filter(sym =>
					sym.kind === SymbolKind.Class
				).map(sym => sym.name)
			};
		});
	});

	client.start();
}

const getAllSymbols = new RequestType<GetAllSymbolsParams, Symbols, void>("polar-analyzer/getAllSymbols");

interface GetAllSymbolsParams {
	names?: string[]
}

interface Symbols {
	classes: string[]
}



export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
