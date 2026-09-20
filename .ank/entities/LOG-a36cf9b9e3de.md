---
id: LOG-a36cf9b9e3de
type: log
title: "normalised on first rewrite, measured in bytes (ADR-63b5): 'ank new task' wrote a canonical file of"
created: 2026-09-20T17:47:32Z
author: claude-code/opus-5+ccb7
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
about: TASK-ccb787dc807f
seq: 6
schema: 4
version: 1
---

 223 bytes ending '...version: 1\n---\n'; stripping the final byte gave 222 ending '---'. 'ank check' then reported 'non-canonical form (round-trip differs)', a fault it can only reach because the file now parses. 'ank amend --scope docs/**' rewrote it to 235 bytes ending '---\n' and 'ank check' came back ok. So the missing newline is read, reported as non-canonical, and put back on the first write -- the same route CRLF takes.
