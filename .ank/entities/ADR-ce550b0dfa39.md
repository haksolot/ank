---
id: ADR-ce550b0dfa39
type: adr
slug: a-specification-is-an-entity-and-its-authority-i
title: A specification is an entity, and its authority is the whole document
created: 2026-08-15T06:53:45Z
author: claude-code/opus-5
status: accepted
scope:
  - docs/**
  - crates/ank-core/**
  - crates/ank-cli/**
constraint: |
  A specification is an entity of kind spec: one entity per document, never one per section. It declares no constraint field, because what it carries is description and not a rule, and it moves through the lifecycle every entity has: proposed, accepted, superseded, ratified by signature, anchored by hash, versioned. context names it the way it names an ADR, id and title, and never quotes its body; show is what reads it. A section is reached by reading the document, never by a second entity.
ratified: "427360235989"
schema: 3
version: 3
---

Section 10 refused a `spec` kind once already, and this decision is not that
proposal returning. What was refused was **spec sections as routable entities**:
cutting one document into scoped fragments served by `context` the way
constraints are. The reason given holds and is not disputed here — a
specification's authority comes from being one coherent document, its sections
are not independent the way ADRs are, and fragmenting it creates drift between
the fragments, which is the exact failure one document prevents.

This decision keeps that whole. One entity per document. A section is reached by
reading the document, never by an entity of its own, and `context` never quotes
a body it would have to cut.

What is being answered is a different complaint, and the record did not
anticipate it: **the specification does not move the way every other decision in
this repository moves.** It has no status, no supersession, no ratification, no
hash anchor, no version. It is a file edited by hand, in a tool whose premise is
that a decision should be readable, anchored and verifiable. The premise is not
applied to the document that states it.

The remedy section 10 offered instead has died on contact with use, and that is
measured rather than argued. Thirty-five ADRs in this corpus, and **not one is a
distilled specification section**. The ten that name `docs/ank-spec-v1.1.md` name
it in `scope:` — decisions that *change* the document, which is the opposite
direction. `see:`, the one field meant to point outward, is used three times,
never at the specification, and `context` never renders it.

Nobody tried because the vehicle cannot carry it. `constraint` is capped in
practice — the longest ever written is 1251 characters, median around 420 — it is
frozen at ratification, since `amend` exits 6 on an accepted ADR whose scope is
anchored in the ratification commit, it is invisible during orientation, which
serves id and title and no rule text at all, and the over-constrained ceiling of
half the budget is **already exceeded by eight tasks**, between 5839 and 12284
characters. The channel was full before anything was put in it.

Section 11 does not cover the remainder and cannot. Every mechanism it describes
is subtractive and presupposes a rule that could in principle be checked: a
constraint is born in prose because we do not yet know how to check it, and
`enforced_by` takes it out of injected context once we do. Nothing in section 11
ever puts a description *in*. "Ank already has the sink for what grows" is true
of rules and of nothing else.

And the missing feature is already implemented by hand. `CLAUDE.md` in this
repository carries a heading that admits it — implementation constraints,
summary of the ADRs, the ADRs are authoritative — across 120 lines that are
unscoped, unhashed, loaded on every session whatever the perimeter, and
confronted with the ADRs they summarise by nothing at all. It drifts by
construction. A shadow implementation of a refused feature is the strongest
evidence the refusal was wrong.

**What this decision does not do.** It does not route sections. It does not put a
specification into `context` in full: the attention budget is 8000 characters and
this document is 218601 bytes, twenty-seven times the whole budget, so serving it
was never on the table and naming it is the entire point. It does not make the
specification binding the way an ADR is — a `spec` describes, an `adr` binds, and
an entity that did both would be an ADR with an unbounded constraint, which is
the ceiling problem restated rather than solved. It does not add a `constraint`
field to the kind, and the absence is the difference that justifies a kind at all.

The cost is worth writing down. A specification entity is large, and every verb
that walks the corpus will walk it. `check` verifies the byte-for-byte round trip
on every entity, so the document is parsed and re-serialised on every run. That
is acceptable and it is not free.
