---
id: ADR-85e6bbb195b8
type: adr
slug: the-name-is-ank
title: The project is called ank
created: 2026-07-29T10:40:00Z
status: accepted
scope:
  - crates/**
  - docs/**
  - skill/**
  - README.md
constraint: |
  The name of the project is ank. The binary is ank, the crates are ank-core
  and ank-cli, the state directory is .ank/, the ref namespace is
  refs/ank/*, the identity variable is ANK_AGENT. No occurrence of "ankor"
  remains, with the single exception of historical anchors that rewriting
  would falsify: log entries already written, and proof references pointing
  at an external artifact.
ratified: c0c1dc33a814
schema: 1
version: 3
---

Three letters, typed on every call of the agent loop and present in every path
of the state directory. The gain is small per occurrence and the occurrence is
constant — the same arithmetic that froze the surface at seven verbs
(ADR-2f8a61c04b7d).

The rename is a format change and not merely a branding one: `.ank/` is the name
any third-party tool must know in order to read the corpus, `refs/ank/*` the one
the host must fetch, `ANK_AGENT` the one the harness must set. The ordering
imposed by ADR-63b59c5c26f7 therefore applies: the specification first, then the
goldens, then the code. The goldens carry none of those three strings, which is
itself the proof that the entity format does not move.

**What was not rewritten.** Two categories, for the same reason: they are
anchors, and a rewritten anchor anchors nothing.

Log entries already written keep the identity they carried
(`claude-code@ankor`). The log is append-only, and a rewritten past entry is a
falsification of history visible in review (§3); the identity *was* that one, and
rewriting it for visual tidiness would be the opposite of what the field
establishes.

The proof reference `ci://haksolot/ankor/runs/30324400136` on TASK-ca4714f5c719
stays intact. It does not name the project, it locates an artifact at a third
party; renaming the repository on the host leaves a redirect, but that is the
host's decision, not ours. A proof is moreover a forbidden write on a `done` task
beyond appending (§3).

**The scopes and constraints of accepted ADRs were rewritten in place**, without
superseding. The distinction from ADR-63b59c5c26f7, which took the opposite
route, is sharp and worth stating: there, the substance of the rule changed — a
round-trip guaranteed on all input became a round-trip guaranteed on canonical
form. Here, `crates/ankor-cli/**` and `crates/ank-cli/**` denote the same code,
and `refs/ankor/claims/<id>` the same mechanism. Renaming the referent of a rule
is not amending the rule. The day `accept` produces real ratification commits,
the same operation will require superseding — not because the nature of the act
will have changed, but because an anchored hash does not get recomputed without
authorisation, and that is precisely what ADR-6b3f19e08a24 exists to make
visible.
