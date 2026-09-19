---
id: LOG-b608f94d0cfb
type: log
title: "the busy timeout alone does nothing, and measuring is what said so: with it set and the refresh"
created: 2026-08-15T18:41:22Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/index.rs
about: TASK-e9dfaf187a1b
seq: 1
schema: 3
version: 1
---

 still on a deferred transaction, eight of twelve readers failed in under a second. SQLite refuses a read-to-write upgrade with 'database is locked' without ever calling the busy handler, to avoid deadlocking two upgraders, so the timeout is not consulted on the one path that needs it. BEGIN IMMEDIATE takes the write lock up front and the wait applies.
