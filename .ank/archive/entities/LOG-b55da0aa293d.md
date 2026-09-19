---
id: LOG-b55da0aa293d
type: log
title: amend --scope takes the glob bare; a leading + is written into the scope literally rather than read
created: 2026-08-25T04:02:18Z
author: claude-code/opus-5+adopt-prompts
scope:
  - install.sh
  - install.ps1
  - docs/getting-started.md
  - crates/ank-cli/tests/adopt.rs
about: TASK-567084d21d2b
seq: 3
schema: 4
version: 1
---

 as an add. Hit it once, repaired with --drop-scope. That is TASK-86ba's defect, already open and not mine to fix here.
