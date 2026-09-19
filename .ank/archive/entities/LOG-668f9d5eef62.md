---
id: LOG-668f9d5eef62
type: log
title: "Done. The set is derived rather than tracked: `here` is keyed on file names and `seen` on what"
created: 2026-08-22T00:39:51Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-5c7aae69a4c0
seq: 0
schema: 4
version: 1
---

 parsed, so the difference is exactly the files that exist and did not read. No new walk, no new state.

Five conclusions rested on absence and they split into two kinds, which is the thing worth recording. Four name a target, so the guard is a lookup: a blocked_by, a reference, a supersedes, and a log entry's about. One does not: "marked superseded but no spec supersedes it" is a claim about the whole corpus, the successor may be any of the unread files, and there is no id in hand to test. Its guard is the emptiness of the set. The chain that leads nowhere is the same shape and got the same treatment.

Falsified on a fixture before trusting it: a spec at schema 99 superseding a readable one that cites it back. Before, that shape produced a --drop-reference against a correct citation and a false broken succession. After, one honest unknown-schema fault and one signal naming the incompleteness with its count.

One deviation from the criterion, deliberate. It asks the new finding to carry "the command that resolves it". It does not, and I decided against inventing one. For the schema case the remedy is owned by the warning schema_ahead already prints, which is the subject of TASK-7a2c9d1b13a0 precisely because the command it names returns the caller where they were. Writing a second remedy here would be two sentences about one fix, free to disagree, and the second would have been as wrong as the first until that task lands. What the finding does carry is the cause, the count, and the ids, which is what a reader needs to know that resolution was partial.

Also cleared an unused_mut warning I left in TASK-027a429aad2e: cargo build is now silent.
