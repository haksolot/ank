---
id: TASK-c84e1300be5a
type: task
slug: doc-replay-format-replays-finds-the-task-its-own
title: doc_replay::format_replays finds the task its own setup created
created: 2026-11-16T15:28:19Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/tests/doc_replay.rs
  - docs/format.md
blocked_by: []
done_criteria: |
  The cause of the failure of doc_replay::format_replays on ubuntu-latest in CI run 43857426507, where the setup 'ank claim TASK-5f1e' answered 'entity not found: TASK-5f1e' after the page's 'ank new' printed 'created TASK-5f1e0a9c2b7d', is named in an ank log entry with the measurement that shows it. The fix removes the cause, and a test reproduces the condition deterministically: it fails before the fix and passes after.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@325fee6
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/5f2b26d5c171@62cd05e
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/2b15bd87dbcb@62cd05e
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/589c57024151@ab98b1c
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/0817f936f150@325fee6
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/4bfee5c23223@325fee6
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/d0f72ff32822@325fee6
    tree: scope/9d459accf8b5
    criteria: a755497a0dfb
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

One failure on PR #798 (a corpus-only PR), green on the rerun and 3 of 3 locally. Read how the replay maps the ids the page prints to the ids the binary creates.
