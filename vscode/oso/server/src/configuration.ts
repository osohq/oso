/* Initialization and configuration routines */

import { InitializeParams, InitializeResult, TextDocumentSyncKind, DidChangeConfigurationNotification, CodeLensResolveRequest, ConfigurationItem, DidChangeConfigurationParams, DidCloseTextDocumentParams, TextDocumentChangeEvent, Connection } from 'vscode-languageserver';


interface Settings {
	osoVersion: string;
}

// The global settings, used when the `workspace/configuration` request is not supported by the client.
// Please note that this is not the case when using this server with the client provided
// but could happen with other clients.
const defaultSettings: Settings = { osoVersion: "0.14" };
let globalSettings: Settings = defaultSettings;

// Client supports workspace/configuration
let hasConfigurationCapability = false;

// 
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;

const getCapabilities = (params: InitializeParams) => {
	// The capabilities provided by the client (editor or tool)
	const capabilities = params.capabilities;
	const workspaceFolders = params.workspaceFolders;
	if (workspaceFolders !== undefined) {
		console.log(`Workspace folders configured: ${workspaceFolders}`);
	}

	// Does the client support the `workspace/configuration` request?
	// If not, we fall back using global settings.
	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	);
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	);
	hasDiagnosticRelatedInformationCapability = !!(
		capabilities.textDocument &&
		capabilities.textDocument.publishDiagnostics &&
		capabilities.textDocument.publishDiagnostics.relatedInformation
	);

	const result: InitializeResult = {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Full,
			// Tell the client that this server supports code completion.
			completionProvider: {
				resolveProvider: true
			},
			documentSymbolProvider: true,
			hoverProvider: true,
			workspace: {
				fileOperations: {
					// follow delete and rename operations
					didDelete: {
						filters: [
							{
								pattern: {
									glob: "**​/*.polar"
								}
							}
						]
					},
					didRename: {
						filters: [
							{
								pattern: {
									glob: "**​/*.polar"
								}
							}
						]
					},
				}
			}
		}
	};
	if (hasWorkspaceFolderCapability) {
		result.capabilities.workspace = {
			workspaceFolders: {
				supported: true
			}
		};
	}
	return result;
};

const updateSettings = (change: DidChangeConfigurationParams) => {
	if (hasConfigurationCapability) {
		// do nothing?
	} else {
		globalSettings = <Settings>(
			(change.settings.osoLsp || defaultSettings)
		);
	}
};

function registerCapabilities(connection: Connection): void {
	connection.onInitialized(() => {
		if (hasConfigurationCapability) {
			// Register for all configuration changes.
			connection.client.register(DidChangeConfigurationNotification.type, undefined);
		}
		if (hasWorkspaceFolderCapability) {
			connection.workspace.onDidChangeWorkspaceFolders(_event => {
				connection.console.log('Workspace folder change event received.');
			});
		}
	});
}


export {
	getCapabilities,
	registerCapabilities,
	updateSettings
};