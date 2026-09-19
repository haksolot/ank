---
id: LOG-0efea042f120
type: log
title: Two defects of my own, both caught by tests rather than by reading. The record framing ended each
created: 2026-08-20T18:20:35Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
about: TASK-1b3d7b61dc8f
seq: 1
schema: 3
version: 1
---

 commit with two NULs, which reads correctly until a commit has no body: %b is then empty, the record ends in three NULs, the split lands one NUL early and every ratification after the first bodyless commit disappears. an_unsigned_ratification_commit_is_refused_as_a_ratification is what caught it, answering 'no ratification' where it must answer 'ratification, unsigned' - the worse of the two, since check says nothing about the first. Replaced by three counted fields: an empty field is still a field, which a separator cannot promise. Second, the walk is memoised for the process, so a commit made after it is absent from it - accept ratifying then reading back, a test staging a second ratification. The walk is now an accelerator and not the authority: a miss falls back to the per-entity search, which asks git about the history as it stands. Also learned the hard way: ADR-b8884edcebe3 forbids porcelain and the PLUMBING list names 'log --name-status' explicitly. I claimed without running ank context on git.rs and wrote the walk as git log; rewritten as rev-list piped into diff-tree --stdin. Final: 616 git starts to 308, all targets green. Wall time is noisy on this machine, 34.5s to 48s for the same binary, which is why the criterion counts processes.
