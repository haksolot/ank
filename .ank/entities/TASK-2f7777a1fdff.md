---
id: TASK-2f7777a1fdff
type: task
slug: a-change-becomes-an-event-and-the-reader-repaint
title: A change becomes an event, and the reader repaints instead of polling
created: 2026-08-24T22:03:25Z
author: claude-code/opus-5+planning
status: in_progress
scope:
  - crates/ank-daemon/**
  - crates/ank-tui/**
  - docs/integrating.md
  - crates/ank-contract/src/events.rs
  - crates/ank-contract/src/lib.rs
blocked_by: [TASK-a73b41660413, TASK-49746735127f]
done_criteria: |
  The daemon emits an event when a corpus it watches changes, stating which corpus by its repository identity and what kind of change occurred, and stating nothing about what a reader should do. The stream is documented in docs/integrating.md well enough for a reader written outside this repository to consume it. ank tui consumes it where it is available and falls back to its own reading where it is not, and a test shows the interface reaching the same displayed state by both routes. A test asserts the interface issues no repeating query while idle with the stream connected, and that no event ever carries entity content a reader would otherwise get from the CLI. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 4
version: 3
---

The last of ADR-a22cd3196529, and the one with a real consumer at last: the
reader of ADR-8bd76e8d7c4e, which is why this waits on it. An event stream with
nothing listening is a design nobody has measured.

**An event says what changed and not what it means.** The line between news and
answers is the line ADR-a22cd3196529 draws to keep the daemon from becoming a
third dispatch path, and it is easy to cross by kindness: an event carrying the
new state of a task saves the reader a call, and makes the daemon a source of
entity content that nothing generated from `COMMANDS` ever validated. So the
criterion forbids it outright, and asks for the absence to be asserted.

**Both routes must reach the same screen.** The daemon is optional, so the
interface has to work without it, and the failure mode of an optional
accelerator is that the fast path and the slow path drift until only the one the
developer runs is correct. Showing the same displayed state by both routes is
what stops that.

**Documented for a reader that is not ours.** ADR-8bd76e8d7c4e keeps the browser
reader outside this repository and keeps the contract it rests on public. A
private event stream that only `ank tui` could consume would quietly take that
back.
