---
id: TASK-3dcf8cfc2111
type: task
slug: the-cold-readers-test-has-a-bound-of-its-own-mea
title: The cold-readers test has a bound of its own, measured against its Windows passes
created: 2026-11-22T08:09:56Z
author: haksolot@vmi3223161
status: done
scope:
  - .config/nextest.toml
blocked_by: []
done_criteria: |
  In the ci profile of .config/nextest.toml, the override that already names cli::concurrent_readers_of_a_cold_corpus_of_this_size_all_answer gives that one test a termination bound of 1200s, and every other test keeps the 600s bound: 'cargo nextest list --profile ci' still parses the file, and a run of that one test under '--profile ci' with ANK_TEST_COLD_ROUNDS=1 passes. The comment above the bound states the measurement it rests on: the eight green windows-latest runs of main from 44956360201 to 44972253757 passed it in 264s to 435s, and run 44946110653 killed it at 600s. Before the bound is written, the per-round times printed by those eight runs are read from the job logs and recorded with ank log, with whether they grew with the round number or with the suite's size; if one round costs more than 30s on Windows, a follow-up task is filed for the cost and this one still closes on the bound.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@633e905
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-d0d8-one-index-open
  - type: test
    ref: local/e19f8cf880ed@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-3dcf-cold-readers-bound
  - type: test
    ref: local/4b774997015e@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-3dcf-cold-readers-bound
  - type: test
    ref: local/858fbcd875ca@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-3dcf-cold-readers-bound
  - type: test
    ref: local/96561bb81334@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-3dcf-cold-readers-bound
  - type: test
    ref: local/331d79f33ffd@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/cfaff9c9b4d4@98d57d9
    tree: scope/c12bf3e2a1cf
    criteria: 2603c8e82f65
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

## Why

The 600s bound (TASK-8cb5af6607f5) was set when the slowest pass seen was
335.911s, 1.8 times under it. The test runs twenty rounds under CI
(TASK-4a819896815a) and now passes on windows-latest in 264.186, 383.806,
391.626, 404.847, 423.918, 429.041, 434.725 and 343.549s: the bound is 1.4
times the slowest, and run 44946110653 of main (445778b3) crossed it and turned
main red over a corpus-only merge.

## The choice made here

The twenty rounds stay, because they are what catches a rare refusal, and the
600s bound stays for every other test, because it is what turns a hang red.
Only this test's bound moves. Cutting the rounds or raising the bound for the
whole suite were the alternatives, and both lose something.
