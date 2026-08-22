---
id: ADR-3e6ce108edcd
type: adr
slug: the-attention-budget-is-the-human-reader-s-and-a
title: The attention budget is the human reader's, and a listing answers a program whole
created: 2026-08-22T20:50:52Z
author: claude-code/opus-5
status: accepted
scope:
  - crates/ank-cli/**
constraint: |
  The attention budget cuts what a human is shown and never what a program receives. A listing verb -- find, review, scope, graph -- answers with every row under --json, whatever the budget, and its human output is unchanged, cut notice included. context keeps the budget under --json, because deciding what a reader is handed first is that verb's answer rather than a limit on it. In a --json listing shown and total agree unless a filter removed rows, and hidden names only what a filter withheld. No flag lifts the budget: a caller who has to know to ask is a caller who still gets a short answer.
ratified: aaf75acd3031
verified:
  - by: claude-code/opus-5
    at: 2026-08-22T20:55:55Z
schema: 4
version: 2
---

Measured on this corpus on 2026-08-22: `ank find --type adr --json` answers
`{"total":60,"shown":40,"hidden":0}`. Twenty rows are missing from a document a
program is meant to read, and the field named `hidden` says zero, because
`hidden` counts what `--free` withheld and knows nothing about the budget. A
parser that trusted either number would be wrong about the corpus, and one that
compared them would be told the difference is nobody's doing.

**The corpus has already said this once, about a neighbouring verb.** `ank log
--json` carries the message whole and never the elided line, on the reasoning
that a parser is not reading a page and has no budget to spend. Nothing extends
that to the listings, and nothing decided it should not be: `find`, `review`,
`scope` and `graph` inherited the cut from the human output they share a code
path with.

**The budget is the human reader's**, and it is the whole subject of one verb.
`context` exists to spend it: it is the verb that decides what a reader is
handed first, and its `--json` stays budgeted for that reason, because the
budget *is* its answer rather than a limit on it. Everywhere else the budget is
presentation -- the same split ADR-0c8ab846d262 draws for colour, one step
further along. What a terminal shows and what a program receives are two
questions, and only one of them has a page.

**The shape does not move, so the contract version does not.** ADR-6fd69efb629c
lets a document gain a field within a version and forbids losing, renaming or
retyping one. `shown` and `total` stay, and stay meaning what they say; what
changes is that they agree unless a filter separated them. A parser written
against contract 1 reads the same document and gets more of it.

**What must not be built is a flag.** An `--all` that a caller has to know about
puts the defect behind a discoverability problem: the parser that did not know
to ask still gets a short answer, and now the tool can say it was told.
