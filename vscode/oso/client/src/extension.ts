import { join } from 'path';

import {
  ExtensionContext,
  languages,
  RelativePattern,
  TextDocument,
  Uri,
  window,
  workspace,
  WorkspaceFolder,
  WorkspaceFoldersChangeEvent,
} from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  TransportKind,
} from 'vscode-languageclient/node';

// TODO(gj): think about what it would take to support `load_str()` via
// https://code.visualstudio.com/api/language-extensions/embedded-languages

// TODO(gj): do we need to maintain state for all (potentially dirty) Polar
// docs in the current workspace?

// TODO(gj): maybe just punt on non-workspace use cases entirely for now? At
// least until progress is made on
// https://github.com/Microsoft/vscode/issues/15178 so we have a less hacky way
// to list all open editors (instead of just the currently visible ones).

// TODO(gj): what about when a workspace is open and you open a new doc/editor
// (via command-N) that may or may not ultimately be saved in a folder that
// exists in the current workspace?
//
// NOTE(gj): punting on the above at least until progress is made on
// https://github.com/Microsoft/vscode/issues/15178 so we have a less hacky way
// to list all open editors (instead of just the currently visible ones).

const extensionName = 'polar-analyzer';
const outputChannel = window.createOutputChannel(extensionName);

// One client per workspace folder.
//
// TODO(gj): handle 'Untitled' docs like this example?
// https://github.com/microsoft/vscode-extension-samples/blob/355d5851a8e87301cf814a3d20f3918cb162ff73/lsp-multi-server-sample/client/src/extension.ts#L62-L79
const clients: Map<string, LanguageClient> = new Map();

// TODO(gj): nested workspace folders:
//     folderA/
//       - a.polar
//       - folderB/
//     folderB/
//       - b.polar
// If the user opens folderA & folderB in the same workspace, should we make
// the assumption that folderB's b.polar should be evaluated alongside
// folderA's a.polar? That would mean we'd be surfacing consistent
// errors/warnings in both workspace folders. It'd probably make the most sense
// to start a single server for the outermost folder and fan out messages from
// it to all subfolders to avoid duplicating a lot of work.
//
// This is probably a great argument for an Oso.toml config file marking the
// root of an Oso project. Then we could just walk up the tree until we find an
// Oso.toml and assume that all .polar files underneath it are part of the same
// project.
//
// If the user only opens `folderB`, it seems reasonable to just load b.polar
// and treat that file as a self-contained policy.
//
// If the user only opens `folderA`, then we'll treat `a.polar` & `b.polar` as
// part of the same policy.

function polarFilesInWorkspaceFolderPattern(folder: WorkspaceFolder) {
  return new RelativePattern(folder, '**/*.polar');
}

// Trigger a `didOpen` event for every Polar file in `folder` as a somewhat
// hacky means of transmitting the initial file system state to the server.
// Alternatives: we could (A) do a file system walk on the server or (B)
// `workspace.findFiles()` in the client and then send the list of files to the
// server for it to load. However, by doing it this way, we delegate the
// responibility for content retrieval to VSCode, which means we can piggyback
// on their capabilities for, e.g., loading non-`file://` schemes if we wanted
// to do that at some point in the future.
//
// Relevant issues:
// - https://github.com/microsoft/vscode/issues/15723
// - https://github.com/microsoft/vscode/issues/33046
async function openPolarFilesInWorkspaceFolder(folder: WorkspaceFolder) {
  const pattern = polarFilesInWorkspaceFolderPattern(folder);
  const uris = await workspace.findFiles(pattern);
  return Promise.all(uris.map(reloadDocument));
}

// Trigger a [`didOpen`][didOpen] event for `uri`.
//
// The server uses `didOpen` events to maintain its internal view of Polar
// documents in the current workspace folder.
//
// [didOpen]: https://code.visualstudio.com/api/references/vscode-api#workspace.onDidOpenTextDocument
async function reloadDocument(uri: Uri) {
  const uriMatch = (d: TextDocument) => d.uri.toString() === uri.toString();
  let doc = workspace.textDocuments.find(uriMatch);

  if (doc) {
    // If `doc` is already open, cycle its `languageId` from `polar` ->
    // `plaintext` -> `polar` to trigger a `didOpen` event. This is the event
    // the server listens to in order to update the state of all Polar
    // documents it's aware of.
    //
    // The [`setTextDocumentLanguage`][setTextDocumentLanguage] API triggers a
    // `didOpen` event but seems to only fire when the document's `languageId`
    // actually changes. Calling `setTextDocumentLanguage(doc, 'polar')`
    // doesn't trigger the event if `doc`'s `languageId` is already `'polar'`.
    //
    // [setTextDocumentLanguage]: https://code.visualstudio.com/api/references/vscode-api#languages.setTextDocumentLanguage
    doc = await languages.setTextDocumentLanguage(doc, 'plaintext');
    await languages.setTextDocumentLanguage(doc, 'polar');
  } else {
    // If `doc` wasn't already open, open it, which will trigger the `didOpen`
    // event without the above `setTextDocumentLanguage` shenanigans.
    await workspace.openTextDocument(uri);
  }
}

async function startClient(folder: WorkspaceFolder, context: ExtensionContext) {
  const server = context.asAbsolutePath(join('server', 'out', 'server.js'));

  // Watch `FileChangeType.Deleted` events for all files and directories in the
  // current workspace, including those not open in any editor in the
  // workspace.
  //
  // NOTE(gj): Due to a current limitation in VSCode, when a parent directory
  // is deleted, VSCode's file watcher does not emit events for matching files
  // nested inside that parent. For more information, see this GitHub issue:
  // https://github.com/microsoft/vscode/issues/60813. If that behavior is
  // fixed in the future, we should be able to go back to a single watcher for
  // files matching '**/*.polar'.
  //
  // NOTE(gj): watching _every_ file might be an issue if the user deletes a
  // ton of irrelevant files at once (e.g., `rm -rf build_directory/`). There
  // doesn't seem to be a good option for this right now, but VSCode is
  // migrating to a new watcher that may ameliorate some of these issues:
  // https://github.com/microsoft/vscode/issues/132483. FWIW, the current
  // watcher seems to respect the `files.watcherExclude` config property, which
  // by default "excludes `node_modules` and some folders under `.git`":
  // https://github.com/microsoft/vscode-docs/blob/04cec7670671ae852ef020991fb8441ee0bb2796/docs/setup/linux.md?plain=1#L239
  const deleteWatcher = workspace.createFileSystemWatcher(
    new RelativePattern(folder, '**'),
    true,
    true
  );
  // Watch `FileChangeType.Created` and `FileChangeType.Changed` events for
  // files in the current workspace, including those not open in any editor in
  // the workspace.
  const createChangeWatcher = workspace.createFileSystemWatcher(
    polarFilesInWorkspaceFolderPattern(folder),
    false,
    false,
    true
  );

  // Clean up watchers when extension is deactivated.
  context.subscriptions.push(deleteWatcher);
  context.subscriptions.push(createChangeWatcher);

  const debugOpts = {
    execArgv: ['--nolazy', `--inspect=${6011 + clients.size}`],
  };
  const serverOpts = {
    run: { module: server, transport: TransportKind.ipc },
    debug: { module: server, transport: TransportKind.ipc, options: debugOpts },
  };
  const clientOpts: LanguageClientOptions = {
    // TODO(gj): seems like I should be able to use a RelativePattern here, but
    // the TS type for DocumentFilter.pattern doesn't seem to like that.
    documentSelector: [
      { language: 'polar', pattern: `${folder.uri.fsPath}/**/*.polar` },
    ],
    synchronize: { fileEvents: deleteWatcher },
    diagnosticCollectionName: extensionName,
    workspaceFolder: folder,
    outputChannel,
  };
  const client = new LanguageClient(extensionName, serverOpts, clientOpts);

  // Start client and mark it for cleanup when the extension is deactivated.
  context.subscriptions.push(client.start());

  // When a Polar document in `folder` (even documents not currently open in
  // VSCode) is created or changed, trigger a `workspace.onDidOpenTextDocument`
  // event that the language server is listening for.
  context.subscriptions.push(createChangeWatcher.onDidCreate(reloadDocument));
  context.subscriptions.push(createChangeWatcher.onDidChange(reloadDocument));

  // Transmit the initial file system state for `folder` (including files not
  // currently open in VSCode) to the server.
  await openPolarFilesInWorkspaceFolder(folder);

  clients.set(folder.uri.toString(), client);
}

async function stopClient(folder: string) {
  const client = clients.get(folder);
  await client.stop();
  clients.delete(folder);
}

function updateClients(context: ExtensionContext) {
  return async function ({ added, removed }: WorkspaceFoldersChangeEvent) {
    // Clean up clients for removed folders.
    for (const folder of removed) await stopClient(folder.uri.toString());

    // Create clients for added folders.
    for (const folder of added) await startClient(folder, context);
  };
}

export async function activate(context: ExtensionContext): Promise<void> {
  const folders = workspace.workspaceFolders || [];

  // Start a client for every folder in the workspace.
  for (const folder of folders) await startClient(folder, context);

  // Update clients when workspace folders change.
  workspace.onDidChangeWorkspaceFolders(updateClients(context));

  // TODO(gj): is it possible to go from workspace -> no workspace? What about
  // from no workspace -> workspace?

  // TODO(gj): think about whether to handle the case where there isn't a
  // workspace but is an open text editor (or potentially multiple) set to
  // `polar` syntax. Maybe think about linking all open editors together once
  // progress is made on https://github.com/Microsoft/vscode/issues/15178, but
  // at present I think it'd be too hacky for too little benefit.

  // TODO(gj): what happens when someone opens the osohq/oso folder and there
  // are a ton of different Polar files in different subfolders that should
  // *not* be considered part of the same policy?
}

export function deactivate(): Promise<void[]> {
  return Promise.all([...clients.values()].map(c => c.stop()));
}
