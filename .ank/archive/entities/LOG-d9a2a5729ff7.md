---
id: LOG-d9a2a5729ff7
type: log
title: "Baseline measured with GIT_TRACE: done on a src/a/** task with one declared verifier starts 16 git"
created: 2026-11-03T14:48:16Z
author: claude-code/opus-5.5+task-64f9-out-of-scope
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/done_out_of_scope.rs
  - crates/ank-cli/tests/main.rs
about: TASK-64f941335624
seq: 1
schema: 4
version: 1
---

 processes (ls-files x2 + hash-object --stdin-paths among them, from verify::keys). Budget is 18. Design: diff-tree --merge-base -r -z --name-only <default> HEAD for committed changes (deletions included), and one cat-file --batch-check HEAD:<p> compared with the hash-object answers keys already holds, for uncommitted ones. diff-index would be one process but ADR-9307 does not admit it, and a third ls-files invocation is refused by ADR-db58. Reusing keys's listing and hashes needs crates/ank-cli/src/verify.rs, outside the scope: asking the orchestrator before widening.
