---
id: TASK-4eef0864be46
type: task
slug: format-md-states-the-anchor-the-schema-and-the-a
title: format.md states the anchor, the schema and the archive as the binary has them
created: 2026-09-19T18:26:17Z
author: claude-code/opus-5+docs-audit
status: open
scope:
  - docs/format.md
blocked_by: []
done_criteria: |
  docs/format.md gives the ratification anchor recipe that reproduces the constraint+scope hash ank accept records, with a worked vector (text, scope, hash) taken from a real accept; says schema 4 and that 1 to 4 are read; lists edit, create and method as records values; carries a log split example whose message exceeds 100 characters and splits where ank log splits it; describes the ratification walk as rev-list --full-history with no path restriction, matched on the commit subject; states what deleting index.db loses; says a non-canonical file is read but is a fault under ank check, CRLF excepted; names the number parser behind the quoting rule; and its conformance list matches crates/ank-core/tests/golden/invalid/ file for file.
criteria_by: creator
schema: 4
version: 1
---

Findings from the 2026-09-19 audit, each measured in a scratch corpus against ank 0.8.0. The anchor: human.rs ratification_anchor builds text, newline, globs each followed by a newline, then freeze_hash_short renormalises and drops the final newline; the documented buffer hashes to b88d9f81eb08 where accept recorded 33045e58af8d for constraint 'Never Y' and scope src/**. The code is not to change: every ratification on record rests on the current hash. SPEC-861d carries the same path-restricted walk wording; that is a supersession for a human to decide, not part of this task. No declared verifier reads prose, so verify: is left empty on purpose and the close is a commit proof; the future doc-output test (ADR-2b62b9a1fe67) is what will hold these pages afterwards.
