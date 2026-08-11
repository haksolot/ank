---
id: ADR-f61e2d2c75e8
type: adr
slug: ank-help-groups-by-the-moment-a-verb-is-used-and
title: ank help groups by the moment a verb is used, and hides none
created: 2026-08-11T22:18:51Z
author: claude-code@sean-laptop
status: proposed
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/src/cli.rs
  - docs/**
constraint: |
  The CLI exposes one surface: every verb is available to every caller, and the
  CLI refuses on state, never on identity. The only hard authority line is the
  signed ratification commit produced by accept. The verbs themselves do not
  change.
  
  ank help lists every verb. No verb is hidden, and there is no second listing to
  ask for. The listing is grouped by the moment a verb is reached for, under
  lowercase headings, and within a group the order stays section 4's. A group says
  when a verb is used and never who may use it; a heading that sorted callers would
  be the wall this project has already refused twice.
  
  The freeze on SKILL.md is carried forward from ADR-e17e1bbd93ff unchanged.
  SKILL.md teaches two modes, and its content remains frozen by revision hash. The
  loop: context, claim, show, log, done, with new, find and release off-loop.
  Planning: ank new adr, ank amend, ank review, ank graph, ank check, ank find
  --status open to list what remains, and the fact that accept is human, signed,
  and runs only on the default branch — the skill teaches what accept is, never
  that an agent should run it. The size ceiling stays at most 140 lines and 1200
  words, a ceiling to notice drift, not a target to fill.
supersedes: ADR-e17e1bbd93ff
schema: 2
version: 1
---

## Context

`ank help` prints twenty-one verbs as one undifferentiated block. ADR-c656cbcc33a9
made it so and ADR-e17e1bbd93ff carried the clause forward; the argument was that
"layering is grouping, and a grouping printed by the binary is a claim the binary
makes about who a verb is for", and that section 4's ordering "carries the same
information as a heading without asserting a category".

The first half of that argument is right and this ADR keeps it. The second half is
what twenty-one lines disprove.

## The objection, answered

**A group is a claim about who a verb is for.** It was, in the shape being
rejected. The layering ADR-c656cbcc33a9 removed was the residue of an agent
surface and a human surface — headings that told a caller which verbs were theirs,
behind a wall built from `$ANK_AGENT`, which the caller sets itself. Refusing that
was correct and nothing here reopens it.

Grouping by **the moment a verb is reached for** asserts nothing about audience.
`check` sits under keeping the corpus honest whether a human or an agent runs it,
and the refusal machinery still consults no caller. The distinction is the one
between a map and a gate, and the previous ADRs were about gates.

**Ordering carries the same information as a heading.** For five verbs, yes. For
twenty-one it does not, and the evidence is direct: a reader of this project's
help said the structure was invisible and produced `git help` as the contrast. The
information is only carried if somebody already knows the ordering is meaningful,
which is precisely what a first reader does not know.

**The taxonomy already exists and is already printed.** `skill/SKILL.md` opens
with Loop, Off-loop and Planning, on three lines, to every agent on every session.
So the categories are not being invented by the binary — they are being taught by
the documentation, and `help` is the one surface that pretends they do not exist.
Grouping there makes two surfaces agree; keeping it flat is what makes them
disagree.

## What is taken from git, and what is not

git's help groups by situation — starting a working area, working on the current
change, examining history — and that is the shape borrowed, because it answers the
question a reader actually arrives with.

What is **not** borrowed is the trimming. `git help` shows the common commands and
sends the rest to `git help -a`. Hiding verbs behind a second invocation is the
mistake ADR-e17e1bbd93ff corrected when it separated what is *taught* from what is
*available*: an agent that types `ank scope` gets the scope, and being untold is
not being refused. A hidden verb is worse than untold — it is untold by the one
surface that claims to be complete. All twenty-one stay visible.

## Rejected

**Grouping only in `--json`, keeping the human listing flat.** It would give a
machine the structure and withhold it from the reader who needs it more. Section 4
already says structure is emitted identically to everyone and only colour depends
on the reader.

**Re-sorting `COMMANDS`.** The array order is section 4's and the tests hold it
there. Grouping is a second axis laid over that order, not a replacement for it,
so the loop still comes first and a verb never moves relative to its neighbours
inside a group.

**Adding the groups to SKILL.md.** They are already there, in the three-line
summary, and spelling them out again would spend the permanently-loaded token
budget to restate what `ank help` now prints on demand.

## Consequences

`CommandSpec` gains one field. The test that every verb appears in the listing
gains two: every verb carries a group, and no group is empty — which is what stops
a twenty-second verb from being added with no home and silently disappearing off
the end.

Nothing in `ank help <verb>` moves. It never had headings and does not gain any.
