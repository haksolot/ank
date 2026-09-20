---
id: LOG-d4e4edea637e
type: log
title: "cause located by measurement, not by reading: instrumented split_frontmatter's two failure points"
created: 2026-09-20T17:44:13Z
author: claude-code/opus-5+ccb7
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
about: TASK-ccb787dc807f
seq: 4
schema: 4
version: 1
---

 on the fixture. starts_with '---\n' = true, strip_prefix succeeded (rest len 125), rest.find("\n---\n") = None, rest.ends_with("\n---") = true. So the opening delimiter is found and it is the closing search that fails; both ok_or arms return the same Error::MissingFrontmatter, whose text names only the opening. One error conflating two conditions.
