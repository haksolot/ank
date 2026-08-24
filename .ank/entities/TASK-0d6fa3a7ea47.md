---
id: TASK-0d6fa3a7ea47
type: task
slug: an-identifier-in-prose-that-names-nothing-is-rep
title: An identifier in prose that names nothing is reported by nobody
created: 2026-08-22T20:51:35Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-core/src/log.rs
blocked_by: []
done_criteria: |
  check reports, once for the corpus and never once per mention, the entity identifiers written in prose that name an entity the corpus does not hold, naming the first few and carrying the rest as a count: a signal, never a fault, so the exit code of a corpus carrying them is unchanged and no done is blocked by it. The prose read is the message of a log entry, a task's done_criteria and an entity's body, and a test asserts each of the three. An identifier that resolves is silent whatever its status, and a test seeds one naming a superseded document and asserts silence, because a frozen criterion naming one cannot be repaired. An identifier this corpus could not have minted is not reported, and a test seeds prose holding one. Nothing is refused at write time and nothing is rewritten: a test asserts the entity files are byte for byte what they were after a check that reported. The check catalogue of a spec superseding the current CLI surface document describes the finding, what it derives and what it deliberately does not conclude. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 3
---

ADR-1e6bcbf62e61 is the decision; this is the code it costs.

**Measured before it was proposed**, on this corpus on 2026-08-22: 1087 distinct
entity identifiers are named in the prose of log entries and 15 resolve to
nothing, which is 1.4 percent. The list is worth reading before implementing,
because it says what the finding will look like on the day it ships: roughly
half are fixture identifiers quoted in prose about tests -- `SPEC-00000000f006`,
`TASK-000000000001` and their kind -- which will never resolve and are perfectly
good writing. The other half are real: `ADR-6b3fa9ba3a05`, which is a mistyped
`ADR-6b3f19e08a24` and appears in the source as well; `TASK-eb2c8b0bee45`, which
is the closure reason that produced ADR-1e6bcbf62e61; and four more.

**One finding for the corpus.** That is the volume rule this catalogue already
applies to entities predating `author`, to actor values and to orphan entries,
and the measurement is what says it is the right one here: fifteen lines would be
fifteen lines a reader scrolls past, where one line naming the first few and
counting the rest is one line they read.

**The silent case is the one to get right.** An identifier that resolves is
silent whatever its status. A `done_criteria` naming a superseded document is
the case this session met three times, and it is frozen at claim: reporting it
would be a finding nobody can clear, which is the failure §11 names and which
this corpus has already refused twice.

**What must not be built.** No refusal at write time, no rewriting of anybody's
prose, and no treatment of a prose identifier as a reference: it confers nothing,
orders nothing and is followed by nobody.
