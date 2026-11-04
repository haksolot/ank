---
id: TASK-c2ed4811f8c7
type: task
slug: a-dead-commit-proof-owes-nothing-once-a-later-pr
title: A dead commit proof owes nothing once a later proof on the task names a reachable commit
created: 2026-11-04T10:03:57Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/proof_superseded.rs
  - crates/ank-cli/tests/main.rs
blocked_by: [TASK-519112db3aa5]
done_criteria: |
  Through the binary, in a scratch repository: a done task whose proof list holds a commit proof naming a commit no ref reaches, followed by a commit proof naming a reachable commit, draws no proof-commit-unreachable signal from ank check; a done task whose only commit proofs name unreachable commits still draws it, with the ank attest remedy it prints today; after ank attest <id> --proof commit:<reachable sha> on that second task, the next ank check reports nothing about it. A test in crates/ank-cli/tests/proof_superseded.rs asserts each case through the binary.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
schema: 4
version: 2
---

Found by TASK-a646a88c1c67 (its report and log entries): detached_commit_proofs (human.rs, near 'reported and never repaired') signals every dead sha the append-only proof list keeps, so the remedy the finding prints, ank attest, never clears it. At c9f113da five tasks carried the signal, all five with a later proof on a commit main reaches. Waits on TASK-519112 because both edit human.rs; that task also classifies this kind as owed (it names a command), which is only true once the command works.
