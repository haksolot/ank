---
id: LOG-aa675d5fb6f0
type: log
title: "the fix is a rollback and not a reordering, and the reason is measurable: the ratification commit"
created: 2026-08-19T22:37:01Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-1dbb6e7843f1
seq: 0
schema: 3
version: 1
---

 is a commit *of* these files, so git takes what is on disk and there is no order in which the write comes second. accept now snapshots the bytes of every path it will touch, writes, commits, and puts the bytes back when any of that fails. Falsified before it was trusted: with the fix stashed and the test unchanged, the ADR comes out 'status: accepted' carrying 'ratified: 92e0d3922328' and a verified entry, which is the incident of 2026-08-18 reproduced; with the fix in place the file is identical before and after, HEAD is unmoved, the index is empty and a second accept ratifies once signing works. Two things measured on the way. The unstage cannot be git reset: git::output holds a debug_assert on the plumbing list of ADR-b8884edcebe3 and reset is porcelain, so the index is put back by staging the restored bytes instead, which is the same operation read from the other end and needs no HEAD to exist. And the lever for a commit git refuses is gpg.format=bogus, already used by a_signature_git_cannot_read_is_reported_rather_than_passed_over: it needs no key, no agent and no passphrase, and it fails the signing alone while every other git question in the fixture still answers. cargo test --workspace: 594 passed, 0 failed. cargo fmt --check clean. ank check ok.
