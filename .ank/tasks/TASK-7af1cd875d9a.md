---
id: TASK-7af1cd875d9a
type: task
slug: the-getting-started-guide-still-tells-the-reader
title: The getting-started guide still tells the reader to open config.yml
created: 2026-08-09T15:17:35Z
author: claude-code@ank
status: open
scope:
  - docs/getting-started.md
blocked_by: [TASK-797d64113614]
done_criteria: |
  docs/getting-started.md no longer instructs anyone to open .ank/config.yml, and the two error blocks it quotes match what the binary prints: the default-branch refusal names 'ank config default_branch <name>', and the undeclared-verifier refusal names 'ank config verifiers.<name>.run'. The section that adds a default branch and the one that declares verifiers both go through the verb. No other file changes, and cargo test stays green.
criteria_by: creator
schema: 2
version: 1
---

Fallout of TASK-797d64113614, found while changing the hints and left out of it because the guide is outside that task's scope.

The guide reproduces two error messages verbatim -- lines 90-92 and 163-164 as of this filing -- and both are the text the binary stopped printing when ADR-e64dfaafd578 was executed. It also says 'Open .ank/config.yml and add one line' at line 82 and points at the file again at line 151.

The guide addresses a human, and a human with an editor keeps every power they had (ADR-e64dfaafd578). So this is not a correctness rule being broken; it is a guide quoting output that no longer exists, which is the kind of staleness a reader has no way to detect. The fix is to quote what the binary now prints and to reach for the verb where the guide currently reaches for the file.
