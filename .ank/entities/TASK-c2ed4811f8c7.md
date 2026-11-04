---
id: TASK-c2ed4811f8c7
type: task
slug: a-dead-commit-proof-owes-nothing-once-a-later-pr
title: A dead commit proof owes nothing once a later proof on the task names a reachable commit
created: 2026-11-04T10:03:57Z
author: haksolot@vmi3223161
status: done
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
proof:
  - type: test
    ref: local/e3b0c44298fc@3b33a31
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/38c5b6f938a4@e3e47fa
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-5191-check-owed
  - type: test
    ref: local/6739b142cbd1@e3e47fa
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-5191-check-owed
  - type: test
    ref: local/78af3fc3c2d0@e3e47fa
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-5191-check-owed
  - type: test
    ref: local/3bcc192cf4c6@3b33a31
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/6b9d93e3fbb9@3b33a31
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/e910b8edcfb4@3b33a31
    tree: scope/b3483f9175e2
    criteria: f65ef9093723
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Found by TASK-a646a88c1c67 (its report and log entries): detached_commit_proofs (human.rs, near 'reported and never repaired') signals every dead sha the append-only proof list keeps, so the remedy the finding prints, ank attest, never clears it. At c9f113da five tasks carried the signal, all five with a later proof on a commit main reaches. Waits on TASK-519112 because both edit human.rs; that task also classifies this kind as owed (it names a command), which is only true once the command works.
