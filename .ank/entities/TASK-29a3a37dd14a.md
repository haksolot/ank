---
id: TASK-29a3a37dd14a
type: task
slug: releasing-md-names-the-trusted-publisher-s-publi
title: releasing.md names the trusted publisher's publish-directly action
created: 2026-11-07T11:04:59Z
author: claude-releasing-doc
status: in_progress
scope:
  - docs/releasing.md
blocked_by: []
done_criteria: |
  docs/releasing.md says each package's trusted publisher must allow publishing directly under Allowed actions, names the 403 a stage-only trust returns, and the workspace suite including the prose lint stays green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---
