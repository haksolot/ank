---
id: LOG-a40ef13046dc
type: log
title: "Fix: output_with_stdin writes stdin from a scoped thread while wait_with_output reads"
created: 2026-10-17T18:08:20Z
author: claude-stdin-deadlock
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/tests/**
about: TASK-f2b8aedcd9a4
seq: 3
schema: 4
version: 1
---

 stdout/stderr. Regression test crates/ank-cli/tests/done_stdin_pipe.rs (4000 files at 40-byte paths, 4004 tracked, verifier inputs ["**"], ank done through the binary under a 120s deadline of its own): red before the fix (hung, killed at 120.32s), green after (0.62s). Release binary rerun of the repro script: N=3194, 4000, 8000 all exit 0 under timeout 20. Other write_all-before-read site checked: claim.rs hash-object -w --stdin emits one hash after reading all input, so it cannot fill stdout first; left alone (outside scope).
