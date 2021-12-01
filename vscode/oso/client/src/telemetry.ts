import { inspect } from 'util';

import { createHash } from 'crypto';

import type { DebouncedFunc } from 'lodash';
import { env, OutputChannel, Uri, workspace } from 'vscode';
// import * as Mixpanel from 'mixpanel';
import {
  Diagnostic,
  DiagnosticSeverity as Severity,
} from 'vscode-languageclient';

// Flush telemetry events in batches every hour.
export const TELEMETRY_INTERVAL = 1000 * 20;
export const TELEMETRY_STATE_KEY = 'telemetry.state';
export const TELEMETRY_DAILY_MAXIMUM = 60;
export const TELEMETRY_MONTHLY_MAXIMUM = 3000;
export type TelemetryCounters = {
  monthly: {
    reset: number;
    count: number;
  };
  daily: {
    reset: number;
    count: number;
  };
};

const loadEventName = 'TEST_load';

const hash = (contents: { toString(): string }) =>
  createHash('sha256')
    .update(`oso-vscode-telemetry:${contents.toString()}`)
    .digest('base64');

// One-way hash of VSCode machine ID.
const distinct_id = hash(env.machineId);

// const MIXPANEL_PROJECT_TOKEN = 'd14a9580b894059dffd19437b7ddd7be';
// const mixpanel = Mixpanel.init(MIXPANEL_PROJECT_TOKEN, { protocol: 'https' });
// const trackBatch = (events: Mixpanel.Event[]) =>
//   new Promise<void>((res, rej) =>
//     mixpanel.track_batch(events, errors => {
//       if (!errors) return res();
//       rej(errors[0]);
//     })
//   );

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

class DiagnosticStats {
  diagnostic_count: number;
  error_count: number;
  load_failure_count: number;
  load_success_count: number;
  unknown_diagnostic_count: number;
  warning_count: number;
  [diagnostic_code: `${string}_count`]: number;

  constructor() {
    this.diagnostic_count = 0;
    this.error_count = 0;
    this.load_failure_count = 0;
    this.load_success_count = 0;
    this.unknown_diagnostic_count = 0;
    this.warning_count = 0;
  }
}

function compileDiagnostics(stats: DiagnosticStats, diagnostics: Diagnostic[]) {
  const errors = diagnostics.filter(d => d.severity === Severity.Error);
  const warnings = diagnostics.filter(d => d.severity === Severity.Warning);

  stats.diagnostic_count += diagnostics.length;
  stats.error_count += errors.length;
  stats.load_failure_count += errors.length === 0 ? 0 : 1;
  stats.load_success_count += errors.length === 0 ? 1 : 0;
  stats.warning_count += warnings.length;

  for (const { code } of diagnostics) {
    if (typeof code !== 'string') {
      stats.unknown_diagnostic_count++;
    } else {
      const count = stats[`${code}_count`] || 0;
      stats[`${code}_count`] = count + 1;
    }
  }
}

export type TelemetryEvent = {
  diagnostics: Diagnostic[];
  policy_stats: {
    inline_queries: number;
    longhand_rules: number;
    polar_chars: number;
    polar_files: number;
    rule_types: number;
    total_rules: number;
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

type LoadStats = TelemetryEvent['policy_stats'] &
  TelemetryEvent['resource_block_stats'];

type MixpanelLoadEvent = {
  event: typeof loadEventName;
  properties: DiagnosticStats & LoadStats;
};

type MixpanelMetadata = {
  // One-way hash of VSCode machine ID.
  distinct_id: string;
  // One-way hash of workspace folder URI.
  workspace_folder: string;
};

type MixpanelEvent = { properties: MixpanelMetadata } & MixpanelLoadEvent;

const purgatory: Map<string, [LoadStats, DiagnosticStats]> = new Map();

export async function sendEvents(log: OutputChannel): Promise<number> {
  if (!telemetryEnabled()) return 0;

  // Drain all queued events, one for each workspace folder.
  const events: MixpanelEvent[] = [...purgatory.entries()].map(
    ([folder, [loadStats, diagnosticStats]]) => ({
      event: loadEventName,
      properties: {
        distinct_id,
        workspace_folder: hash(folder),
        ...diagnosticStats,
        ...loadStats,
      },
    })
  );

  purgatory.clear();

  log.appendLine('Sending:');
  for (const event of events) {
    log.appendLine(inspect(event));
  }

  if (events.length === 0) return 0;

  await Promise.resolve();
  // await trackBatch(events);

  return events.length;
}

export type TelemetryRecorder = DebouncedFunc<(event: TelemetryEvent) => void>;

export function recordEvent(uri: Uri, event: TelemetryEvent): void {
  if (!telemetryEnabled()) return;

  const { diagnostics, policy_stats, resource_block_stats } = event;

  const folder = uri.toString();
  const existing = purgatory.get(folder);
  const diagnosticStats = existing ? existing[1] : new DiagnosticStats();
  compileDiagnostics(diagnosticStats, diagnostics);
  purgatory.set(folder, [
    { ...policy_stats, ...resource_block_stats },
    diagnosticStats,
  ]);
}
