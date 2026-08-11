---
id: ADR-c9f9d0d6f05d
type: adr
slug: entities-live-in-one-flat-directory-and-kinds-ar
title: Entities live in one flat directory, and kinds are a registry
created: 2026-08-11T22:16:34Z
author: claude-code@sean-laptop
status: accepted
scope:
  - crates/ank-core/**
  - crates/ank-cli/src/store.rs
  - docs/**
constraint: |
  Entities live in one flat directory, .ank/entities/<ID>.md. The file name is the
  id, and the id prefix already carries the kind. No directory means anything, and
  none is added to make one mean something.
  
  Entity kinds are a registry rather than a closed enum repeated at every layer.
  One table declares, per kind: the name, the id prefix, which fields are required
  and which optional, and the canonical field order. Adding a kind is an entry in
  that table, a golden fixture, and a specification section -- never a second
  serializer, a second parser branch, or a second directory.
  
  Strictness does not move. Unknown fields stay rejected inside a known kind, and
  an unknown kind is rejected by name. The registry makes a kind cheap to add; it
  never makes the format permissive.
  
  A reader accepts the previous layout, .ank/tasks/ and .ank/adr/, and a writer
  never produces it. A corpus still in the old layout is a check finding naming the
  command that moves it.
ratified: 353381bd34b6
schema: 2
version: 2
---

## Context

`docs/format.md` states the layout is flat "deliberately: attachment happens
through the `scope` field, not through location", and section 6 gives the reason
— a tree mirroring the code would force a single parent and break at the first
refactor. The layout then contradicts the sentence by splitting `.ank/tasks/`
from `.ank/adr/`.

The split carries no information. The kind is in the id prefix, which is in the
file name, which `store.rs` already cross-checks against the id inside the file.
The directory is a third statement of the same fact, and the only thing a third
copy can do is disagree with the first two.

Underneath, the same redundancy is in the code. The kind is a closed enum in
`id.rs`, a closed sum type in `model.rs`, and a match in `parse.rs`, with two
straight-line serializers that differ only in which fields they emit and in what
order. Nothing there is wrong; it is simply written three times, so it costs
three edits and a specification revision to say one new thing.

## What this is not

**This adds no entity kind.** `TASK-00660963bcce` records the rejection of a
`spec` kind and the reasoning holds: a document that binds is an ADR, a document
that describes is not what ank stores, and section 10's "do not anticipate" row
stays exactly where it is. What changes is the price of the decision if it is
ever taken — a table entry rather than a revision — so that the answer to "should
this be a kind?" stops being decided by how expensive the answer is.

**This does not import OKF's extensibility.** The Open Knowledge Format makes
`type` the only required field, preserves unknown keys, and instructs consumers
never to reject a document for what it lacks. That is the right trade for sharing
knowledge between organisations and the wrong one here: ank's value is that a
criterion is frozen and a proof is anchored, and neither survives a reader that
must accept whatever it is handed. The shape is borrowed — one document model,
kinds declared in a table — and the permissiveness is not.

## Rejected

**A configurable layout.** It was asked for and it is the one thing to refuse. A
layout read from `config.yml` means every third-party reader must parse the
configuration before it can find a file, which is exactly the coupling the format
exists to avoid. `crates/ank-core/tests/golden/` would stop being a suite anybody
can run against a directory. Flat and fixed, or the format is not a format.

**Keeping the split and merely deduplicating the code.** It removes the three
copies in the source and leaves the fourth on disk, which is the one a third-party
tool sees.

**A migration verb.** A twenty-second verb to run once is a permanent surface for
a temporary problem. The reader accepting both layouts for one release does the
same work with no surface at all, and `check` is already the place that reports a
corpus needing attention.

## Consequences

The corpus in this repository moves in one commit, 154 files, with `ank check`
run either side of it. Nobody outside this repository has a corpus yet, which is
what makes the window for this change now rather than never.

`index.db` is derived and carries `path` already, so it rebuilds and nothing
migrates.

The `.gitattributes` entry pinning `.ank/** text eol=lf` covers the new directory
without change, since it was written against the whole tree rather than against
the two subdirectories.
