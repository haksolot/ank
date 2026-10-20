---
id: LOG-adb2e405ab5b
type: log
title: Measured on Windows, ratify_ref::a_shallow_clone_verifies_a_ref_ratification_and_cannot_verify_a_com
created: 2026-10-20T16:43:17Z
author: claude-code/8afa
scope:
  - crates/ank-cli/tests/**
about: TASK-8afa3d0a6e25
seq: 2
schema: 4
version: 1
---

mit_one. The clone stays shallow after fetching +refs/ank/*:refs/ank/* (is-shallow true, rev-list --count HEAD = 1). check exits 0 with 0 faults; GIT_TRACE counts 1 verify-tag (A's tag judged); A's only line is 'ratified by its own author'; C's line is 'signal: ADR-0000000006cc: ratified, but no ratification commit is reachable: the freeze cannot be verified'. The behaviour was already in the tree (first meaningful run green; the first red was my over-strict about(A).is_empty() hitting the self-ratification signal). Falsified twice: (a) git fetch --unshallow before check -> red at the C assertion, the unverifiable signal is gone ('check: ok, 0 tasks, 3 adr, 3 signal(s)'); (b) A tagged under an undeclared key -> red, exit 8, 'error: ADR-0000000006aa: ratified by SHA256:I5Zc..., which signers: in .ank/config.yml does not declare'. So the signature is judged in the shallow clone, not skipped.
