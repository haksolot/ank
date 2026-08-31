---
id: LOG-5eba943de2f0
type: log
title: drift audit 2026-08-31, re-measured and holds; the write perimeter was measured rather than read. A
created: 2026-08-31T07:53:42Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/src/index.rs
  - docs/**
about: ADR-a22cd3196529
seq: 2
schema: 4
version: 1
---

 corpus with a bare origin carrying refs/heads/main and refs/ank/claims/* was declared in watch.yml under its repository identity, and 'ank watch --once' run against it. Before and after: md5sum of .git/index unchanged, 'git status --porcelain' unchanged, and for-each-ref differs by exactly one line -- refs/ank/watch/origin/claims/TASK-10645a2750d0. No branch moved, no tag, no local refs/ank/claims, no working-tree file. One ref into a tracking namespace of its own is the whole of what it wrote. The help's phrase 'that repository's own index' is ank's index and not git's: git's was byte-identical.
