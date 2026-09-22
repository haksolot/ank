---
id: LOG-6bf44f085ad1
type: log
title: "Measured before the fix, 2026-09-22, ank mcp --repo . on this tree: tools/call ank_find"
created: 2026-09-22T17:43:19Z
author: claude-code/opus-5+7cb7
scope:
  - crates/ank-mcp/**
about: TASK-7cb77bc870b1
seq: 1
schema: 4
version: 1
---

 {worktree:/tmp} came back result with total 738, the call ran. tools/list carried 0 occurrences of "worktree" (and ank_status too): no tool advertises it already, so adding the row changes no schema -- the question in the body is moot, hiding is what it already did. New stdio test worktree_is_refused_by_name_and_the_three_keep_their_reasons red: reply was a result, isError, 'error[1]: --worktree /somewhere/else is not a directory' from the CLI. 3 of 4 in tests/withheld.rs passed.
