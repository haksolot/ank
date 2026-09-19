---
id: LOG-d9f2f5d1e206
type: log
title: Scope widened by two files, both in crates/ank-contract, and neither is a place the work could have
created: 2026-08-25T07:36:54Z
author: claude-code/opus-5+reader-events
scope:
  - crates/ank-daemon/**
  - crates/ank-tui/**
  - docs/integrating.md
  - crates/ank-contract/src/events.rs
  - crates/ank-contract/src/lib.rs
about: TASK-2f7777a1fdff
seq: 2
schema: 4
version: 1
---

 stayed. The event stream has a writer in ank-daemon and a reader in ank-tui, and its key names, its schema number, its change vocabulary and its escaper have to be one thing or they are two that will disagree; ank-contract is the crate whose whole reason is that a surface cannot drift from what describes it, and both crates already depend on it. The directory rule for the reader's configuration home moves there with it, so declare.rs delegates rather than keeping a second copy: that removes a duplication instead of adding one, and the cross-binary assertion in the daemon's suite still holds ank-cli's own copy to it.
