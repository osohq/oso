import { createHash } from 'crypto';

import { env, ExtensionContext, OutputChannel, Uri, workspace } from 'vscode';
import * as Mixpanel from 'mixpanel';
import {
  Diagnostic,
  DiagnosticSeverity as Severity,
} from 'vscode-languageclient';

export const telemetryEventsKey = 'events';
// Flush telemetry events in batches every hour.
export const TELEMETRY_INTERVAL = 1000 * 60 * 60;

const hash = (contents: string) =>
  createHash('sha256').update(contents).digest('base64');

// One-way hash of VSCode machine ID.
const distinct_id = hash(env.machineId);

const MIXPANEL_PROJECT_TOKEN = 'd14a9580b894059dffd19437b7ddd7be';
const mixpanel = Mixpanel.init(MIXPANEL_PROJECT_TOKEN, { protocol: 'https' });

function telemetryEnabled() {
  const setting = workspace
    .getConfiguration('oso.polarLanguageServer.telemetry')
    .get<'default' | 'on' | 'off' | undefined>('enabled');

  // Check if user explicitly enabled or disabled telemetry.
  if (setting === 'on') return true;
  if (setting === 'off') return false;

  // Otherwise, default to VSCode's telemetry setting.

  // VSCode >=1.55
  //
  // https://code.visualstudio.com/updates/v1_55#_telemetry-enablement-api
  if (env.isTelemetryEnabled !== undefined) return env.isTelemetryEnabled;

  // VSCode <1.55
  const config = workspace.getConfiguration('telemetry');
  const enabled = config.get<boolean>('enableTelemetry');
  return enabled;
}

type MixpanelLoadEvent = {
  event: 'load';
  properties: {
    diagnostics: number;
    errors: number;
    has_resource_blocks: boolean;
    successful: boolean;
    warnings: number;
  };
};

type MixpanelDiagnosticEvent = {
  event: 'diagnostic';
  properties: {
    code: Diagnostic['code'];
  };
};

type MixpanelMetadata = {
  // One-way hash of VSCode machine ID.
  distinct_id: string;
  // Unique ID for a `diagnostic_load` call. We use this to tie diagnostic
  // events (errors & warnings) to the load event they came from.
  load_id: string;
  // One-way hash of workspace folder URI.
  workspace_id: string;
};

type MixpanelEvent = { properties: MixpanelMetadata } & (
  | MixpanelLoadEvent
  | MixpanelDiagnosticEvent
);

type State = ExtensionContext['globalState'];

export function flushQueue(
  state: State,
  outputChannel: OutputChannel
): () => void {
  return () => {
    if (!telemetryEnabled()) return;

    void (async () => {
      try {
        // Retrieve all queued events.
        const events = state.get<MixpanelEvent[]>(telemetryEventsKey, []);

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

export type TelemetryEvent = {
  diagnostics: Diagnostic[];
  has_resource_blocks: boolean;
};

export function enqueueEvent(
  state: State,
  log: OutputChannel,
  uri: Uri,
  { diagnostics, has_resource_blocks }: TelemetryEvent
): void {
  if (!telemetryEnabled()) return;

  void (async () => {
    try {
      const load_id = hash(Math.random().toString());
      const workspace_id = hash(uri.toString());
      const metadata: MixpanelMetadata = { distinct_id, load_id, workspace_id };

      const errors = diagnostics.filter(d => d.severity === Severity.Error);
      const warnings = diagnostics.filter(d => d.severity === Severity.Warning);

      const loadEvent: MixpanelEvent = {
        event: 'load',
        properties: {
          diagnostics: diagnostics.length,
          errors: errors.length,
          has_resource_blocks,
          successful: errors.length === 0,
          warnings: warnings.length,
          ...metadata,
        },
      };

      const diagnosticEvents: MixpanelEvent[] = diagnostics.map(({ code }) => ({
        event: 'diagnostic',
        properties: { code, ...metadata },
      }));

      // TODO(gj): race condition?
      const old = state.get<MixpanelEvent[]>(telemetryEventsKey, []);
      const events: MixpanelEvent[] = [...old, loadEvent, ...diagnosticEvents];
      await state.update(telemetryEventsKey, events);
    } catch (e) {
      log.append('Caught error while recording telemetry: ');
      log.appendLine(e);
    }
  })();
}
