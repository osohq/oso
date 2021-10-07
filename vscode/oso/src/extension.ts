import {
  ExtensionContext,
  StatusBarAlignment,
  window,
  StatusBarItem,
  workspace,
  TextEditor,
} from 'vscode';

// TODO(gj): think about what it would take to support `load_str()` via
// https://code.visualstudio.com/api/language-extensions/embedded-languages

// TODO(gj): do we need to maintain state for all (potentially dirty) Polar
// files in the current workspace?

function updateStatus(status: StatusBarItem, filenames: string[]) {
  status.text = `${workspace.name || '(none)'} -> [${filenames
    .map(f => f.split('/').pop())
    .join(', ')}]`;
}

// TODO(gj): maybe just punt on non-workspace use cases entirely for now? At
// least until progress is made on
// https://github.com/Microsoft/vscode/issues/15178 so we have a less hacky way
// to list all open editors (instead of just the currently visible ones).
function updateStatusFromVisibleEditors(
  status: StatusBarItem,
  es: TextEditor[]
) {
  const polarEditors = es.filter(e => e.document.languageId === 'polar');
  const filenames = polarEditors.map(e => e.document.fileName);
  updateStatus(status, filenames);
}

// TODO(gj): what about when a workspace is open and you open a new file/editor
// (via command-N) that may or may not ultimately be saved in a folder that
// exists in the current workspace?
//
// NOTE(gj): punting on the above at least until progress is made on
// https://github.com/Microsoft/vscode/issues/15178 so we have a less hacky way
// to list all open editors (instead of just the currently visible ones).
async function updateStatusFromWorkspace(status: StatusBarItem): Promise<void> {
  const polarFiles = await workspace.findFiles('**/*.polar');
  console.log('polarFiles', polarFiles);
  const filenames = polarFiles.map(f => f.fsPath);
  updateStatus(status, filenames);
}

export async function activate(context: ExtensionContext): Promise<void> {
  const status = window.createStatusBarItem(StatusBarAlignment.Left, 1000000);
  context.subscriptions.push(status);

  // TODO(gj): is it possible to go from workspace -> no workspace? What about
  // from no workspace -> workspace?

  if (workspace.name === undefined) {
    // Not in a workspace.

    // NOTE(gj): Maybe think about linking all open editors together once
    // progress is made on https://github.com/Microsoft/vscode/issues/15178,
    // but at present I think it'd be too hacky for too little benefit.

    context.subscriptions.push(
      window.onDidChangeVisibleTextEditors(es => {
        console.log('onDidChangeVisibleTextEditors');
        updateStatusFromVisibleEditors(status, es);
      })
    );

    // Initialize with current visible editors.
    updateStatusFromVisibleEditors(status, window.visibleTextEditors);
  } else {
    // In a workspace.

    // TODO(gj): need to track files separately per workspace folder; currently
    // tracking all files across the entire workspace. Use case is opening
    // osohq/gitclub in one workspace folder and osohq/oso in another.

    // TODO(gj): what happens when someone opens the osohq/oso folder and there
    // are a ton of different Polar files in different subfolders that should
    // *not* be considered part of the same policy?

    const watcher = workspace.createFileSystemWatcher('**/*.polar');
    context.subscriptions.push(watcher);
    context.subscriptions.push(
      watcher.onDidChange(async e => {
        console.log('onDidChange');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );
    context.subscriptions.push(
      watcher.onDidCreate(async e => {
        console.log('onDidCreate');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );
    context.subscriptions.push(
      watcher.onDidDelete(async e => {
        console.log('onDidDelete');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidChangeTextDocument(async e => {
        console.log('onDidChangeTextDocument');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidChangeWorkspaceFolders(async e => {
        console.log('onDidChangeWorkspaceFolders');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidCreateFiles(async e => {
        console.log('onDidCreateFiles');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidDeleteFiles(async e => {
        console.log('onDidDeleteFiles');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidRenameFiles(async e => {
        console.log('onDidRenameFiles');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    context.subscriptions.push(
      workspace.onDidSaveTextDocument(async e => {
        console.log('onDidSaveTextDocument');
        console.dir(e);
        await updateStatusFromWorkspace(status);
      })
    );

    await updateStatusFromWorkspace(status);
  }
  status.show();
}
