---
id: LOG-ca7e39c6ac7a
type: log
title: "Measured with the tree's binary in a scratch repo: init with no origin prints the 5 effects plus"
created: 2026-10-02T15:54:26Z
author: claude-code/opus-5.5+init-docs
scope:
  - docs/quickstart.md
  - docs/claims.md
  - crates/ank-core/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/doc_replay.rs
about: TASK-eff5715bfd87
seq: 1
schema: 4
version: 1
---

 'refspec +refs/ank/*:refs/ank/* not added: no remote named origin yet, run this once it exists (ank init)', and a re-run prints 'already initialised, nothing to do' plus that same line (so the old 'changes nothing' was wrong twice). With no origin, status has no warning and check no origin signal; after git remote add origin without the refspec, status prints 'warning: remote.origin.fetch lacks ...' and check 'signal: origin: ...' (3 signals, exit 0); a second init prints 'refspec added' and both go quiet. peers.bb <https URL> is refused exit 1 naming 'ank config --user peers.<identity>.bb <path>', which writes the override at exit 0. New test the_no_origin_init_is_held_by_the_replay watched red (block absent), then green; altering the claims.md signal line by one word turned claims_replays red (reverted).
