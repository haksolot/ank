---
id: LOG-66beeab9b0ce
type: log
title: "The test is proved non-vacuous by mutation: with the Absent arm of check_adr replaced by a no-op,"
created: 2026-08-02T22:27:06Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-1ea38a17d854
seq: 1
schema: 3
version: 1
---

 a_ratification_commit_stripped_of_its_signature_is_a_fault_through_the_binary fails with 'check: ok' and exit 0 -- the incident's exact output. Restored, it passes. The fixture reproduces the rewrite faithfully: accept signs for real, check is asserted silent first, then git commit --amend --no-edit with commit.gpgsign=false keeps the tree, the message and the constraint+scope trailer and drops the signature, which is what the merge did to f770c98. It also declares the generated key in .ank/allowed_signers, without which section 8 puts the corpus in advisory mode and every verdict is None -- the silence the test exists to refuse. No production code changed: none was wrong.
