import * as path from 'path';
import { extensions, languages, window, workspace, ExtensionContext, commands, SymbolInformation, SymbolKind } from 'vscode';

import * as net from 'net';

import {
	integer,
	LanguageClient,
	LanguageClientOptions,
	RequestType,
	ServerOptions,
	StreamInfo,
} from 'vscode-languageclient/node';

import {
	SymbolInformation as LspSymbolInformation
} from 'vscode-languageserver-types';
import { createConverter } from 'vscode-languageclient/lib/common/codeConverter';

const converter = createConverter();

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
				path.join('client', 'out', 'polar-analyzer')
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
			// const langs = await languages.getLanguages();
			info(`All extensions: ${extensions.all.map(ext => ext.id)}`);
			info(`All langs: ${await languages.getLanguages()}`);
			info(`Activating extensions for symbols`);
			const langExtensions = [
				extensions.getExtension("ms-python.python"),
				extensions.getExtension("ms-python.vscode-pylance"),
			];
			const allActivated = langExtensions.every(ext => ext.isActive);
			if (!allActivated) {
				for (const ext of langExtensions) {
					await ext.activate();
				}

				// sleep for 2s to let the extensions load
				await new Promise(resolve => setTimeout(resolve, 2000));
			}

			info(`Requested symbols: ${params.names} `);
			const symbols: LspSymbolInformation[] = [];
			for (const symbol of params.names) {
				const matches: SymbolInformation[] = await commands.executeCommand('vscode.executeWorkspaceSymbolProvider', symbol);
				info(`Matches for ${symbol}: ${matches.map(m => m.name)} `);
				symbols.push(...convertSymbols(matches));
			}
			const global: SymbolInformation[] = await commands.executeCommand('vscode.executeWorkspaceSymbolProvider', "");
			info(`Global symbols: ${global.map(m => m.name)} `);
			symbols.push(...convertSymbols(global));
			info(`Returned symbols: ${symbols.map(s => s.toString()).join(",")} `);
			return {
				symbols
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
	symbols: LspSymbolInformation[]
}

function convertSymbols(symbols: SymbolInformation[]): LspSymbolInformation[] {
	return symbols.map(s => {
		return {
			name: s.name,
			containerName: s.containerName,
			location: converter.asLocation(s.location),
			kind: converter.asSymbolKind(s.kind),
			tags: s.tags === undefined ? undefined : converter.asSymbolTags(s.tags),
		};
	});
}

export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}
