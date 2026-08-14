---
id: TASK-03fd4b2c27bc
type: task
slug: ank-help-groups-its-listing-by-the-moment-a-verb
title: ank help groups its listing by the moment a verb is used
created: 2026-08-11T22:22:32Z
author: claude-code@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  ank help prints every verb the dispatch table holds, under lowercase group
  headings, with a blank line between groups and no verb omitted. Within a group
  the verbs keep their COMMANDS order. ank help --json carries a group field on
  every verb. ank help <verb> is byte-identical to what it prints today.
  
  Two tests in crates/ank-cli/tests/cli.rs: every verb in COMMANDS appears exactly
  once in the grouped listing, and every verb carries a non-empty group so a verb
  added later cannot fall off the end unnoticed.
criteria_by: creator
schema: 2
version: 2
---

Implements ADR-f61e2d2c75e8, which supersedes ADR-e17e1bbd93ff on this clause
alone. The freeze on SKILL.md is carried forward unchanged, so nothing in
`skill/SKILL.md` moves and `tests/skill.rs` should still pass untouched — if it
does not, something went further than this task.

`CommandSpec` gains one field. `COMMANDS` keeps its order: the array order is
section 4's, the tests hold it there, and grouping is a second axis laid over that
order rather than a re-sort. A verb must not move relative to its neighbours
inside its group.

The groups, as the ADR settles them:

    run the loop             context claim show log done release
    shape the work           new amend close attest review accept
    look around              find status graph scope
    keep the corpus honest   check edit
    set up a repository      init config help

The rendering already computes a usage column width across all of `COMMANDS`.
Keep computing it globally rather than per group, or the columns will not line up
between sections and the listing will read as five tables instead of one.

The trailer lines — `global:`, `ank help <verb> ...`, `ank --version ...` — are
not a group and gain no heading.

Do not trim the listing. git shows common commands and sends the rest to
`git help -a`; that split is the mistake ADR-e17e1bbd93ff corrected, and hiding a
verb from the one surface claiming to be complete is worse than never teaching it.
