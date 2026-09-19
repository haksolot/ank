---
id: ADR-bcb18aecb7e1
type: adr
slug: a-read-only-local-viewer-opens-ank-in-the-browse
title: A read-only local viewer opens .ank in the browser, without a server
created: 2026-08-05T04:04:29Z
author: seanl@sean-laptop
status: superseded
scope:
  - docs/**
constraint: |
  The read-only web view deferred by the specification, section 10, is reopened
  with this shape and no other: a single self-contained HTML page, no backend,
  no network, no account, no build server at view time. It reads .ank/ live
  through the browser File System Access API, read-only — it never writes a
  file, a ref, or a claim. It parses the format as the specification defines it
  and holds no state of its own. It is a third-party reader in the sense of
  ADR-01b6dd05f0db, which constrains agents, not tools: nothing in it goes
  through the CLI, and nothing in it is available to an agent as a route around
  the CLI.
ratified: 2a48845ea581
schema: 2
version: 4
---

## Context

The specification, section 10, deferred a read-only web view "to reopen only if
non-developers must read the board". That condition is now met: the board needs
to be readable by someone who will not learn a CLI, and needs views a terminal
does not give — a browsable DAG, filters, a board by status.

## Decision

Live over snapshot: an export command would freeze the board at export time and
grow the CLI surface; a page that opens the repository folder through the File
System Access API reads the same files the CLI reads, at the moment of viewing.
The known cost is browser support — the File System Access API means
Chromium-based browsers today — and it is accepted for a local developer-side
tool.

The GPL note in the README already states that third-party tools reading .ank/
are not derivative works; this viewer simply lives in-tree, under viewer/, and
is held to the same reading the licence promises others.

## Sequencing

This ADR is the framing decision only. Implementation tasks are created after
acceptance, blocked on nothing else; none exist yet by design.
