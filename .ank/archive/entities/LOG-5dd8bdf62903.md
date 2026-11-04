---
id: LOG-5dd8bdf62903
type: log
title: "Reproduced without the daemon (scratchpad repro.sh: init, find, then ank new task with a python"
created: 2026-11-04T07:47:59Z
author: claude-code/opus-5.5+task-watch-red
scope:
  - crates/ank-cli/tests/watch.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-daemon/**
about: TASK-161fe3c2c7da
seq: 1
schema: 4
version: 1
---

 read-only sqlite connection held open on index.db.N during new and exited by os._exit, then ank find --status open --json). Head b35de33c: hold=no -> main file sha changes at new (d6d9ea->7972fd), wal=0; hold=yes -> main file unchanged after new AND after the find (d6d9ea both), WAL left at 98912 bytes. Same binary with only the scope_hint open (commands.rs:344) replaced by ScopeHint::default(): hold=no new->15b164, find->0e8a38; hold=yes new unchanged d6d9ea, find->4c3d9f, WAL absent. Mechanism: try_open sets NO_CKPT_ON_CLOSE and Drop re-enables the checkpoint only on a connection that wrote. Since #753 new's second open indexes the creation entry new wrote after its first open, so new leaves nothing for the next reader to write; when any other connection is open as new closes (the daemon's find, fired by the fingerprint change from store.create), new's close checkpoint is skipped, the find writes nothing and closes without checkpointing, and the frames stay in the WAL. The test reads the main file's bytes, so it waits on a checkpoint nobody runs. The data is correct (it is in the WAL); the oracle is what broke.
