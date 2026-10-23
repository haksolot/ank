---
id: LOG-e20abafc42dc
type: log
title: "Measured after the change, same lab, GIT_TRACE2_EVENT top-level processes. Origin only, --once: 9 ="
created: 2026-10-23T17:20:01Z
author: claude-code/opus-5.5+w451a
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
  - docs/watch.md
  - crates/ank-cli/src/cli.rs
about: TASK-451aaa32043f
seq: 4
schema: 4
version: 1
---

 the 8 of 2b7c8957 plus one startup probe 'config --get-regexp ^remote\.(origin|ank)\.url$' (the resolver of git.rs); the cycle (config --get remote.origin.url, fetch origin +refs/ank/claims/*:refs/ank/watch/origin/claims/*, for-each-ref refs/ank/watch/origin/) and the 4 warm processes are argv-identical, diffed. Over a 3.5s run at fetch: 1s: before 4 cycles x 3, after 4 cycles x 3 identical plus exactly 1 get-regexp, so the probe is per start and not per cycle. Origin+ank: the cycle is config --get remote.ank.url, fetch ank +refs/ank/claims/*:refs/ank/watch/ank/claims/*, for-each-ref refs/ank/watch/ank/; no process names origin; mirror holds refs/ank/watch/ank/claims/<id> only, local refs/ank/claims stays empty, and status of the second clone lists the claim held by first (before the change: 'no claim by another agent').
