---
id: LOG-7ee5a8418ff8
type: log
title: "Fix: root() creates the lock file at .owner.tmp, takes the lock there, and renames it to .owner; a"
created: 2026-10-17T16:17:07Z
author: claude-code/fable-5.1+8e79
scope:
  - crates/ank-cli/tests/scratch/mod.rs
about: TASK-8e79eca6e5de
seq: 4
schema: 4
version: 1
---

 sweep only ever opens .owner, so it finds either no file (GRACE keeps the young root) or a held lock. Measured on Linux with the same test and 4 sweepers: 0 of 1000 children failed, three runs in a row (13.9s, 13.5s, 13.6s), against 524 of 1000 on the unfixed tree in the same session. The regression test was watched red before the fix was written. macOS and Windows are measured by CI on the pull request, this session has Linux only; the Windows question is the rename of an open, locked handle, which std opens with FILE_SHARE_DELETE.
