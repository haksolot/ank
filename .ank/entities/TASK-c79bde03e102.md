---
id: TASK-c79bde03e102
type: task
slug: an-ank-older-than-requires-ank-warns-on-every-ve
title: An ank older than requires_ank warns on every verb and is a fault in check
created: 2026-10-24T13:37:23Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/requires_ank.rs
blocked_by: [TASK-109153abe8e0]
done_criteria: |
  Through the binary, in crates/ank-cli/tests/requires_ank.rs, against scratch corpora whose config declares requires_ank: with a value above the running version (read from ank --version), a read verb (ank find) and a write verb (ank new task) each print on stderr one warning naming the declared version, the running version and ank update, and exit as they would without it; their --json stdout is byte-identical to a run without the key; --quiet prints no warning; ank check exits 8 with a fault naming both versions; with a value equal to or below the running version, and with no key, nothing is printed and check reports nothing about it; versions compare component by component as numbers (0.10.0 is above 0.9.9); the suite starts no network process, shown by GIT_TRACE at an absolute path listing no ls-remote; each case asserts the exit code and the sentence.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Applies ADR-756e8b0fe416 once it is ratified; do not claim before. The warning sits beside warn_if_schema_ahead in cli.rs and takes its shape. The version compared is the one ank --version prints first, the same one ank update compares.
