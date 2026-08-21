---
id: TASK-3c12e0ced2c0
type: task
slug: a-verb-that-changes-content-outside-a-transition
title: A verb that changes content outside a transition writes the entry that accounts for it
created: 2026-08-21T20:45:08Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-027a429aad2e, TASK-353036d7972f]
done_criteria: |
  ank edit on both its paths, ank amend, and ank claim --criteria each write one log entry marked as machinery, naming the fields that changed, the version moved from and to, and freeze_hash_short of the state replaced. A verb that changes nothing writes none. The entry is written once and never revisited, and no verb reads it to decide anything: a test asserts that deleting every such entry changes no exit code and no answer of any other verb, which is what makes it a trace and not an anchor. Driven through the binary end to end: an entity created and then edited twice answers ank log with the two entries in order, each naming its fields and its version transition, and the hash in the first entry matches what freeze_hash_short returns for the state the second entry replaced. A status transition writes none of this, and a test on done asserts it. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 3
version: 1
---

The writing half of ADR-16813b3bcf37, and it is deliberately the third task
rather than the first: an entry that arrives before TASK-027a429aad2e can
separate it damages `ank log`, and one written from a surface whose refusals
are not yet pinned records an edit that should have been refused.

**Three verbs and no fourth.** `edit` and `amend` change content, and
`claim --criteria` writes a `done_criteria` the task did not have, which is
content the whole authority model then rests on. Everything else that touches an
entity is a status transition with a record of its own: `done` writes a proof
and a ref, `accept` writes a signed commit, `release` and `close` delete a ref.
Tracing a transition twice would say nothing the corpus does not already say
and would put mechanical lines on every task in the corpus.

**The hash is of the state replaced, and it is not an anchor.** ADR-ff294eff4d1a
requires that nothing authoritative be anchored in the log and that no hash
chain over it, and this respects both: the value exists so a reader handed a
claim about a past state can check it, and no verb consults it. The criterion
asks for that property to be tested by deletion rather than asserted in a
comment, because a trace that something quietly depends on has stopped being a
trace.

**What must not be built here.** No refusal, on any path, on any count. The
signal that reads these entries is TASK order and is a signal; a verb that
declined to write because the arithmetic looked wrong would be the wall
ADR-6b3f19e08a24 and ADR-3877fef1d662 both refuse.
