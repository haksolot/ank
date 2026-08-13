---
id: TASK-ae77a9ee2964
type: task
slug: the-specification-carries-attest-detached-and-th
title: The specification carries attest --detached and the attestation category
created: 2026-08-13T05:46:43Z
author: claude-agent-b
status: done
scope:
  - docs/ank-spec-v1.1.md
blocked_by: [TASK-6d404f17f56d]
done_criteria: |
  docs/ank-spec-v1.1.md states, normatively: section 4's dispatch block shows the --detached form of attest alongside --proof; section 7 names external attestation as a third category beside durable state and ephemeral coordination, with refs/ank/proof/<id> as its home, no TTL, and pruning once an equivalent proof reaches the task file on the default branch; section 10 keeps its deferred row on a CI calling attest unprompted and says why --detached does not lift it. No source file changes in this task. ank check exits 0.
criteria_by: creator
proof:
  - type: test
    ref: "31677348357"
    criteria: fa11b28283fe
schema: 2
version: 4
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

## Log
- 2026-08-13T07:19:18Z claude-agent-b — Three places, and a fourth I had to add so the document does not contradict itself. Section 4's dispatch block now shows the --detached form. Section 7 names external attestation as a third category beside durable state and ephemeral coordination, with the argument the ADR makes: a green pipeline is durable and authored by an actor with no branch, so it must survive and has no tree to travel in. The union, the absence of a TTL and the pruning predicate are stated there. Section 10 keeps its deferred row and says why the flag does not lift it -- --detached removes the obstacle a pipeline used to hit, which was having to produce a commit, and removing an obstacle is not calling the verb. The fourth: section 7 already argues that claim and completed share one address because two namespaces would let a stale claim and a completion coexist. Read against a separate proof namespace that reads as a contradiction, so the ref-lifecycle paragraph now says why it is not one -- claim and completed are mutually exclusive answers to who holds this task, an attestation answers what anchors this work, and a task legitimately carries both at once. Verified that the guard is live rather than assuming it: misspelling the verb in the dispatch block turns three tests in skill.rs red, because section_4_order parses that block and help is held to it. No source file changed. Suite green, ank check exits 0.
- 2026-08-13T07:24:16Z claude-agent-b — done, proof test:31677348357
