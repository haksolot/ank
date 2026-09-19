---
id: LOG-c0a7e7297dbb
type: log
title: Verification, since the criterion asks for identity rather than for a green suite. ank check on
created: 2026-08-24T17:34:32Z
author: claude-code/opus-5-history-walk
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
about: TASK-0515cfe21421
seq: 2
schema: 4
version: 1
---

 this corpus prints byte for byte the same 106 dead-scope findings and the same 130 explanation lines before and after, compared with cmp on the release build of each. The only line that ever differed was a claim ref the first run pruned, which the second run no longer had to prune.

Under the answer, the four readers agree path by path. A probe held wide and narrow histories side by side and asserted rename_of, deletion_of, directory_rename_of and deletions_under equal for all 21 asked paths, seven times. It is a probe and not a test: it takes 4 seconds and pins this repository's history rather than a rule, and the rule it would have pinned is already carried by a_scope_deleted_by_a_commit_is_a_signal_and_a_scope_git_never_knew_is_a_fault, which drives the binary over three dead scopes in three directories, one of them a path git never knew.

What is pinned in tests is the reduction itself: the ancestor swallowing its descendants without swallowing a-b, which sorts between a and a/c and would otherwise be dropped; the literal pathspec magic, since a scope that never reduced to a directory names a path git never had; and the command-line budget, past which the walk is wide again rather than refused by the operating system at 32767 characters.
