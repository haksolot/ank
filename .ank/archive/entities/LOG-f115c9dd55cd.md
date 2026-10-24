---
id: LOG-f115c9dd55cd
type: log
title: "Runner record, job 117369363083 (windows-latest, run 39023341364 attempt 1): the failing test's"
created: 2026-10-17T16:55:46Z
author: claude-code/opus-5.5+78ef
scope:
  - crates/ank-cli/src/human.rs
about: TASK-78ef9b4f7cb2
seq: 2
schema: 4
version: 1
---

 stderr carries 'warning: re-init: ignored --initial-branch=main' just before the panic at human.rs:9502 'ssh-keygen: ' with nothing after. git init only says re-init when .git already exists, so Temp::new landed on a directory that already held a repository: ank-human-<pid>-<seq> is keyed on the process id, nextest runs one process per test, Windows reuses PIDs quickly, and a leftover from an earlier process (Drop's remove_dir_all is best-effort and silent) is then reused, signing-key included.
