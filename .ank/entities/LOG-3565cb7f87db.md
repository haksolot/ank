---
id: LOG-3565cb7f87db
type: log
title: The criterion is measured through the binary and the ordering lives in one file, so the measurement
created: 2026-08-28T20:09:28Z
author: claude-code/opus-5+alive-first
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/tests/ordering.rs
about: TASK-b5185df7aa44
seq: 1
schema: 4
version: 1
---

 needs a second: scope widened by exactly crates/ank-tui/tests/ordering.rs, a file that does not exist yet and that no live claim can reach. Not tests/** -- TASK-252bf02de218 needed the whole directory because it rewrote the suite that asserted four panels; this adds a suite and rewrites none, and three agents are in this crate. src/model.rs sorts what find handed over: three bands (alive, waiting, the rest) and the instant inside each, with no key mentioning an identifier.
