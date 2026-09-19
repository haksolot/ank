---
id: LOG-3df7823a9cae
type: log
title: The per-entity phase is not per-entity. Timing each probe's slowest single call apart from the rest
created: 2026-08-23T22:20:37Z
author: claude-code/opus-5-measure
scope:
  - crates/ank-cli/src/human.rs
about: TASK-756a870eb0ab
seq: 2
schema: 4
version: 1
---

 splits the loop cleanly: 400 of the 544 ms, 74 percent, is four one-shot lazy initialisations that happen to be reached from inside the loop, on the first entity that needs them. git::history 251 ms (one call, 46 percent of the whole loop). The all_ratifications git log walk 56 ms, visible as the first freeze_state. Index::open on the signature sqlite 59 ms, and gpg_config's git config spawn 34 ms, both inside the first signature_cache lookup. That is why one ADR costs 142 to 164 ms and the other 61 cost 12 ms between them, and why one task costs 30 ms and the other 265 cost 76 ms between them.
