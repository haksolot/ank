---
id: LOG-3eee7907aef4
type: log
title: "Green, suite exit 0 (46 binaries), fmt ok, ank check exit 0. Through the binary: one fact attested"
created: 2026-09-14T09:49:00Z
author: claude-code/opus-5+bearing-on
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
  - docs/getting-started.md
about: TASK-be336b87a145
seq: 4
schema: 4
version: 1
---

 twice -> 1 entry on the ref; a second run of it -> 1 entry holding the latest run; another identity -> 2 entries, show --json lists both. Forged ref of 171 entries of one fact, pushed to origin by name: show --json detached_proofs lists 1 (ci-run-171) before compaction; check --json carries a signal naming 'ank attest <id> --compact --detached' at exit 0; --compact --detached leaves 1 entry, the forge's latest under its own identity, nothing under the caller's; show still lists ci-run-171; the signal is gone. Mutations watched red: the signal disabled fails at 'check names the compaction'; the fold removed from compact_proofs fails at the entry count. This repository, read-only, no ref rewritten: 205 proof refs carrying 20518 entries (counted with git cat-file); TASK-b2c3d4e5f6a7 carries 170; show --json detached_proofs lists 170 with the PATH binary and 1 with this branch; check --json on this branch raises 201 compact signals naming 20514 entries of 201 facts, 0 faults (the other 4 refs already hold one entry). Golden help.json blessed: the only diff is --compact, two notes and the compacting document shape.
