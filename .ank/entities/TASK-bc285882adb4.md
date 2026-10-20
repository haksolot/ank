---
id: TASK-bc285882adb4
type: task
slug: docs-ratifying-md-names-accept-reanchor-as-the-w
title: docs/ratifying.md names accept --reanchor as the way a commit anchor moves into a ref
created: 2026-10-20T19:15:42Z
author: claude-code/orchestrator
status: open
scope:
  - docs/ratifying.md
blocked_by: []
done_criteria: |
  docs/ratifying.md, in its section on anchors made before the ref, names ank accept <id> --reanchor as the command that moves a legacy ratify <id> commit anchor into refs/ank/ratify/<id>, says that it re-signs under whoever runs it and rewrites no history, and lists the refusals the binary gives for it (a ref already carries the document, the file has moved from the commit's anchor, no such commit reachable in this clone), each matching the binary's own help or output.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

PR #658 (TASK-4b25843e6151) added `accept --reanchor` and noted in its body that the docs, out of its scope, should mention it. PR #655 had just rewritten docs/ratifying.md, whose section "Anchors made before the ref" is where the flag belongs.
