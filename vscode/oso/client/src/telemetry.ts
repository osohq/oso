import { createHash } from 'crypto';

import { env, Uri, workspace } from 'vscode';
import * as Mixpanel from 'mixpanel';
import {
  Diagnostic,
  DiagnosticSeverity as Severity,
} from 'vscode-languageclient';

// Flush telemetry events in batches every hour.
export const TELEMETRY_INTERVAL = 1000 * 60 * 60;

const hash = (contents: { toString(): string }) =>
  createHash('sha256').update(contents.toString()).digest('base64');

// One-way hash of VSCode machine ID.
const distinct_id = hash(env.machineId);

const MIXPANEL_PROJECT_TOKEN = 'd14a9580b894059dffd19437b7ddd7be';
const mixpanel = Mixpanel.init(MIXPANEL_PROJECT_TOKEN, { protocol: 'https' });
const trackBatch = (events: Mixpanel.Event[]) =>
  new Promise<void>((res, rej) =>
    mixpanel.track_batch(events, errors => {
      if (!errors) return res();
      rej(errors[0]);
    })
  );

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

const queue: MixpanelEvent[] = [];

export async function flushQueue(): Promise<void> {
  if (!telemetryEnabled()) return;

  // Drain all queued events.
  const events = queue.splice(0);

  if (events.length === 0) return;

  return trackBatch(events);
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

export function enqueueEvent(uri: Uri, event: TelemetryEvent): void {
  if (!telemetryEnabled()) return;

  const load_id = hash(Math.random());
  const workspace_id = hash(uri);
  const metadata: MixpanelMetadata = { distinct_id, load_id, workspace_id };

  const { diagnostics, general_stats, resource_block_stats } = event;

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

  queue.push(loadEvent, ...diagnosticEvents);
}
