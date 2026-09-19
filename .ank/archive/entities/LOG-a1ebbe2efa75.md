---
id: LOG-a1ebbe2efa75
type: log
title: git rev-list --no-walk is ignored the moment a range is given, so --no-walk <shas> --not --all
created: 2026-08-14T21:16:42Z
author: claude-code/f113
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-f113addd8f40
seq: 0
schema: 3
version: 1
---

 prints the ancestors of an unreachable commit too, and --ignore-missing drops a reference that resolves to nothing without a word, which is the shallow case the criterion asks about; measured on git 2.54, so the question is asked the other way round, against one rev-list --all listing of what this clone reaches, with every recorded reference tested against it as a prefix: one process, and a reference that resolves nowhere and one a rebase detached are answered the same way, which is what a reader of another clone needs.
