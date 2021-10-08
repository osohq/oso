import { join } from 'path';

import {
  ExtensionContext,
  RelativePattern,
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
// files in the current workspace?

// TODO(gj): maybe just punt on non-workspace use cases entirely for now? At
// least until progress is made on
// https://github.com/Microsoft/vscode/issues/15178 so we have a less hacky way
// to list all open editors (instead of just the currently visible ones).

// TODO(gj): what about when a workspace is open and you open a new file/editor
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
// TODO(gj): handle 'Untitled' files like this example?
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

// Trigger an 'open' event for every Polar file in `folder` as a somewhat hacky
// means of transmitting the initial file system state to the server.
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
  const files = await workspace.findFiles(pattern);
  return Promise.all(files.map(f => workspace.openTextDocument(f)));
}

async function startClient(folder: WorkspaceFolder, context: ExtensionContext) {
  const server = context.asAbsolutePath(join('server', 'out', 'server.js'));

  const pattern = polarFilesInWorkspaceFolderPattern(folder);
  // Watch `FileChangeType.Deleted` events for files in the current workspace,
  // including those not open in any editor in the workspace.
  const deleteWatcher = workspace.createFileSystemWatcher(pattern, true, true);
  // Watch `FileChangeType.Created` and `FileChangeType.Changed` events for
  // files in the current workspace, including those not open in any editor in
  // the workspace.
  const createChangeWatcher = workspace.createFileSystemWatcher(
    pattern,
    false,
    false,
    true
  );

  // Clean up watchers when extension is deactivated.
  context.subscriptions.push(deleteWatcher);
  context.subscriptions.push(createChangeWatcher);

  // TODO(gj): remove debugOpts when we move server from TS -> Rust.
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
    documentSelector: [{ pattern: `${folder.uri.fsPath}/**/*.polar` }],
    synchronize: { fileEvents: deleteWatcher },
    diagnosticCollectionName: extensionName,
    workspaceFolder: folder,
    outputChannel,
  };
  const client = new LanguageClient(extensionName, serverOpts, clientOpts);

  // Start client and mark it for cleanup when the extension is deactivated.
  context.subscriptions.push(client.start());

  // When a Polar file in `folder` (even files not currently open in VSCode) is
  // created or changed, trigger a `workspace.onDidOpenTextDocument` event that
  // the language server is listening for.
  context.subscriptions.push(
    createChangeWatcher.onDidCreate(file => workspace.openTextDocument(file))
  );
  context.subscriptions.push(
    createChangeWatcher.onDidChange(file => workspace.openTextDocument(file))
  );

  // Trigger a `workspace.onDidOpenTextDocument` event for every Polar file in
  // `folder` (even files not currently open in VSCode).
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

  // TODO(gj): need to track files separately per workspace folder; currently
  // tracking all files across the entire workspace. Use case is opening
  // osohq/gitclub in one workspace folder and osohq/oso in another.

  // TODO(gj): what happens when someone opens the osohq/oso folder and there
  // are a ton of different Polar files in different subfolders that should
  // *not* be considered part of the same policy?
}

export function deactivate(): Promise<void[]> {
  return Promise.all([...clients.values()].map(c => c.stop()));
}
