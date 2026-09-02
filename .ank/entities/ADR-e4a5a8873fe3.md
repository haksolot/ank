---
id: ADR-e4a5a8873fe3
type: adr
slug: the-skill-system-gains-tdd-and-diagnose-and-meth
title: The skill system gains tdd and diagnose, and method stays policy rather than dispatch
created: 2026-09-02T13:55:04Z
author: claude-code/fable-5
status: proposed
scope:
  - skill/**
  - crates/ank-cli/tests/skill.rs
  - .claude-plugin/**
  - docs/**
constraint: |
  The skill system is plural. skill/SKILL.md is the contract: the rules and why, loaded by every agent. Sibling skills live under skill/, one directory per skill, each teaching one activity: plan, drift, loop, tdd, diagnose. A sibling states that the contract applies and never restates it; a rule written twice is the drift this system exists to catch.
  
  tdd teaches the red-green loop against a task's frozen criterion and names the anti-patterns it forbids; diagnose teaches the debugging loop -- reproduce, minimise, hypothesise, instrument, fix, regression-test -- for a task whose criterion names a defect. Both are policy for a moment, never dispatch: no verb enforces a method, no verifier inspects the route, and done keeps measuring the tree rather than the process that produced it.
  
  No skill's content is frozen by revision hash. Content evolves under normal review. Three anchors survive: every skill declares metadata.revision as the hash of its own body, recomputed by test; the binary names the contract skill's revision it was built alongside; and accept stays described and never invited, in any skill that mentions it.
  
  The frontmatter description is the only part every session pays for. It names the activity that should trigger the load and nothing else, and it does not grow without a measurement. Bodies stay within 180 lines and 1500 words per skill.
supersedes: ADR-91b77f036884
schema: 4
version: 1
---

## Context

An external comparison (2026-09-01) set ank against Matt Pocock's skills, a
process-first system where the agent is guided while it works: interview, spec,
tickets, TDD, review. Most of what it proposed for ank is already refused by
ratified decisions -- a glossary kind anticipates (SPEC-1d5b), tracker
write-back is refused outright (SPEC-1d5b), method-checking verifiers grade the
route where done grades the tree, and process verbs were rejected by
ADR-91b77f036884 itself: judgment is not dispatch.

Two proposals survive confrontation with the corpus, and both are activities in
the exact sense ADR-91b77f036884 built the plural skill system for: a policy
for a moment, carried by a sibling, triggering on its own description.

- TDD as taught method. The loop skill consumes tasks but says nothing about
  how an implementation should meet a frozen criterion. Red-green against the
  criterion, with the documented anti-patterns (tautological tests, horizontal
  slicing, testing the implementation rather than the behaviour), is a policy
  no existing sibling carries and the contract cannot absorb.
- Diagnosis as taught method. A task whose criterion names a defect invites
  patch-first guessing. The disciplined loop -- reproduce, minimise,
  hypothesise, instrument, fix, close with a regression test -- is likewise an
  activity, not a rule for every session.

The superseded constraint enumerated the siblings closed: plan, drift, loop.
This decision reopens the list to admit exactly two, and restates everything
else unchanged.

## What changes

Two directories join skill/: skill/tdd/ and skill/diagnose/. Each opens by
stating the contract applies, adds only its policy, declares its revision,
and stays within the ceiling. Everything else in the superseded constraint
survives word for word.

## Rejected

- A tdd-seam verifier reading git history to check the test preceded the
  implementation. It grades the route, not the tree; ank commits nothing but
  accept, so commit granularity is not even a fact ank controls; and an agent
  graded on its process learns to fake the process.
- ank tdd and ank diagnose as verbs. Rejected by the superseded decision in
  the same words: the CLI stays a set of primitives the skills reach for.
- A glossary kind and pre-agreed seams in ADRs. Both anticipate; the kind
  registry makes a kind cheap, never a good idea unmeasured.

## Consequences

tests/skill.rs gains the two sibling anchors. .claude-plugin/plugin.json
lists six directories. The npm and pi channels ship six skills; tasks carry
the siblings and the sweep of citations this supersession requires before it
can be accepted.
