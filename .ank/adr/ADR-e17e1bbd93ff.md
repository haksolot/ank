---
id: ADR-e17e1bbd93ff
type: adr
slug: the-skill-teaches-planning-as-well-as-the-loop
title: The skill teaches planning as well as the loop
created: 2026-08-05T04:03:55Z
author: seanl@sean-laptop
status: accepted
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - docs/**
constraint: |
  The CLI exposes one surface: every verb is available to every caller, and the
  CLI refuses on state, never on identity. The only hard authority line is the
  signed ratification commit produced by accept. ank help stays one flat listing
  of every verb, in the order of section 4, with no headings and no grouping.
  The verbs themselves do not change.
  
  What changes is the content of the freeze. SKILL.md teaches two modes, and its
  content remains frozen by revision hash. The loop: context, claim, show, log,
  done, with new, find and release off-loop. Planning: ank new adr, ank amend,
  ank review, ank graph, ank check, ank find --status open to list what remains,
  and the fact that accept is human, signed, and runs only on the default
  branch — the skill teaches what accept is, never that an agent should run it.
  The size ceiling moves with the content: at most 140 lines and 1200 words,
  a ceiling to notice drift, not a target to fill.
supersedes: ADR-c656cbcc33a9
ratified: ddcb4b4ec828
schema: 2
version: 2
---

## Context

ADR-c656cbcc33a9 froze SKILL.md on the loop alone: an agent taught only by that
file can execute work, but cannot propose a decision, amend a graph, or check
the coherence of the corpus. ank new adr is never mentioned; an agent that sees
an architectural problem has no documented path from the observation to a
recorded ADR. Planning — deciding what tasks should exist, in what order, under
which constraints — is exactly the activity that produces what other agents
then loop on, and it was untaught.

## The argument any addition to SKILL.md has to beat

The spec (section 4) states it: SKILL.md is loaded permanently, so every word
costs tokens on every call, for every agent, including the ones that only loop.
The addition is accepted here with eyes open. Planning is the highest-leverage
activity in the corpus — a badly shaped backlog wastes more tokens downstream
than the teaching costs upstream — and the alternative (a second, separately
loaded skill) was considered and rejected by the maintainer in favour of one
file that tells the whole truth. The ceiling moves from 80 lines / 700 words to
140 lines / 1200 words, and stays a ceiling to notice drift, not a target to
fill.

## What does not move

One surface, refusal on state never identity, the flat help listing, and the
verb set are all carried over unchanged from ADR-c656cbcc33a9. accept remains
outside what the skill invites an agent to do: it is described (human, signed,
default branch only) so a planning agent knows where its own authority ends.
