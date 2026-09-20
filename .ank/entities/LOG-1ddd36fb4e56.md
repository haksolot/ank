---
id: LOG-1ddd36fb4e56
type: log
title: regression seen red before green, which is the point of writing it from the minimal case. With
created: 2026-09-20T17:49:51Z
author: claude-code/opus-5+ccb7
scope:
  - crates/ank-core/tests/**
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
about: TASK-ccb787dc807f
seq: 7
schema: 4
version: 1
---

 split_frontmatter reverted to the single find("\n---\n"), 5 of 31 golden tests fail: a_closing_delimiter_at_end_of_file... panics 'a closing delimiter at EOF must be read, not rejected: MissingFrontmatter' -- the failure from step one, verbatim -- and invalid_files_are_rejected_with_the_right_error panics 'unterminated-frontmatter: wrong error: MissingFrontmatter'. With the fix restored, 31/31 pass.
