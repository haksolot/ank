---
id: LOG-88959f9adbb0
type: log
title: that done was premature and the proof above overstated what it covered. Run 30648501580 failed to
created: 2026-07-31T16:52Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/store.rs
about: TASK-dc87e0ecfb6c
seq: 2
schema: 3
version: 1
---

 compile on macos-latest and ubuntu-latest: `Lock::acquire_as(..).unwrap_err()` needs `Lock: Debug`, and the two cfg(unix) tests that contain that call are never compiled on Windows, so `cargo test` was green here on code that does not build there. The criterion says cargo test is green; on two of three platforms it was not, and CLAUDE.md says in as many words that OS-dependent behaviour is not verified until it has run on all three. I read that rule, wrote code no local compiler could see, and closed the task on a one-platform proof anyway. Fixed by deriving Debug on Lock. A ci:// proof is appended below, and that one is the evidence the criterion actually asked for.
