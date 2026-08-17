---
id: ADR-894defc26f3d
type: adr
slug: the-reader-lives-outside-this-repository-and-thi
title: The reader lives outside this repository, and this one keeps the surface it rests on
created: 2026-08-17T21:48:20Z
author: claude-code/2.1.233+exposition
status: accepted
scope:
  - docs/**
  - crates/ank-cli/**
constraint: |
  No read-only viewer lives in this repository. The web view specification section
  10 deferred and ADR-bcb18aecb7e1 reopened is withdrawn from this tree: nothing
  under a viewer/ directory, no HTML page, no browser reader, and no task that
  would produce one. What this repository keeps is the surface such a reader rests
  on -- `ank help --json`, the exit codes, the contract version, the corpus
  identity and the golden suites -- and that surface is public, documented in
  docs/integrating.md, and permissively licensed.
  
  A reader may exist. It ships from its own repository, on the same terms as any
  other third-party tool, and this decision says nothing about its shape.
supersedes: ADR-bcb18aecb7e1
ratified: f79b47a4aeaa
schema: 3
version: 2
---

## What this supersedes, and what it does not call wrong

ADR-bcb18aecb7e1 reopened the read-only web view that specification section 10
had deferred "to reopen only if non-developers must read the board", and it
reopened it with a shape argued in detail: one self-contained page, no backend,
no network, the File System Access API, read-only, holding no state of its own.

**That decision was not a mistake, and this does not treat it as one.** Its
condition was real and its shape was the right shape for it. What has changed is
where such a reader belongs, and a record that simply vanished would leave a
reader in a year unable to see that the question was ever asked.

## Why it leaves

**A reader is a tool, and the tool this repository builds is the CLI.** The
viewer was accepted as a third-party reader in the sense of ADR-01b6dd05f0db --
"which constrains agents, not tools" -- and then placed in-tree, which is the
one place a third-party reader cannot be. It made this repository the home of two
products with different audiences, different release cadences and, as it turned
out, different implementations of the same reading: an attempt at it had to
reimplement packed-ref lookup and packfile delta chains in JavaScript, and then
reconcile the result against the CLI to know it was right.

**What it needed did not exist, and now does.** That reimplementation was
necessary because nothing stated the contract a reader binds to. It is stated
now: `ank help --json` describes every verb, its refusals with their codes and
the shape of what comes back (TASK-155e98c184ed); every document carries the
contract version (ADR-6fd69efb629c); a corpus has an identity that survives being
moved or cloned (ADR-621a7fd96ce1); a reader is never refused by contention on
the index (TASK-4111dfae8a87); `docs/integrating.md` says all of it to somebody
who has never seen this tree (TASK-af4a6db95aab); and the whole of it is
Apache-2.0 (ADR-9f03438f5422). A reader outside this repository is now a
supported thing rather than a fork.

**And the split cost real attention.** TASK-34d27790dba9 was written, built on a
branch, closed, reopened by the expiry of the block that parked it, and closed
again -- and closing it at all required correcting how `check` judges a closure
(TASK-4c031f7b44ed), because `viewer/**` names no file here and never will. That
is what an in-tree product with no home in the tree costs.

## What is not decided here

Nothing about whether a reader should exist, or what it should look like. The
shape ADR-bcb18aecb7e1 argued for may well be the right one, and its reasoning is
preserved by the supersession rather than deleted by it. This decides one thing:
not here.
