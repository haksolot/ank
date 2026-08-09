---
id: TASK-bfe1cbd9ec42
type: task
slug: every-status-a-listing-prints-carries-a-colour-a
title: Every status a listing prints carries a colour, and in_progress is not missing from the table
created: 2026-08-09T02:57:52Z
author: seanl@sean-laptop
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/style.rs
blocked_by: []
done_criteria: |
  Section 4 of docs/ank-spec-v1.1.md declares one colour for every status the CLI can print, and it declares it before the code moves: [open] blue, [in_progress] cyan alongside [claimed:...], [proposed] magenta, [accepted] green and [superseded] dim stated rather than left implied by the transition-word rule, and [blocked] struck from the table because nothing emits it. state_sgr in crates/ank-cli/src/style.rs is the only function that changes, it returns a colour for every one of those states, and no status the CLI prints reaches a reader unstyled. [open expired:who] is still yellow. Every escape byte is still written in style.rs and nowhere else. The table test in style.rs enumerates the new table state by state rather than spot-checking it, and asserts through the status and landed accessors both, so a second table that happened to agree today would not pass. The whole integration suite through the binary stays green with no edit: no escape sequence reaches a pipe, and the colour of a status is unit-tested because a spawned binary writes to a pipe and can never observe it.
criteria_by: creator
proof:
  - type: test
    ref: "31291756290"
    criteria: 0e4bf5f40617
schema: 2
version: 4
---

## Why

TASK-4601ed18d84e verified that the palette covered every verb. Nobody verified
that it covered every status, and it does not.

state_sgr falls through to None on three states. Two of them are deliberate --
section 4 says `[open] | default` in as many words -- but the third is not:
`in_progress` appears nowhere, not in the table and not in the code. So one
task renders cyan `[claimed:who]` under `context`, which reads the claim refs,
and unstyled `[in_progress]` under `find`, `scope`, `graph` and `show`'s edge
sections, which read the index. One fact, two colours, decided by which verb
the reader happened to type.

## What is being decided, and what is not

Giving `[open]` a colour is a change to section 4, not a patch to a table in
the code: ADR-0c8ab846d262 binds the palette to the specification, and the
specification is what moves first. No new ADR is needed -- 0c8a already decided
that colour is a property of the reader, and this only fills in the table it
governs.

Blue and magenta are the two remaining base colours. Spending them here closes
the palette: after this, no status is without a colour and no base colour is
unassigned, which is a property a reader can check rather than a list they have
to remember.

## Consequence to expect, not to fix

`release` prints `released TASK-x -> open` through landed(), which reads the
same table. Its `open` becomes blue. That is section 4's rule holding -- the
state a transition lands on takes exactly the colour its bracketed marker takes
-- and not a regression to undo.

## Follow-on

`[in_progress]` and `[claimed:who]` will still be two different strings after
this task; only their colour is unified here. Unifying the text is its own
task, because it means `find`, `scope` and `graph` reading the claim refs.

## Log
- 2026-08-09T03:03:45Z seanl@sean-laptop — Falsified the two table tests before trusting them: dropping the `open` arm from state_sgr turns both red with the message each was written to give -- "the marker [open] is not the colour section 4 gives it" and "open reaches a reader with no colour". Restored, 325 green.

The model-driven test is the one that matters. It reads the variants through a match rather than a typed array, so adding a variant to TaskStatus or AdrStatus stops it compiling. A bare list would have gone stale in silence, and going stale in silence is precisely how in_progress came to be absent from the table: TASK-4601ed18d84e verified the palette covered every verb, and nothing verified it covered every status.

Also struck [blocked] from section 4. It was in the table and in state_sgr and nothing has ever emitted it -- blocked is derived from blocked_by at read time, never stored, so no listing can print it. A dead row in a table that is meant to be checkable is worse than an omission.
- 2026-08-09T03:10:11Z seanl@sean-laptop — done, proof test:31291756290
