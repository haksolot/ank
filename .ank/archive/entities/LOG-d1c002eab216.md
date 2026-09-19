---
id: LOG-d1c002eab216
type: log
title: Measured before writing the successor, through the binary built at 44483d9, in a lab repository
created: 2026-09-14T12:04:07Z
author: claude-code/opus-5+plane-namespaces
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-daemon/**
  - docs/**
about: TASK-fe3ea1136b12
seq: 3
schema: 4
version: 1
---

 with a bare origin and one done task. attest --detached as process:ci with test:run-1 then test:run-2: 1 entry, then still 1 entry carrying run-2 only. As process:bk: 2 entries. A ref forged the way a pre-rule ref arrives (5 process:ci entries run-10..run-14 plus bk-1): 6 entries; show --json lists run-14 and bk-1 only; check prints 'signal: refs/ank/proof/<id>: 6 attestations of 2 fact(s): the ref grows with runs (ank attest <id> --compact --detached)'. attest --compact without --detached: exit 1 'needs --detached to say so'; with --proof: exit 1 'adds nothing, so it takes no --proof'. attest --compact --detached as human:sean: exit 0 'removed 4 (2 detached)', ref keeps run-14 and bk-1 and no human:sean entry; run again: 'already holds one entry per fact'. Three forged refs (refs/ank/remote-check/ x2, refs/ank/watch/origin/proof/ x1): check exit 0 with one signal per namespace (2 signals), one note line per ref 'git push origin :<ref>; git update-ref -d <ref>'. The mirror refspec is the one counted in tests/watch.rs a_cycle_mirrors_the_claims_namespace_and_no_proof (PR #426): one fetch, +refs/ank/claims/*:refs/ank/watch/origin/claims/*. Citations of SPEC-183d297253ac outside .ank/: git grep for SPEC-183d finds 0, so nothing to re-point. Found stale: ank help watch still says 'a mirror of refs/ank/* under refs/ank/watch/', and SPEC-77689b90b211 (CLI surface) says the watcher 'fetches refs/ank/*'; both describe the pre-#426 mirror and are outside this criterion.
