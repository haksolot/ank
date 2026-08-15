---
id: LOG-d4ea81b4c579
type: log
title: "Reproduced and cause found: the code was never wrong. Ran the incident state directly -- a detached"
created: 2026-08-02T22:21:49Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-1ea38a17d854
seq: 0
schema: 3
version: 1
---

 worktree at 7cde6ce, whose history carries the rewritten, unsigned ratification commit f770c98 for ADR-c656cbcc33a9. The installed ank.exe (C:\Users\seanl\.local\bin, built 2026-07-31 22:28) answers 0 there. The binary freshly built from this tree answers 8 with 'ADR-c656cbcc33a9: its ratification commit is not signed: the anchor proves nothing'. The installed binary predates d93af3b (2026-08-01 13:53), the commit that added the signature check: it has no signature check to run. So the cause lies neither in reaching the commit nor in judging it -- every git step answers correctly under both (rev-list finds f770c98 on the ADR path, allowed_signers is tracked and present, rev-list --format=%G? returns N). The measurement was taken with a binary older than the feature. What is still missing, and is the real hole: nothing exercises check through the binary on this case, so nothing would have caught a stale or broken wiring either.
