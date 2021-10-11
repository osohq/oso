import { inspect } from 'util';

import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  // TextDocumentSyncKind,
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

// Create manager for open text documents
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

const files: Map<string, TextDocument> = new Map();

documents.onDidChangeContent(({ document }) => {
  files.set(document.uri, document);
  console.log('Files loaded:', files.size);
});

documents.listen(connection);

connection.onDidChangeWatchedFiles(({ changes }) => {
  for (const { uri } of changes) {
    files.delete(uri);
  }
  console.log('Files loaded:', files.size);
});

connection.onInitialize(() => {
  // TODO(gj): what does returning `capabilities.workspace.fileOperations` do?
  //
  // TODO(gj): everything seems to work fine even when I return no
  // capabilities?
  return {
    capabilities: {
      //   // textDocumentSync: {
      //   //   openClose: true,
      //   //   save: true,
      //   //   change: TextDocumentSyncKind.Full,
      //   // },
      //   workspace: { workspaceFolders: { supported: true } },
    },
  };
});
connection.listen();
