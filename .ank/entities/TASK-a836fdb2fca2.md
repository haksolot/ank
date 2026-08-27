---
id: TASK-a836fdb2fca2
type: task
slug: the-contract-s-note-and-the-header-diagram-descr
title: The contract's note and the header diagram describe a reader with six verbs and a band of chrome
created: 2026-08-27T04:06:48Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-contract/**
  - crates/ank-tui/**
blocked_by: []
done_criteria: |
  ank help tui names every verb the reader spells, new included, and describes them in terms true of all of them rather than of the six that act on the focused panel's entity. The header diagram in crates/ank-tui/src/view.rs draws the frame the built binary actually paints: no band of keys, no trailer, and no comment in that file says six verbs. A test in crates/ank-tui fails when the contract's tui note and the reader's own binding table disagree on which verbs the reader spells, so a verb added to the reader cannot leave the note behind a second time. cargo test --workspace green, cargo fmt --check clean, ank check green.
criteria_by: creator
schema: 4
version: 1
---

Two agents found these in sequence, each declared them out of its own perimeter,
and each recorded them rather than widening a scope to reach them. Neither is a
defect in code that runs: both are prose that describes a reader which no longer
exists, which is the kind of drift this corpus is built to make visible instead
of tolerable.

WHAT THE CONTRACT SAYS AND WHAT THE READER DOES. ank-contract's note on the `tui`
verb reads "claim, log, release, done, amend and accept act on the entity the
focused panel names". That was true of six verbs. TASK-d832452630d2 added a
seventh, `new`, which is Group::Create rather than Group::Write precisely because
its first positional is a kind and not the marked panel's entity -- so the note
is wrong twice over: it is short one verb, and the sentence it uses to describe
the six is false of the seventh. LOG-f455c09157bc records the reasoning.

WHAT THE DIAGRAM DRAWS AND WHAT THE BINARY PAINTS. view.rs's header diagram still
draws "the keys, and the six verbs that write" as a band of chrome.
TASK-9a402a54886f deleted that band -- the trailer's two rows and the target
band's two are gone, and the frame spends four non-panel rows at 80x24 against
the thirteen the diagram implies. LOG-1afc1b09f95b measured it.

THE POINT IS THE TEST, NOT THE TWO EDITS. Correcting both texts is an afternoon's
work that buys a week. What is worth buying is the check that fails when they
disagree again: crates/ank-tui can see ank-contract's table and its own bindings,
so the comparison has somewhere to live. A note transcribed beside a table drifts
from it; a note measured against it cannot. The reader's whole design already
rests on reading the contract's verb table rather than copying it, and this is
the same argument applied to the prose about that table.
