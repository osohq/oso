import { ExtensionContext, env, Diagnostic, Uri, OutputChannel } from 'vscode';
import { hash } from 'blake3-wasm';
import * as Mixpanel from 'mixpanel';

export const telemetryEventsKey = 'events';
// Flush telemetry events in batches every five minutes.
export const TELEMETRY_INTERVAL = 300000;

// `distinct_id`: One-way hash of VSCode machine ID.
const distinct_id = hash(env.machineId).toString('base64');

const MIXPANEL_PROJECT_TOKEN = 'd14a9580b894059dffd19437b7ddd7be';
const mixpanel = Mixpanel.init(MIXPANEL_PROJECT_TOKEN, {
  protocol: 'https',
  debug: true,
  test: true,
  verbose: true,
});

type HasResourceBlocksPayload = {
  key: 'has_resource_blocks';
  values: [boolean];
};
type DiagnosticPayload = {
  key: 'diagnostic';
  values: Diagnostic['code'][];
};
type TelemetryPayload = DiagnosticPayload | HasResourceBlocksPayload;
// `workspaceId`: One-way hash of workspace folder URI.
type TelemetryEvent = { workspaceId: string } & TelemetryPayload;

type State = ExtensionContext['globalState'];

export function sendQueuedEvents(
  state: State,
  outputChannel: OutputChannel
): () => void {
  return () => {
    void (async () => {
      try {
        // Retrieve all queued events.
        const events = state.get<TelemetryEvent[]>(telemetryEventsKey, []);

        outputChannel.appendLine(`Queue length: ${events.length.toString()}`);

        if (events.length === 0) return;

        // Clear events queue.
        outputChannel.appendLine('Flushing...');
        await state.update(telemetryEventsKey, []);

        mixpanel.track_batch(
          events.flatMap(({ key, values, workspaceId: workspace_id }) => {
            // Generate `group_id` to track events encountered en masse.
            const group_id = hash(Math.random().toString()).toString('base64');
            return values.map((value: TelemetryPayload['values'][number]) => ({
              event: key,
              properties: { distinct_id, group_id, [key]: value, workspace_id },
            }));
          }),
          errors =>
            errors.forEach(({ name, message }) =>
              outputChannel.appendLine(
                `Mixpanel track_batch error: ${name}\n\t${message}`
              )
            )
        );
      } catch (e) {
        outputChannel.append('Caught error while sending telemetry: ');
        outputChannel.appendLine(e);
      }
    })();
  };
}

type Recorder = (uri: Uri, payload: TelemetryPayload) => void;

export function createTelemetryRecorder(
  state: State,
  outputChannel: OutputChannel
): Recorder {
  return (uri: Uri, { key, values }: TelemetryPayload) =>
    void (async () => {
      try {
        const workspaceId = hash(uri.toString()).toString('base64');

        // TODO(gj): race condition?
        const events = state.get<TelemetryEvent[]>(telemetryEventsKey, []);
        const newEvent = { key, values, workspaceId };
        await state.update(telemetryEventsKey, [...events, newEvent]);
      } catch (e) {
        outputChannel.append('Caught error while recording telemetry: ');
        outputChannel.appendLine(e);
      }
    })();
}
