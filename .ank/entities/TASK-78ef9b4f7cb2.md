---
id: TASK-78ef9b4f7cb2
type: task
slug: the-ssh-keygen-signing-fixture-of-human-tests-sa
title: The ssh-keygen signing fixture of human::tests says why it failed, and survives a transient failure on Windows
created: 2026-10-17T13:42:56Z
author: haksolot@omarchy
status: done
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  When ssh-keygen fails in human::tests enable_signing, the assertion message carries its exit status and stdout as well as stderr. The cause of the empty-stderr failure seen on windows-latest is named in a log on this task, from a reproduction or from the runner's record. The fixture is then changed so that this cause no longer fails the test, and the change is shown by a run of human::tests on all three platforms.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@6e77859
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/44ca270bf8c3@75f0570
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/fable-5.1+fe25
  - type: test
    ref: local/023c770a8dfc@75f0570
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/fable-5.1+fe25
  - type: test
    ref: local/de5e5c48a8b9@75f0570
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/fable-5.1+fe25
  - type: test
    ref: local/1d48bf54c246@6e77859
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/b56fd3e66110@6e77859
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/a6c044c73799@6e77859
    tree: scope/0703bf2f802e
    criteria: b812ba7885fa
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

Seen on Windows CI for PR #624 (run 39023341364, job 117369363083): human::tests::accept_refuses_an_adr_that_already_carries_an_anchor panicked at src/human.rs:9502 with 'ssh-keygen: ' and nothing after it. ssh-keygen exited non-zero and wrote nothing to stderr. The PR changed only one .ank file, and the rerun is the evidence that it is transient. The assertion prints only stderr, so the exit status, which would separate a crash, a missing binary on PATH and a key-file race, is lost.
