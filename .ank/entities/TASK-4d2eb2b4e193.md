---
id: TASK-4d2eb2b4e193
type: task
slug: the-bindings-are-one-table-and-the-reader-reads
title: The bindings are one table, and the reader reads it back
created: 2026-08-26T17:07:10Z
author: claude-code/opus-5+reader-redesign
status: done
scope:
  - crates/ank-tui/**
blocked_by: []
done_criteria: |
  crates/ank-tui declares every binding of the reader once -- its key, its aliases, the command it runs, the word it is called by, the group it belongs to, what the screen must hold before it is offered at all, and the CLI verb it spells where it spells one -- and keys::typed, App::actions and the key list are all computed from it rather than written beside it. The list of verbs that may be spawned is NOT computed from it and stays written in ank.rs, measured against the table by a test that runs that dependency the other way round: a gate generated from the table it guards guards nothing. Every verb the table names resolves in ank_contract::verbs::COMMANDS and every flag it names to a FlagSpec of that command, and no field of the table may ever carry the value - , which the CLI reads as stdin and this reader's child does not have. Driven on a pseudo-terminal, the built binary answers ? with a list naming every binding the table declares and nothing it does not.
criteria_by: creator
proof:
  - type: commit
    ref: c91b21fe6372c9ef7bb1b607decf6d6cd881e1b4
    criteria: c54dbbf75b81
    via: submitted
schema: 4
version: 6
---

Seams found while designing, written here so the first holder does not pay for
them a second time.

**`keys.rs:296 no_bare_key_can_write` goes red by design, and must be replaced
rather than deleted.** It exists because reaching a verb took a key *and* a
word, and that asymmetry is what this task spends. The invariant that survives
is narrower and still sufficient: every `Press::Run(Command::Act(_))` reaches
`App::propose` and never `Ank::act`. Together with `tests/dependencies.rs:273`'s
count of one `ank.act(` in `src/`, that is what the old asymmetry bought.
`keys.rs:786 no_chord_runs_a_confirmed_command_over_the_whole_table` then
becomes the single most load-bearing test in this crate, and is not to be
touched.

**The gate is measured against this table and never generated from it.**
`ank.rs:94 ACTS` stays a hand-written list. Running the dependency the other way
round -- every binding that writes is in `ACTS`, and `ACTS` is not built from
the bindings -- is what keeps it a gate.

**`ratatui::widgets::Clear` is re-exported unconditionally in ratatui 0.30**,
not behind `all-widgets`, so the overlay TASK-8a6578851244 builds costs no
feature change in `Cargo.toml`.
