---
id: ADR-6fd69efb629c
type: adr
slug: the-machine-surface-is-a-versioned-contract-gene
title: The machine surface is a versioned contract, generated from one table
created: 2026-08-17T05:12:40Z
author: claude-code/2.1.233+integration-contract
status: accepted
scope:
  - crates/ank-cli/**
constraint: |
  Every --json document is produced by one writer and one escaper, and carries the contract version it was written against. The verb table, the exit codes and the output shape of every verb live in one crate that every surface consumes, so no surface can describe a verb the CLI does not dispatch, and none can drift from it. `ank help --json` is that description: verbs, flags, refusals, exit codes and output shapes, generated and never written by hand. A golden fixture pins the output of every verb, and a shape that changes without its fixture changing is a failing test. Within a contract version a document may gain a field and may never lose, rename or retype one; anything else bumps the version. No document carries a key whose name depends on the data.
ratified: 274d910f127a
schema: 3
version: 4
---

## Why

`--json` is already available on every verb without exception, and §4 calls that
a spec invariant rather than a convenience. The transport is guaranteed too: one
line, stdout only, never coloured, no stray output — warnings were moved to
stderr precisely so a caller's parser keeps reading what it already read.

The payload has none of those guarantees, and the gap is not theoretical:

- four different string escapers, in `cli.rs`, `commands.rs`, `context.rs` and
  `human.rs`, which do not agree — one of them escapes a tab and the others do
  not;
- every document built by `format!`, so the shape of a verb's output exists only
  in the string that produces it;
- no version field anywhere, so a consumer cannot tell which contract it is
  holding;
- `accept` emits a top-level key whose name is the kind of the entity accepted,
  which no typed client can bind;
- `review` exits 8 while its refusals are empty and no document says so.

None of that matters while the only consumer is an agent reading prose. All of
it matters the moment something is written against it, because the first
integration written against a shape is what freezes that shape.

## What this decides, and what it does not

It decides that the description is generated rather than maintained. A verb
table written twice is a verb table that will disagree with itself, and the
disagreement lands on whoever wrote the client, days later, as a bug they cannot
see from their side.

It does not decide what any surface beyond the CLI looks like. It is the
condition every such surface rests on, and it is deliberately settled first, on
its own, so that a later decision about a protocol argues about the protocol and
not about whether the thing it exposes has a shape.

## The cost, stated

The envelope is a breaking change to every document. This is the moment to spend
it: the version is 0.3.0, and no consumer outside this repository exists yet.
Spending it later means spending it on somebody.
