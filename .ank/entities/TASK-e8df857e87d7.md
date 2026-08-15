---
id: TASK-e8df857e87d7
type: task
slug: the-core-admits-the-spec-and-log-kinds
title: The core admits the spec and log kinds
created: 2026-08-15T06:56:22Z
author: claude-code/opus-5
status: in_progress
scope:
  - crates/ank-core/**
  - crates/ank-cli/**
blocked_by: [TASK-1d47cc52c42d]
done_criteria: |
  crates/ank-core declares the kinds spec and log in its registry, each with an entity struct, a field table and canonical serialisation, and EntityKind, Entity and the parser admit them. A spec declares no constraint field; a log entry names the entity it is about. A golden fixture per new kind round-trips byte for byte under crates/ank-core/tests/golden.rs, and an invalid fixture per new kind is refused naming what is wrong. cargo test --workspace passes and cargo fmt --check is clean.
criteria_by: creator
schema: 3
version: 4
---

The registry exists so that adding a kind is a table entry rather than a second
parser, and this is the first time anybody uses it for that. ADR-c9f9d0d6f05d is
explicit that the cost of a kind is a row, a golden fixture and a section of the
specification — never a second serializer, a second parser branch, and never a
second directory. Hold it to that: if this task grows a `match` arm per kind
somewhere the registry should have answered, the registry is what is wrong.

Two kinds, and they differ from `adr` in one field each.

**`spec` declares no `constraint`.** That absence is the whole justification for
the kind. A specification describes; an ADR binds. An entity that did both would
be an ADR with an unbounded constraint, which is the ceiling problem restated
rather than solved, and the ceiling is already exceeded by eight tasks.

**A `log` entry names the entity it is about.** Today that association is
arithmetic on the id, and it becomes a field. This is what ADR-c9f9's "the
address is computed, never looked up" costs, and it is accepted deliberately in
ADR-25f977377fa0 rather than discovered here.

**The enum indexes the registry by declaration order.** `EntityKind` is a closed
enum whose variants index `KINDS` positionally, and a test asserts the two agree.
Adding rows in one place and variants in another, in a different order, is the
failure this arrangement is built to catch — let it catch you rather than
reasoning about it.

**Goldens are the specification made executable.** The round trip is byte for
byte on canonical form, so a valid fixture per kind proves the serialiser and the
parser agree, and an invalid fixture per kind proves the refusal names what is
wrong rather than failing vaguely. A kind with no invalid fixture is a kind whose
strictness is untested, and the existing `bad-status.md` is the shape to copy.

Nothing in the CLI moves under this task. `ank new spec` and the log verb come
after, in TASK-3e68786fa443 and TASK-df9c6d46e8ef, and keeping the core change
alone is what lets those two proceed in parallel with anything else.
