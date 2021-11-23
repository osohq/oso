import { ExtensionContext, env, Diagnostic, Uri, OutputChannel } from 'vscode';
import { hash } from 'blake3-wasm';

export const telemetryEventsKey = 'events';
const userId = hash(env.machineId).toString('base64');

type InitPayload = 'Init';
type PolicyPayload = { hasResourceBlocks: boolean };
type DiagnosticPayload = { code: Diagnostic['code'] };
type TelemetryPayload = InitPayload | PolicyPayload | DiagnosticPayload;

interface TelemetryEvent {
  // One-way hash of VSCode machine ID.
  userId: string;
  // One-way hash of workspace folder URI.
  workspaceId: string;
  payload: TelemetryPayload;
}

type State = ExtensionContext['globalState'];
type Recorder = (payload: TelemetryPayload) => void;

export function createTelemetryRecorder(
  state: State,
  uri: Uri,
  outputChannel: OutputChannel
): Recorder {
  const workspaceId = hash(uri.toString()).toString('base64');

  return (payload: TelemetryPayload) =>
    void (async () => {
      // TODO(gj): race condition?
      const events = state.get<TelemetryEvent[]>(telemetryEventsKey, []);
      const newEvent = { userId, workspaceId, payload };
      try {
        await state.update(telemetryEventsKey, [...events, newEvent]);
      } catch (e) {
        outputChannel.append('Caught error while updating telemetry state: ');
        outputChannel.appendLine(e);
      }
    })();
}
