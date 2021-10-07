import {
  ExtensionContext,
  StatusBarAlignment,
  window,
  StatusBarItem,
  workspace,
  TextEditor,
} from 'vscode';

function updateStatus(status: StatusBarItem, filenames: string[]) {
  status.text = `[${filenames.map(f => f.split('/').pop()).join(', ')}]`;
}

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
async function updateStatusFromWorkspace(status: StatusBarItem): Promise<void> {
  const polarFiles = await workspace.findFiles('**/*.polar');
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
