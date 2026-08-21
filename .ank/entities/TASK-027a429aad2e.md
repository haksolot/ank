---
id: TASK-027a429aad2e
type: task
slug: a-log-entry-says-what-kind-of-entry-it-is-and-th
title: A log entry says what kind of entry it is, and the work trace stops carrying machinery
created: 2026-08-21T20:44:14Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-core/src/model.rs
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/registry.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  A log entry may carry a field naming what it is, optional and absent by default, so that every entry already in the corpus stays valid and nothing is migrated. An entry that does not carry it is a work entry, which is what a reader means by the log. ank show and ank log present the two apart: the work trace holds only what a previous holder wrote, and a test drives the binary on an entity carrying both kinds and asserts the mechanical one appears in neither the work trace nor the count beside it. The round trip holds on the new field, canonical form included, and an unknown value in it is a check finding and never a parse error, on the terms ADR-3877fef1d662 already sets for a typed actor. A spec superseding SPEC-acee5d9cb21b carries the field in the log registry, on the optional side. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 3
version: 1
---

ADR-16813b3bcf37 puts the record of an edit in a log entry, which costs nothing
in storage because ADR-25f977377fa0 already made an entry an entity: an id, a
typed author, an instant, written once and never modified, and any kind may
carry entries. What it costs is the log itself, and that is what this task
protects.

`ank log <id>` is what an agent reads before repeating what a previous holder
already tried. An entity edited eight times would answer that question with
eight mechanical lines and whatever prose survives between them, and the verb
that exists to carry understanding would become the verb nobody reads. The
field is therefore not decoration: it is what keeps the work trace worth
reading once the machinery starts writing into the same place.

**Optional, and absent means work.** That direction is the only one that leaves
the thousand entries already written valid, and it is the direction
ADR-3877fef1d662 took for `author` for the same reason: the corpus is not
migrated by a rule it predates. It also makes the common case free, since a
`log` written by a holder carries nothing new.

**This lands before anything writes an entry**, deliberately. A trace that
arrives before the reader can separate it is a trace that damages the verb it
was added to, and the order is cheap to respect: nothing writes one until
TASK order says so.
