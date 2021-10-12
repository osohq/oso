import {
  createConnection,
  DiagnosticSeverity,
  ProposedFeatures,
  TextDocuments,
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

  type Severity = 'error' | 'warning';

  interface Position {
    line: number;
    character: number;
  }

  interface Range {
    start: Position;
    end: Position;
  }

  const polarDiagnosticsForFiles = (await oso.loadStrings(
    sourceStrings
  )) as unknown as Map<
    string,
    { message: string; href?: string; severity: Severity; range: Range }[]
  >;

  function polarSeverityToDiagnosticSeverity(severity: Severity) {
    switch (severity) {
      case 'error':
        return DiagnosticSeverity.Error;
      case 'warning':
        return DiagnosticSeverity.Warning;
    }
  }

  for (const [file, polarDiagnostics] of polarDiagnosticsForFiles) {
    const { uri, version } = files.get(file);
    const diagnostics = polarDiagnostics.map(
      ({ message, href, severity: sev, range }) => {
        const codeDescription = href && { href };
        const severity = polarSeverityToDiagnosticSeverity(sev);
        return {
          range,
          severity,
          codeDescription,
          source: 'polar-analyzer',
          message,
        };
      }
    );
    connection.sendDiagnostics({ uri, version, diagnostics });
  }
}

documents.onDidChangeContent(async ({ document }) => {
  files.set(document.uri, document);
  await reloadFiles();
});
documents.listen(connection);

connection.onDidChangeWatchedFiles(({ changes }) => {
  for (const { uri } of changes) {
    files.delete(uri);
  }
  reloadFiles(); // eslint-disable-line @typescript-eslint/no-floating-promises
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
