---
id: TASK-652de6ead019
type: task
slug: find-json-spends-an-attention-budget-on-a-reader
title: find --json spends an attention budget on a reader that has none
created: 2026-08-22T20:51:57Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  find, review, scope and graph answer with every row under --json whatever the budget, and a test drives each of the four on a corpus seeded past the budget and asserts that every entity it holds is in the document. shown equals total in those documents unless a filter removed rows, and hidden names only what a filter withheld: a test asserts both on a corpus past the budget with no filter, and on one where --free hid a row. The human output of all four is unchanged, budget and cut notice included, asserted byte for byte against what it prints today. context --json stays budgeted and a test asserts it. The contract version is unchanged and the goldens are blessed with the diff read and stated. The machine surface document of a spec superseding the current CLI surface document states where the budget applies. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

ADR-3e6ce108edcd is the decision; this is the code it costs.

Measured on this corpus on 2026-08-22: `ank find --type adr --json` answers
`{"total":60,"shown":40,"hidden":0}`. Twenty rows are missing from a document a
program reads, and the field named `hidden` says zero because it counts what
`--free` withheld and knows nothing about the budget.

**It bit this session's own work.** A measurement of how many identifiers the
corpus's prose names that resolve to nothing was taken twice, because the first
pass built its set of existing entities from `find --json` and was handed 40 of
60 -- so entities that plainly exist were counted as absent. The second pass read
the roster from `ank graph --json`, which is not cut. A tool whose own measuring
is wrong by default is the argument for this task, and it is not hypothetical.

**The human output does not move.** The budget, the cut and the line naming
`ank config context_budget` all stay exactly where they are: what a terminal
shows is not what is being changed.

**`context` is the exception and it is not one.** Deciding what a reader is
handed first is that verb's answer rather than a limit on it, so its `--json`
stays budgeted.

**The contract version does not move.** ADR-6fd69efb629c allows a document to
gain a field within a version and forbids losing, renaming or retyping one.
Nothing is gained or lost here; two numbers that used to disagree now agree
unless a filter separated them, so a parser written against contract 1 reads the
same document and receives more of it.
