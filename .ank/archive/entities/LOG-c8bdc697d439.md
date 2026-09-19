---
id: LOG-c8bdc697d439
type: log
title: Shape settled before any code. The declaration is a watch.yml beside corpora.yml under the reader's
created: 2026-08-25T02:09:18Z
author: claude-code/opus-5+daemon-watch
scope:
  - crates/ank-daemon/**
  - Cargo.toml
  - docs/integrating.md
about: TASK-f8802467b622
seq: 1
schema: 4
version: 1
---

 config home, same rule as ADR-96174f1ac2b7, keyed on the 40-hex root commit, value one path or a list of them so two worktrees of one repository are two paths under one key and therefore one watched corpus. Warming goes through the CLI: ank find --repo <corpus> --status open --json --quiet opens the index, refreshes it against the file hashes, and writes nothing else, so the daemon is not a second implementation of index.rs and never links it -- which is also what keeps this task inside crates/ank-daemon/**, since ank-cli has no library target. No file-watching crate enters the tree: the poll is a stat walk of the declared .ank/ and only that.
