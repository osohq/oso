import { createHash } from 'crypto';

import { ExtensionContext, env, Uri, OutputChannel } from 'vscode';
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

type MixpanelLoadEvent = {
  event: 'TEST_load';
  properties: {
    diagnostics: number;
    errors: number;
    successful: boolean;
    total_rules: number;
    warnings: number;
  };
} & {
  properties: TelemetryEvent['general_stats'] &
    TelemetryEvent['resource_block_stats'];
};

type MixpanelDiagnosticEvent = {
  event: 'TEST_diagnostic';
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
  general_stats: {
    inline_queries: number;
    longhand_rules: number;
    polar_chars: number;
    polar_files: number;
    rule_types: number;
  };
  resource_block_stats: {
    resource_blocks: number;
    actors: number;
    resources: number;
    declarations: number;
    roles: number;
    permissions: number;
    relations: number;
    shorthand_rules: number;
    cross_resource_shorthand_rules: number;
  };
};

export function enqueueEvent(
  state: State,
  log: OutputChannel,
  uri: Uri,
  { diagnostics, general_stats, resource_block_stats }: TelemetryEvent
): void {
  void (async () => {
    try {
      const load_id = hash(Math.random().toString());
      const workspace_id = hash(uri.toString());
      const metadata: MixpanelMetadata = { distinct_id, load_id, workspace_id };

      const errors = diagnostics.filter(d => d.severity === Severity.Error);
      const warnings = diagnostics.filter(d => d.severity === Severity.Warning);

      const loadEvent: MixpanelEvent = {
        event: 'TEST_load',
        properties: {
          diagnostics: diagnostics.length,
          errors: errors.length,
          successful: errors.length === 0,
          total_rules:
            general_stats.longhand_rules + resource_block_stats.shorthand_rules,
          warnings: warnings.length,
          ...general_stats,
          ...resource_block_stats,
          ...metadata,
        },
      };

      const diagnosticEvents: MixpanelEvent[] = diagnostics.map(({ code }) => ({
        event: 'TEST_diagnostic',
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
