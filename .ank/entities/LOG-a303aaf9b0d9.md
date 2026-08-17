---
id: LOG-a303aaf9b0d9
type: log
title: "not verified, and it is the one clause I cannot reach from here: whether the picker is granted to a"
created: 2026-08-17T07:14:40Z
author: claude-code/2.1.233+integration-contract
scope:
  - viewer/**
about: TASK-34d27790dba9
seq: 2
schema: 3
version: 1
---

 page opened from a file:// URL. ADR-bcb18aecb7e1 says no build server at view time, which implies opening the file from disk, and the browser automation available to me refuses the file:// scheme outright -- so every browser run above went through http://localhost. The page now translates a SecurityError on a file:// origin into the one sentence that distinguishes 'this browser will not hand a local page a directory' from 'this repository is unreadable'. Settle it by double-clicking viewer/index.html and pressing Open.
