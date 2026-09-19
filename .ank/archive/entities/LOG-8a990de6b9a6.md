---
id: LOG-8a990de6b9a6
type: log
title: "Implemented with no new field: the listing prints spec.summary, the same string ank help <verb>"
created: 2026-08-11T03:27:37Z
author: seanl@sean-laptop
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
about: TASK-fe130d2b732c
seq: 4
schema: 3
version: 1
---

 prints above the flags. One text rather than two is stronger than any test comparing them, and the test asserts the identity anyway so a future renderer cannot paraphrase. Descriptions too long for the column are folded on words and indented under themselves, never truncated -- the clause a verb refuses on is always the tail of the sentence, which is exactly where truncation would drop it. Six summaries gained a refusal clause where the refusal is what distinguishes the verb: claim, log, done, accept, close, init. The mechanical rule is that every --flag a description names is offered by the verb or preceded by the word refuses, checked against the flags and global lines of the verb's own page. Proven to bite before being trusted: rewriting init's description to say '--repo names the target' fails with the verb, the flag and what the verb actually offers. Three existing tests asserted flags in the flat listing and now assert them on the per-verb page, which is where they moved -- help_lists_every_verb_of_the_table, help_answers_outside_a_repository, and the surface() helper two path-classification tests walk.
