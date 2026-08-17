---
id: TASK-edddce600c4f
type: task
slug: the-npm-packages-ship-the-licence-file-they-decl
title: The npm packages ship the licence file they declare
created: 2026-08-17T20:03:18Z
author: claude-code/2.1.233+exposition
status: open
scope:
  - .github/workflows/release.yml
  - npm/**
blocked_by: []
done_criteria: |
  Each published npm package contains the licence text its package.json's files array declares. The npm smoke job proves it on the package it installs rather than on the directory before publish: a package whose files list names LICENSE and whose tarball lacks it fails the job. cargo test is green.
criteria_by: creator
schema: 3
version: 1
---

Found while relicensing the tree (TASK-47beb64fd204). All four
`npm/*/package.json` list `LICENSE` in their `files` array, and nothing puts one
there: `release.yml:149` copies `README.md` and `LICENSE` into `dist/<name>/`,
which is the release archives, not the npm package directories. npm ignores a
`files` entry that names nothing, silently, so the packages publish with
`"license": "Apache-2.0"` in the metadata and no text beside it.

Harmless until somebody vendors the package and looks for terms, which is
exactly the reader a permissive licence exists for (ADR-9f03438f5422).

**The check belongs after packing, not before.** A job that asserts the file is
in `npm/ank/` proves the copy ran; what needs proving is that the tarball a user
installs carries it, which is what `npm pack` and the smoke job already have in
hand. That is the same distinction the rest of this repository draws between
testing a function and testing the process.
