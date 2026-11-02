---
id: TASK-9d158e54cd7f
type: task
slug: no-pseudo-terminal-test-fails-on-macos-because-a
title: No pseudo-terminal test fails on macOS because another test filled the system's table of them
created: 2026-11-02T13:36:36Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - .config/nextest.toml
  - crates/ank-tui/tests/terminal/**
blocked_by: []
done_criteria: |
  The cause of run 41449785197's macos-latest failure (graph::the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing: posix_openpt EAGAIN, the table full) is established and recorded with ank log, measured rather than read. Under the ci profile, no test that opens many pseudo-terminals at once runs concurrently with another test that opens one: cargo nextest show-config test-groups --profile ci shows the arrangement, and a test in crates/ank-tui fails if a pseudo-terminal test leaves it. The ci workflow's macos-latest test step is green on the task's branch.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/40c4de2018be@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/6dd037b7ca3a@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/7d2a0ce3bd38@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/03a8c60f4a54@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/8ba4cb3ecddb@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/f697b2c0f741@d51c0fe
    tree: scope/05b6e0f61ff0
    criteria: a44b4ae08307
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 6
---

Main went red on the merge of #736 (9610a0ec), run 41449785197, macos-latest
only, in a test #736 did not touch:

    graph::the_cycle_key_unfolds_the_history_and_folds_it_again_reading_nothing
    panicked at crates/ank-tui/tests/terminal/mod.rs:1163:13:
    a pseudo-terminal could not be opened: posix_openpt: errno -6 ...
    (EAGAIN or ENOSPC from posix_openpt is the system's table of them full)

Hypothesis, to be confirmed by diagnosis and not assumed:
`terminal::every_terminal_opened_at_once_is_told_its_own_slave` opens many
terminals at once on purpose and is outside the `pty-serial` group in
.config/nextest.toml, so nextest can run it beside a `graph::` session, which
then finds the table full. The nextest.toml comments record why the group
exists and that its removal hung macOS twice: read them before changing it.

Nothing retries (the harness says so at mod.rs:1172), and a rerun that goes
green is not a fix.
