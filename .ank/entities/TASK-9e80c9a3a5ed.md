---
id: TASK-9e80c9a3a5ed
type: task
slug: every-output-block-the-documentation-shows-is-re
title: Every output block the documentation shows is replayed against the binary
created: 2026-09-19T18:26:57Z
author: claude-code/opus-5+docs-audit
status: done
scope:
  - docs/**
  - README.md
  - crates/ank-cli/tests/**
blocked_by: [TASK-4eef0864be46, TASK-959205eb8dee, TASK-58f946514828, TASK-6fe39029f6f1]
done_criteria: |
  A test in the workspace suite finds every block in docs/ and README.md marked as ank output, replays the commands that produce it in a scratch repository, redacts ids, hashes, times and versions with the golden redactor, and fails on any difference. Every block presented as ank output in those files is marked, or rewritten so it no longer presents itself as output. A deliberately altered block makes the test fail, shown in the log.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/1b7b42d27200@50353b2
    tree: scope/eea0222a3cf8
    criteria: a00479365e9a
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@50353b2
    tree: scope/eea0222a3cf8
    criteria: a00479365e9a
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Rests on ADR-2b62b9a1fe67 and ADR-33970fcdb6e8, both proposed on 2026-09-19: do not claim before they are ratified, since the shape of this work is what they decide. Blocked by the page fixes, which have to land first or this test starts red.
