---
id: LOG-e2b2f5384ae6
type: log
title: "The reader is done and its own suites are green: Snapshot is find alone, Held is status and is"
created: 2026-08-27T18:51:41Z
author: claude-code/opus-5+reader-json
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/tests/opening.rs
about: TASK-fff0a98511b2
seq: 3
schema: 4
version: 1
---

 asked only when the claims panel takes focus (the road requeue already takes for the queue), session draws before it reads and attaches the follower once find has named the corpus, and lib.rs carries a test that the first frame precedes any read. 184 unit tests and all twelve ank-tui pty suites pass.

Four tests fail in crates/ank-cli/tests/tui.rs, a fifth file and a second crate. Two of them (a_claim_held_elsewhere_is_named_with_its_holder, a_driven_session_names_the_entities_the_corpus_carries) assert claims panel content without focusing it, which is the price now being charged where it is asked for. The other two are more interesting and are a harness fact rather than a defect: that suite's barrier is until(contains("ENTITIES")), and the opening frame now draws the panels empty, so the barrier is satisfied before the read lands and the assertions that follow race it. ank-tui's own harness waits on "2 ENTITIES" -- the count -- and did not race. Every such barrier in that file has to wait on the read rather than on a panel existing.
