/* eslint-disable @typescript-eslint/restrict-template-expressions */

import { join } from 'path';

import { debounce } from 'lodash';
import {
  ExtensionContext,
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

import {
  counters,
  recordEvent,
  seedState,
  sendTelemetryEvents,
  TelemetryCounters,
  TelemetryRecorder,
  TELEMETRY_STATE_KEY,
  TELEMETRY_INTERVAL,
} from './telemetry';

// TODO(gj): think about what it would take to support `load_str()` via
// https://code.visualstudio.com/api/language-extensions/embedded-languages

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

const extensionName = 'Polar Language Server';
const outputChannel = window.createOutputChannel(extensionName);

// One client per workspace folder.
//
// TODO(gj): handle 'Untitled' docs like this example?
// https://github.com/microsoft/vscode-extension-samples/blob/355d5851a8e87301cf814a3d20f3918cb162ff73/lsp-multi-server-sample/client/src/extension.ts#L62-L79
const clients: Map<string, [LanguageClient, TelemetryRecorder]> = new Map();

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

// Trigger a [`didOpen`][didOpen] event for every not-already-open Polar file
// in `folder` as a somewhat hacky means of transmitting the initial file
// system state to the server. Alternatives: we could (A) do a file system walk
// on the server or (B) `workspace.findFiles()` in the client and then send the
// list of files to the server for it to load. However, by doing it this way,
// we delegate the responibility for content retrieval to VS Code, which means
// we can piggyback on their capabilities for, e.g., loading non-`file://`
// schemes if we wanted to do that at some point in the future.
//
// Relevant issues:
// - https://github.com/microsoft/vscode/issues/15723
// - https://github.com/microsoft/vscode/issues/33046
//
// [didOpen]: https://code.visualstudio.com/api/references/vscode-api#workspace.onDidOpenTextDocument
async function openPolarFilesInWorkspaceFolder(folder: WorkspaceFolder) {
  const pattern = polarFilesInWorkspaceFolderPattern(folder);
  const uris = await workspace.findFiles(pattern);
  return Promise.all(uris.map(openDocument));
}

// Trigger a [`didOpen`][didOpen] event if `uri` is not already open.
//
// The server uses `didOpen` events to maintain its internal view of Polar
// documents in the current workspace folder.
//
// [didOpen]: https://code.visualstudio.com/api/references/vscode-api#workspace.onDidOpenTextDocument
async function openDocument(uri: Uri) {
  const uriMatch = (d: TextDocument) => d.uri.toString() === uri.toString();
  const doc = workspace.textDocuments.find(uriMatch);
  if (doc === undefined) await workspace.openTextDocument(uri);
}

async function startClient(folder: WorkspaceFolder, context: ExtensionContext) {
  const server = context.asAbsolutePath(join('out', 'server.js'));

  // Watch `FileChangeType.Deleted` events for Polar files in the current
  // workspace, including those not open in any editor in the workspace.
  //
  // NOTE(gj): Due to a current limitation in VS Code, when a parent directory
  // is deleted, VS Code's file watcher does not emit events for matching files
  // nested inside that parent. For more information, see this GitHub issue:
  // https://github.com/microsoft/vscode/issues/60813. If that behavior is
  // fixed in the future, we should be able to remove the
  // `workspace.fileOperations.didDelete` handler on the server and go back to
  // a single watcher for files matching '**/*.polar'.
  const deleteWatcher = workspace.createFileSystemWatcher(
    polarFilesInWorkspaceFolderPattern(folder),
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

  const recordTelemetry = debounce(
    event => recordEvent(folder.uri, event),
    1_000
  );
  context.subscriptions.push(client.onTelemetry(recordTelemetry));

  // Start client and mark it for cleanup when the extension is deactivated.
  context.subscriptions.push(client.start());

  // When a Polar document in `folder` (even documents not currently open in VS
  // Code) is created or changed, trigger a [`didOpen`][didOpen] event if the
  // document is not already open. This will transmit the current state of the
  // newly created or changed document to the Language Server, and subsequent
  // changes to it will be relayed via [`didChange`][didChange] events.
  //
  // [didOpen]: https://code.visualstudio.com/api/references/vscode-api#workspace.onDidOpenTextDocument
  // [didChange]: https://code.visualstudio.com/api/references/vscode-api#workspace.onDidChangeTextDocument
  context.subscriptions.push(createChangeWatcher.onDidCreate(openDocument));
  context.subscriptions.push(createChangeWatcher.onDidChange(openDocument));

  // Transmit the initial file system state for `folder` (including files not
  // currently open in VS Code) to the server.
  await openPolarFilesInWorkspaceFolder(folder);

  clients.set(folder.uri.toString(), [client, recordTelemetry]);
}

async function stopClient(folder: string) {
  const exists = clients.get(folder);
  if (exists) {
    const [client, recordTelemetry] = exists;
    // Try flushing latest event in case one's in the chamber.
    recordTelemetry.flush();
    await client.stop();
  }
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

// Create function in global context so we have access to it in `deactivate()`.
// See corresponding comment in `activate()` where we update the stored
// function.
let persistState: (state: TelemetryCounters) => Promise<void> = async () => {}; // eslint-disable-line @typescript-eslint/no-empty-function

export async function activate(context: ExtensionContext): Promise<void> {
  // Seed extension-local state from persisted VS Code memento-backed state.
  seedState(context.globalState.get<TelemetryCounters>(TELEMETRY_STATE_KEY));

  // Capturing `context.globalState` in this closure since we won't have access
  // to it in deactivate(), where we want to persist the updated state.
  persistState = async (state: TelemetryCounters) =>
    context.globalState.update(TELEMETRY_STATE_KEY, state);

  const folders = workspace.workspaceFolders || [];

  // Send telemetry events every `TELEMETRY_INTERVAL` ms. We don't `await` the
  // `sendTelemetryEvents` promise because nothing depends on its outcome.
  const interval = setInterval(
    () => sendTelemetryEvents(outputChannel), // eslint-disable-line @typescript-eslint/no-misused-promises
    TELEMETRY_INTERVAL
  );
  // Clear interval when extension is deactivated.
  context.subscriptions.push({ dispose: () => clearInterval(interval) });

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

export async function deactivate(): Promise<void> {
  await Promise.all(
    [...clients.values()].map(([client, recordTelemetry]) => {
      // Try flushing latest event in case one's in the chamber.
      recordTelemetry.flush();
      return client.stop();
    })
  );

  // Flush telemetry queue on shutdown.
  await sendTelemetryEvents(outputChannel);

  // Persist monthly/daily counter/timestamp state.
  return persistState(counters);
}
