---
id: LOG-dddfe6bbb623
type: log
title: Counted after, measured not read, on ank 0.7.0 (bdf6e82) rebuilt from this tree and re-measured
created: 2026-08-31T04:54:46Z
author: claude-code/opus-5+citA
scope:
  - crates/ank-daemon/src/declare.rs
  - crates/ank-daemon/src/fetch.rs
  - crates/ank-daemon/src/lib.rs
  - crates/ank-daemon/src/stream.rs
  - crates/ank-daemon/src/warm.rs
  - crates/ank-daemon/tests/dependencies.rs
  - crates/ank-contract/src/events.rs
  - crates/ank-tui/src/stream.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - .github/workflows/release.yml
  - docs/integrating.md
about: TASK-d8448e35354e
seq: 1
schema: 4
version: 1
---

 after merging origin/main. Over the same fourteen paths: 0 occurrences of the six retired ids in full, and 0 of the abbreviated forms -- from 37 and 40. What stands in their place: ADR-24e21cb83793 32 times, SPEC-d58b3a9e4e4d 5 and the prefix SPEC-d58b twice, ADR-67a4ac10c534 once. Nothing was dropped: every one of the 40 references survives as a citation of the successor. ank check exits 0, 364 tasks, 84 adr, 0 fault(s), and no finding names a file in this scope. cargo test --workspace green, 0 FAILED across 42 test binaries; cargo fmt --check green.

Why a swap was safe for the bulk of it: ADR-24e21cb83793's constraint is ADR-a22cd3196529's word for word -- diffed with ank show and not assumed -- and only the scope moves, widening to crates/ank-daemon/**. Every one of the 32 sites cites a clause of that constraint: answers no verb, writes nothing but the fetch into a tracking namespace, nothing depends on it, an event says what changed and never what to do about it. Each sentence is true of the successor unchanged.

One site was rewritten rather than swapped, and it is the one the equal id length would have hidden. crates/ank-daemon/src/stream.rs:17 read ".ank/log/ is a work trace with a grammar and an append-only rule (ADR-ff29)". ADR-67a4ac10c534 moves an entry to .ank/entities/LOG-<ID>.md, one file per entry addressed as an entity, and drops the append-only sentence; it says so about its own wording. Measured rather than read: this task's first entry landed at .ank/entities/LOG-d0e0223d29bd.md, so the successor already describes the tree and the retired text describes nothing that exists. The sentence now reads "an entity's log is a work trace with a grammar, kept as entities of its own (ADR-67a4ac10c534)", which is the half that survives the change and the half the contrast with a bounded, thrown-away stream actually needed.

That abbreviation is written out in full deliberately. "ADR-ff29" is invisible to the walk accept and check share, so it would have outlived every future sweep while reading as a citation.

Nothing the binary prints changed. The only non-comment edits are five assertion messages in crates/ank-daemon/tests/dependencies.rs and seven fixture lines in crates/ank-tui/src/view.rs, so no golden and no snapshot moves and the scope needed no amendment. The TUI fixture keeps status "accepted" beside the now-proposed successor because that field is sample data and already was: the same fixture carries TASK-49746735127f as "in_progress" where the corpus says "done".

Two claims were pruned under me before the branch was pushed, which is the orphan sweep another agent's ank check runs: a task commit on an unpushed branch is not reachable to them. Pushing the branch first is what made the third claim hold.
