---
id: ADR-cb68e20acbad
type: adr
slug: a-ratified-entity-s-scope-git-records-as-deleted
title: A ratified entity's scope git records as deleted owes nothing while another of its scopes lives
created: 2026-10-24T17:39:31Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/human.rs
constraint: |
  For an accepted ADR or spec, a dead scope git records as deleted is reported as owing nothing when the entity keeps at least one scope that matches a file, naming the deleting commit; a dead scope git records as renamed stays a signal and proposes accept --rescope rather than a supersession; an entity all of whose scopes are dead is reported as today; check writes nothing and moves no anchor.
amends: [ADR-3094538d831e]
schema: 4
version: 1
---

## Context

Measured 2026-10-24 on this corpus: seven accepted ADRs (ADR-052accd6e3b2,
ADR-0bb7ea8991bc, ADR-47e2ac102f58, ADR-493471d64ba0, ADR-9307e5d214a7,
ADR-af533e7a3e03, ADR-b6b69053a47b) carry a dead scope on
docs/ank-spec-v1.1.md, which git records as deleted when the specification
became entities. Each keeps one or two live scopes, so each still governs
code. Their only repair today is a supersession: seven new ADRs, seven
signatures and citations re-pointed in 75 files, for decisions that do not
change. So the signals stay, and a signal that never clears teaches the reader
to stop reading signals.

ADR-3094538d831e already lowers severity where git answers, and says nothing
is owed for a closed task's dead scope. A deleted path inside a decision that
still governs other paths is the same case: there is nowhere for that scope to
go, and the decision is whole without it.

## Decision

1. For an accepted ADR or spec, a dead scope git records as **deleted** is
   reported as owing nothing, provided the entity keeps at least one scope
   that matches a file. The finding stays visible under its kind and names the
   deleting commit, as today.
2. A dead scope git records as **renamed** stays a signal: the perimeter
   probably ought to follow the file, and the finding proposes the act that
   moves it (ADR on accept --rescope, proposed alongside) rather than a
   supersession.
3. An entity **all** of whose scopes are dead is unchanged: it governs nothing,
   and that is worth saying.
4. Nothing is written, nothing is signed, no anchor moves. This changes what
   check says, never what the corpus holds.

## Rejected

- Silencing renames too: a renamed file would leave a decision's perimeter
  without anyone choosing it.
- Rewriting the scope automatically: ADR-3094538d831e forbids any automatic
  repair, and the scope is anchored.
