---
id: LOG-f42a7c7af917
type: log
title: "the durable lesson, since the same trap is waiting for the next cfg(unix) block: this machine"
created: 2026-07-31T16:53Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/store.rs
about: TASK-dc87e0ecfb6c
seq: 3
schema: 3
version: 1
---

 cannot compile the unix branch at all. `cargo check --target x86_64-unknown-linux-gnu` gets as far as libsqlite3-sys and stops, because the bundled SQLite needs a Linux C toolchain that is not here. So CI is the only compiler the unix code has. Two consequences taken now: cfg(unix) blocks stay as small as possible, and `acquire_as` gained a test that runs on every platform, so at least the seam's signature and trait bounds are compiled by the host before a push. That test is what would have caught this one.
