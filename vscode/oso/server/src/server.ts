import { inspect } from 'util';

import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  // TextDocumentSyncKind,
} from 'vscode-languageserver/node';

import 'vscode-languageserver';

import { TextDocument } from 'vscode-languageserver-textdocument';

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

// Create manager for open text documents
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

let folderLen: number;
let folder: string | null;

documents.onDidChangeContent(({ document }) => {
  const file = document.uri.slice(folderLen);
  const charCount = document.getText().length;
  connection.console.log(
    `[${process.pid} ${folder}] onDidChangeContent => ${file} [${charCount}]`
  );
});

documents.listen(connection);

connection.onDidChangeWatchedFiles(({ changes }) => {
  connection.console.log(
    `[${process.pid} ${folder}] onDidChangeWatchedFiles => ${inspect(changes)}`
  );
});

connection.onInitialize(params => {
  folderLen = params.rootUri.length + 1;
  folder = params.rootUri.split('/').pop();
  connection.console.log(`[${process.pid} ${folder}] onInitialize`);
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
