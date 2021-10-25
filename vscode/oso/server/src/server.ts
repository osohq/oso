import {
  createConnection,
  ProposedFeatures,
  PublishDiagnosticsParams,
  TextDocumentSyncKind,
} from 'vscode-languageserver/node';
import { PolarLanguageServer } from '../out/polar_language_server';

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

const sendDiagnosticsCallback = (params: PublishDiagnosticsParams) =>
  connection.sendDiagnostics(params);
const pls = new PolarLanguageServer(sendDiagnosticsCallback);

connection.onNotification((...args) => pls.onNotification(...args));

connection.onInitialize(() => {
  return {
    capabilities: {
      textDocumentSync: {
        openClose: true,
        save: true,
        change: TextDocumentSyncKind.Full,
      },
      workspace: { workspaceFolders: { supported: true } },
    },
  };
});

connection.listen();
