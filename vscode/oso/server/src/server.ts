import {
  createConnection,
  ProposedFeatures,
  PublishDiagnosticsParams,
  TextDocumentSyncKind,
} from 'vscode-languageserver/node';
import { PolarLanguageServer } from '../out/polar_language_server';

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

const pls = new PolarLanguageServer((params: PublishDiagnosticsParams) =>
  connection.sendDiagnostics(params)
);

// eslint-disable-next-line @typescript-eslint/no-unused-vars
connection.onRequest((method, params, _token) => {
  console.log('[TS onRequest]:', method, params);
  pls.onRequest(method, params);
});

connection.onNotification((method, params) =>
  pls.onNotification(method, params)
);

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
