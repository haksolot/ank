---
id: LOG-b4b6593fa6b0
type: log
title: Maintainer chose the freeze-only ref read. freeze_state now falls back to the anchor line of the
created: 2026-10-20T11:35:56Z
author: claude-code/1f3e
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/entries.rs
  - docs/format.md
  - docs/check.md
  - docs/quickstart.md
  - docs/peers.md
about: TASK-1f3e455dc134
seq: 8
schema: 4
version: 1
---

 tag at refs/ank/ratify/<id> when the commit walk finds nothing (human.rs ref_anchor); red first: the_freeze_reads_the_anchor_from_the_ratify_ref_when_no_commit_carries_it answered Unverifiable, now Intact then Altered. Signatures via verify-tag and every other reader stay for TASK-6e3f8f2ff6c9. Tests of the commit reader and its signatures now make the legacy ratify commit from the tag message (tests/legacy_anchor.rs). Two unix-only TUI tests rewritten (they never compile on Windows: off-default-branch refusal, ratification compared by commit message). Goldens blessed: accept json commit->tag, help, and the accepted files now uncommitted (status drift 2->4, verify changed 3->5, check/review +1 'differ from main' signal). Measured, one cargo at a time, MSYS_NO_PATHCONV unset: integration 707/707 (cli:: 397 in 387s, rest 310 in 239s), bin 371/371, core/contract/daemon/mcp/tui all green, cargo fmt --check clean, ank check exit 0.
