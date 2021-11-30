---
title: IDE Telemetry
description: Information about telemetry collected by IDE integrations.
---

# IDE Telemetry

## VSCode

The VSCode extension collects **non-identifiable** telemetry data that we use
to improve Oso. We collect data into un-timestamped batches instead of sending
it on every policy load since we care about aggregate statistics; not tracking
your personal development behavior. **We will never sell this data**.

| Data collected | Link to code | Purpose |
| -------------- | ------------ | ------- |
| One-way hash of VSCode [`machineId`][machineId] | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L79) | Distinguish users in a non-identifiable way. This helps us distinguish 1 user encountering the same error 10,000 times from 1,000 users each encountering it 10 times. |
| One-way hash of VSCode [workspace URI][uri] | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L84) | Distinguish errors occurring in project A from errors occurring in project B. |
| # of diagnostics encountered for a particular load event | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L59) | Feature usage stats. |
| # of errors encountered for a particular load event | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L60) | Feature usage stats. |
| # of warnings encountered for a particular load event | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L63) | Feature usage stats. |
| Boolean indicating whether the load was successful (resulted in no errors) | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L61) | Feature usage stats. |
| # of rules in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L62) | Feature usage stats. |
| # of inline queries in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L108) | Feature usage stats. |
| # of "regular" (non-shorthand) rules in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L109) | Feature usage stats. |
| # of characters across all files in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L110) | Get a (very) rough sense of how much Polar code the average policy contains. |
| # of Polar files in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L111) | Get a rough sense for how common multi-file policies are. |
| # of rule types in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L112) | Distinguish invalid rule type errors for built-in rule types vs. (possibly) user-defined rule types. |
| # of resource blocks (actor & resource) in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L115) | Feature usage stats. |
| # of `actor` blocks in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L116) | Feature usage stats. |
| # of `resource` blocks in the loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L117) | Feature usage stats. |
| # of declarations (roles, permissions, and relations) in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L118) | Feature usage stats. |
| # of roles declared in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L119) | Feature usage stats. |
| # of permissions declared in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L120) | Feature usage stats. |
| # of relations declared in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L121) | Feature usage stats. |
| # of shorthand rules in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L122) | Feature usage stats. |
| # of shorthand rules that cross resource boundaries in loaded policy | [link](https://github.com/osohq/oso/blob/1a7a0ab130696a7849c04de5b8a869eda32d3998/vscode/oso/client/src/telemetry.ts#L123) | Feature usage stats. |

[machineId]: https://code.visualstudio.com/api/references/vscode-api#3252
[uri]: https://code.visualstudio.com/api/references/vscode-api#2515
