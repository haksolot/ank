---
id: ADR-d3a8dcf38817
type: adr
slug: english-only
title: English is the only language of the project
created: 2026-07-29T14:10:00Z
status: accepted
scope:
  - crates/**
  - docs/**
  - skill/**
  - .ank/**
  - README.md
  - CLAUDE.md
  - AGENTS.md
constraint: |
  English is the only language of the project. Specification, README, guides,
  identifiers, comments, CLI output, error messages, entity titles, bodies,
  slugs and log entries are all in English. Non-English text is a finding, not
  a matter of taste. The single exception is a string whose meaning is its
  literal value: an external proof reference, a quoted third-party message, or
  a fixture asserting a byte sequence.
schema: 1
version: 1
---

The tool's whole claim is agnosticism — any agent, any harness, any host. A
French specification contradicts that claim at the first line a contributor
reads, and the specification is the source of truth (ADR-63b59c5c26f7): a
contributor who cannot read it cannot implement the format, which is the one
thing the project asks other people to do.

The cost is asymmetric and that is what settles it. Writing English costs the
author a little; reading French costs every reader, every time, forever. The
same arithmetic already froze the agent surface at seven verbs
(ADR-2f8a61c04b7d) and shortened the name to `ank` (ADR-85e6bbb195b8).

**CLI output is included, and it is the part that matters most.** Error messages
are self-correcting by design (§4) — they carry the exact command to run next.
An agent parses them, a contributor greps them, and a test asserts them. A
French `error[7]` is a dead end for everyone outside one language.

**Past log entries and frozen criteria have been translated in place.** This is
the same line ADR-85e6bbb195b8 drew for the rename, and it holds for the same
reason: a faithful translation does not amend substance. `done_criteria` says
what it said, log entries record what they recorded. Nothing anchored is
recomputed, because nothing is anchored yet — the `proof` entries of this repo's
`done` tasks are of type `commit` and carry no `criteria` hash, the claims were
manual, and `allowed_signers` is empty.

That last sentence is the whole reason this is cheap today and will not be
tomorrow. Once `accept` produces real ratification commits and `claim` records
real criteria hashes, translating a `constraint` or a `done_criteria` will
require superseding the entity, not editing it — not because the act changes
nature, but because a hash does not move without authorisation. That is exactly
what ADR-6b3f19e08a24 exists to make visible.

The proof reference `ci://haksolot/ankor/runs/30324400136` on TASK-ca4714f5c719
stays as it is. It is not language, it locates an artifact on a third-party
host, and it is a forbidden write on a `done` task beyond appending (§3).
