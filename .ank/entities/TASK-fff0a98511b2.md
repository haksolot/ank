---
id: TASK-fff0a98511b2
type: task
slug: the-reader-draws-its-first-frame-from-one-answer
title: The reader draws its first frame from one answer and waits for no second one
created: 2026-08-27T16:33:20Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-tui/src/model.rs
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-tui/tests/opening.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-tui/tests/chrome.rs
blocked_by: [TASK-b917fc12fee8]
done_criteria: |
  On a corpus of at least a thousand entities the reader draws its first frame having spawned no child that reads the corpus, and the rows arrive after it. ank status is spawned from no place on the opening road: neither for the corpus identity the event stream needs, which find now carries, nor inside the snapshot. A test names the two call sites that are gone -- crates/ank-tui/src/lib.rs and crates/ank-tui/src/model.rs -- and fails if a status spawn returns to either. ank tui --json still answers with branch, default_branch and identity, which is a different road and keeps its price. The pseudo-terminal harness sees a frame in under one second. Measured through the binary.
criteria_by: creator
proof:
  - type: commit
    ref: 7699a07f1414503505144d15c89e05701bb3be64
    criteria: c1dded4f4270
    via: submitted
schema: 4
version: 7
---
