import {
  createConnection,
  TextDocuments,
  ProposedFeatures,
  // TextDocumentSyncKind,
} from 'vscode-languageserver/node';
import { TextDocument } from 'vscode-languageserver-textdocument';
import { Oso } from 'oso';

// Create LSP connection
const connection = createConnection(ProposedFeatures.all);

// Create manager for open text documents
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

const files: Map<string, TextDocument> = new Map();

const oso = new Oso();

async function reloadFiles() {
  oso.clearRules();
  const sourceStrings = [...files.entries()].map(([filename, document]) => ({
    filename,
    contents: document.getText(),
  }));
  await oso.loadStrings(sourceStrings);
}

documents.onDidChangeContent(async ({ document }) => {
  files.set(document.uri, document);
  await reloadFiles();
});
documents.listen(connection);

// eslint-disable-next-line @typescript-eslint/no-misused-promises
connection.onDidChangeWatchedFiles(async ({ changes }) => {
  for (const { uri } of changes) {
    files.delete(uri);
  }
  await reloadFiles();
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
