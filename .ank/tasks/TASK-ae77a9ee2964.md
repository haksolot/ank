---
id: TASK-ae77a9ee2964
type: task
slug: the-specification-carries-attest-detached-and-th
title: The specification carries attest --detached and the attestation category
created: 2026-08-13T05:46:43Z
author: claude-agent-b
status: open
scope:
  - docs/ank-spec-v1.1.md
blocked_by: [TASK-6d404f17f56d]
done_criteria: |
  docs/ank-spec-v1.1.md states, normatively: section 4's dispatch block shows the --detached form of attest alongside --proof; section 7 names external attestation as a third category beside durable state and ephemeral coordination, with refs/ank/proof/<id> as its home, no TTL, and pruning once an equivalent proof reaches the task file on the default branch; section 10 keeps its deferred row on a CI calling attest unprompted and says why --detached does not lift it. No source file changes in this task. ank check exits 0.
criteria_by: creator
schema: 2
version: 1
---

The verb half of ADR-493471d64ba0 ships in TASK-6d404f17f56d, whose scope is
five files under crates/ and no documentation. The ADR's own scope names
docs/ank-spec-v1.1.md, so the gap is declared rather than discovered.

Three places disagree with the binary today. Section 4's dispatch block spells
`ank attest <id> --proof <type>:<ref>` and the binary now also accepts
`--detached`. Section 7 separates durable state in files from ephemeral
coordination in refs, and the ADR names a third thing that split did not
anticipate -- external attestation, durable and authored by an actor with no
branch -- which is the paragraph section 7 is missing. Section 10 defers "a CI
calling attest on its own" as an integration, and that row stays true: nothing
calls attest automatically, and the flag does not change it. Say so in the row
rather than deleting it.

The short-form table in section 4 is deliberately untouched: `--detached` takes
no letter, as `--scope` and `--drop-scope` take none, and ADR-0c8ab846d262 asks
for a mapping only where one is declared.

This is documentation of a shipped verb, so the order of ADR-63b59c5c26f7 is
not violated by it following the code: what that rule governs is the format,
and no format field moves here.
