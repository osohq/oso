/* eslint-disable @typescript-eslint/restrict-template-expressions */

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

const hash = (contents: { toString(): string }) =>
  createHash('sha256').update(contents.toString()).digest('base64');

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

export function flushQueue(state: State, log: OutputChannel): () => void {
  return () => {
    if (!telemetryEnabled()) return;

    void (async () => {
      try {
        // Retrieve all queued events.
        const events = state.get<MixpanelEvent[]>(telemetryEventsKey, []);

        if (events.length === 0) return;

        // Clear events queue.
        log.appendLine(`Flushing ${events.length} events`);
        await state.update(telemetryEventsKey, []);

        mixpanel.track_batch(events, errors =>
          (errors || []).forEach(({ name, message, stack }) => {
            log.appendLine(`Mixpanel track_batch error: ${name}\n\t${message}`);
            if (stack) log.appendLine(`\t${stack}`);
          })
        );
      } catch (e) {
        log.appendLine(`Caught error while sending telemetry: ${e}`);
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
  if (!telemetryEnabled()) return;

  void (async () => {
    try {
      const load_id = hash(Math.random());
      const workspace_id = hash(uri);
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
      log.appendLine(`Caught error while recording telemetry: ${e}`);
    }
  })();
}
