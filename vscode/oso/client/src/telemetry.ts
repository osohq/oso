import { inspect } from 'util';

import { ExtensionContext, env, Uri, OutputChannel } from 'vscode';
import { hash } from 'blake3-wasm';
import * as Mixpanel from 'mixpanel';
import { Diagnostic } from 'vscode-languageclient';

export const telemetryEventsKey = 'events';
// Flush telemetry events in batches every hour.
export const TELEMETRY_INTERVAL = 1000 * 60 * 60;

// `distinct_id`: One-way hash of VSCode machine ID.
const distinct_id = hash(env.machineId).toString('base64');

const MIXPANEL_PROJECT_TOKEN = 'd14a9580b894059dffd19437b7ddd7be';
const mixpanel = Mixpanel.init(MIXPANEL_PROJECT_TOKEN, {
  protocol: 'https',
  debug: true,
  test: true,
  verbose: true,
});

type DiagnosticsPayload = {
  event: 'diagnostic';
  properties: {
    code: Diagnostic['code'];
  };
};
type TelemetryPayload = DiagnosticsPayload;
// `workspaceId`: One-way hash of workspace folder URI.
type TelemetryMetadata = {
  distinct_id: string;
  load_id: string;
  workspace_id: string;
};
type TelemetryEvent = { properties: TelemetryMetadata } & TelemetryPayload;

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

        if (events.length === 0) return;

        // Clear events queue.
        outputChannel.appendLine(`Flushing ${events.length.toString()} events`);
        await state.update(telemetryEventsKey, []);

        mixpanel.track_batch(events, errors =>
          errors.forEach(({ name, message, stack }) => {
            outputChannel.appendLine(`Mixpanel track_batch error: ${name}`);
            outputChannel.appendLine(`\t${message}`);
            if (stack) outputChannel.appendLine(`\t${stack}`);
          })
        );
      } catch (e) {
        outputChannel.append('Caught error while sending telemetry: ');
        outputChannel.appendLine(e);
      }
    })();
  };
}

export function recordDiagnostics(
  state: State,
  outputChannel: OutputChannel,
  uri: Uri,
  diagnostics: Diagnostic[]
): void {
  void (async () => {
    try {
      const workspace_id = hash(uri.toString()).toString('base64');

      // TODO(gj): race condition?
      const oldEvents = state.get<TelemetryEvent[]>(telemetryEventsKey, []);
      const newEvents: TelemetryEvent[] = diagnostics.map(({ code, data }) => ({
        event: 'diagnostic',
        properties: {
          code,
          distinct_id,
          load_id: (data as { load_id: string }).load_id,
          workspace_id,
        },
      }));
      outputChannel.appendLine(`Recording new events: ${inspect(newEvents)}`);
      const combinedEvents = [...oldEvents, ...newEvents];
      await state.update(telemetryEventsKey, combinedEvents);
    } catch (e) {
      outputChannel.append('Caught error while recording telemetry: ');
      outputChannel.appendLine(e);
    }
  })();
}
