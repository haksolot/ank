---
id: LOG-f120fd4ac130
type: log
title: "reproduced through the binary and the library. Fixture: .ank/entities/TASK-0123456789ab.md, 129"
created: 2026-09-20T17:44:03Z
author: claude-code/opus-5+ccb7
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
about: TASK-ccb787dc807f
seq: 3
schema: 4
version: 1
---

 bytes, od shows last byte '-' (no final newline), opening '---\n' present. 'ank check' prints: error: TASK-0123456789ab.md: missing frontmatter: the file must start with '---' and 'ank show TASK-0123' the same, both exit 0/1. Appending one byte \n makes it parse (the remaining fault becomes 'non-canonical form'), so the whole difference is the trailing newline.
