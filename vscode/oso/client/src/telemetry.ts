/* eslint-disable @typescript-eslint/restrict-template-expressions */

import { createHash } from 'crypto';
import os from 'os';

import type { DebouncedFunc } from 'lodash';
import { env, OutputChannel, UIKind, Uri, version, workspace } from 'vscode';
import * as Mixpanel from 'mixpanel';
import {
  Diagnostic,
  DiagnosticSeverity as Severity,
} from 'vscode-languageclient';

import { version as extversion } from '../../package.json';
import { osoConfigKey } from './index';

const ONE_HOUR_IN_MS = 1_000 * 60 * 60;
const ONE_DAY_IN_MS = ONE_HOUR_IN_MS * 24;
const ONE_MONTH_IN_MS = ONE_DAY_IN_MS * 30;

// Flush telemetry events in batches every hour.
export const TELEMETRY_INTERVAL = ONE_HOUR_IN_MS;
export const TELEMETRY_STATE_KEY = 'telemetry.state';
const TELEMETRY_DAILY_MAXIMUM = 60;
const TELEMETRY_MONTHLY_MAXIMUM = 3_000;
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

export const counters: TelemetryCounters = {
  monthly: { count: 0, reset: 0 },
  daily: { count: 0, reset: 0 },
};

export const sendTelemetryEvents: (log: OutputChannel) => Promise<void> =
  async (log: OutputChannel) => {
    const now = Date.now();

    // If a month has elapsed, reset both counters and timestamps.
    if (now > counters.monthly.reset + ONE_MONTH_IN_MS) {
      counters.monthly = { reset: now, count: 0 };
      counters.daily = { reset: now, count: 0 };
    }

    // If a day has elapsed, reset the daily counter and timestamp.
    if (now > counters.daily.reset + ONE_DAY_IN_MS) {
      counters.daily = { reset: now, count: 0 };
    }

    // If at or over monthly count and *also* at or over daily count, no-op.
    if (
      counters.monthly.count >= TELEMETRY_MONTHLY_MAXIMUM &&
      counters.daily.count >= TELEMETRY_DAILY_MAXIMUM
    )
      return;

    try {
      const flushedEvents = await sendEvents();
      counters.monthly.count += flushedEvents;
      counters.daily.count += flushedEvents;
    } catch (e) {
      log.appendLine(`Caught error while sending telemetry: ${e}`);
    }
  };

export function seedState(state?: TelemetryCounters): void {
  if (state === undefined) {
    // Initialize monthly & daily reset timestamps; counters will already be
    // initialized to 0.
    const now = Date.now();
    counters.monthly.reset = now;
    counters.daily.reset = now;
  } else {
    // Initialize monthly & daily reset timestamps & counters from memento
    // state.
    counters.monthly = state.monthly;
    counters.daily = state.daily;
  }
}

const loadEventName = 'diagnostic_load';

const hash = (contents: { toString(): string }) =>
  createHash('sha256')
    .update(`oso-vscode-telemetry:${contents.toString()}`)
    .digest('base64');

// One-way hash of VS Code machine ID.
const distinct_id = hash(env.machineId);
// VS Code common telemetry properties.
// https://github.com/microsoft/vscode-extension-telemetry/blob/188ee72da1741565a7ac80162acb7a08924c6a51/src/common/baseTelemetryReporter.ts#L134-L174
const vscodeCommonProperties = {
  os: os.platform(),
  nodeArch: os.arch(),
  platformversion: os.release().replace(/^(\d+)(\.\d+)?(\.\d+)?(.*)/, '$1$2$3'),
  extname: 'osohq.oso',
  extversion,
  vscodesessionid: env.sessionId,
  vscodeversion: version,
  isnewappinstall: env.isNewAppInstall?.toString() || 'false',
  product: env.appHost,
  uikind: (() => {
    switch (env.uiKind) {
      case UIKind.Web:
        return 'web';
      case UIKind.Desktop:
        return 'desktop';
      default:
        return 'unknown';
    }
  })(),
  remotename: env.remoteName || 'none',
};

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
    .getConfiguration(`${osoConfigKey}.telemetry`)
    .get<'default' | 'on' | 'off' | undefined>('enabled');

  // Check if user explicitly enabled or disabled telemetry.
  if (setting === 'on') return true;
  if (setting === 'off') return false;

  // Otherwise, default to VS Code's telemetry setting.

  // VS Code >=1.55
  //
  // https://code.visualstudio.com/updates/v1_55#_telemetry-enablement-api
  if (env.isTelemetryEnabled !== undefined) return env.isTelemetryEnabled;

  // VS Code <1.55
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
      stats.unknown_diagnostic_count += 1;
    } else {
      const count = stats[`${code}_count`] || 0;
      stats[`${code}_count`] = count + 1;
    }
  }
}

type TelemetryEvent = {
  diagnostics: Diagnostic[];
  lsp_event: {
    lsp_method: LspMethod;
    lsp_file_extensions: string[];
  };
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

type LspMethod =
  | 'textDocument/didChange'
  | 'textDocument/didOpen'
  | 'workspace/didChangeWatchedFiles'
  | 'workspace/didDeleteFiles';

class LspStats {
  'LSP_method_textDocument/didChange_count': number;
  'LSP_method_textDocument/didOpen_count': number;
  'LSP_method_workspace/didChangeWatchedFiles_count': number;
  'LSP_method_workspace/didDeleteFiles_count': number;
  'LSP_file_extensions': string[];

  constructor(method: LspMethod, extensions: string[]) {
    this['LSP_method_textDocument/didChange_count'] = 0;
    this['LSP_method_textDocument/didOpen_count'] = 0;
    this['LSP_method_workspace/didChangeWatchedFiles_count'] = 0;
    this['LSP_method_workspace/didDeleteFiles_count'] = 0;
    this[`LSP_method_${method}_count`] = 1;
    this['LSP_file_extensions'] = [...new Set(extensions)];
  }
}

function combineLspStats(first: LspStats, maybeSecond?: LspStats) {
  if (!maybeSecond) return first;

  // Combine sets of file extensions.
  first['LSP_file_extensions'] = [
    ...new Set([
      ...first['LSP_file_extensions'],
      ...maybeSecond['LSP_file_extensions'],
    ]),
  ];
  // Combine counts of method occurrences.
  (
    [
      'LSP_method_textDocument/didChange_count',
      'LSP_method_textDocument/didOpen_count',
      'LSP_method_workspace/didChangeWatchedFiles_count',
      'LSP_method_workspace/didDeleteFiles_count',
    ] as (keyof LspStats)[]
  ).forEach(
    lspMethodCount =>
      ((first[lspMethodCount] as number) += maybeSecond[
        lspMethodCount
      ] as number)
  );
  return first;
}

type MixpanelLoadEvent = {
  event: typeof loadEventName;
  properties: DiagnosticStats & LoadStats & LspStats;
};

type MixpanelMetadata = {
  // One-way hash of VS Code machine ID.
  distinct_id: string;
  // One-way hash of workspace folder URI.
  workspace_folder: string;
  // VS Code common telemetry properties.
  os: string;
  nodeArch: string;
  platformversion: string;
  extname: string;
  extversion: string;
  vscodesessionid: string;
  vscodeversion: string;
  isnewappinstall: string;
  product: string;
  uikind: string;
  remotename: string;
};

type MixpanelEvent = { properties: MixpanelMetadata } & MixpanelLoadEvent;

const purgatory: Map<string, [LoadStats, DiagnosticStats, LspStats]> =
  new Map();

async function sendEvents(): Promise<number> {
  if (!telemetryEnabled()) return 0;

  // Drain all queued events, one for each workspace folder.
  const events: MixpanelEvent[] = [...purgatory.entries()].map(
    ([folder, [loadStats, diagnosticStats, lspStats]]) => ({
      event: loadEventName,
      properties: {
        distinct_id,
        ...vscodeCommonProperties,
        workspace_folder: hash(folder),
        ...diagnosticStats,
        ...loadStats,
        ...lspStats,
      },
    })
  );

  purgatory.clear();

  if (events.length === 0) return 0;

  await trackBatch(events);

  return events.length;
}

export type TelemetryRecorder = DebouncedFunc<(event: TelemetryEvent) => void>;

export function recordEvent(uri: Uri, event: TelemetryEvent): void {
  if (!telemetryEnabled()) return;

  const { diagnostics, lsp_event, policy_stats, resource_block_stats } = event;

  const folder = uri.toString();

  const existing = purgatory.get(folder);
  const diagnosticStats = existing ? existing[1] : new DiagnosticStats();
  compileDiagnostics(diagnosticStats, diagnostics);

  const lspStats = combineLspStats(
    new LspStats(lsp_event.lsp_method, lsp_event.lsp_file_extensions),
    existing?.[2]
  );

  const updatedMetrics: [LoadStats, DiagnosticStats, LspStats] = [
    { ...policy_stats, ...resource_block_stats },
    diagnosticStats,
    lspStats,
  ];

  purgatory.set(folder, updatedMetrics);
}
