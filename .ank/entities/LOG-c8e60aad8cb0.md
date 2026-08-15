---
id: LOG-c8e60aad8cb0
type: log
title: Implemented and falsified. git::rename_of is rev-list -1 HEAD -- <path> then diff-tree -M -r -z
created: 2026-08-13T04:22:28Z
author: claude-agent-b
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-1e79ff3738df
schema: 3
version: 1
---

 --name-status --no-commit-id on that commit; diff-tree was missing from the PLUMBING list and is added with the criterion ADR-9307 asks for. Deliberately no --full-history here, unlike ratification_at: there the target is one commit identified by subject and simplification would lose the anchor, here the target is the change itself, and --full-history would keep the merges that merely carried it -- a merge is a commit diff-tree prints nothing for, so asking for more history would answer less often. The walk is skipped for a glob carrying a wildcard: git has no answer for where src/** went, and the single-file entry is the common case. Finding gained a note field rather than folding the proposal into message, because review filters findings on the opening of message and a note carries no severity of its own. Rendered with the LAST and CLEAR glyphs already declared in section 4, so no character is added to the structure alphabet. Falsified by making the walk return nothing: the five tests that assert it fail, and outside_a_repository_the_rename_walk_is_skipped_without_a_word stays green, which is what it should do. The renamed-ADR test spends the proposed command instead of matching it -- split back into argv, handed to the binary, must exit 0 and leave the scope alive.
