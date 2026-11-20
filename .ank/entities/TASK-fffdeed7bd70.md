---
id: TASK-fffdeed7bd70
type: task
slug: the-graph-pty-test-paints-no-frame-nobody-asked
title: The graph pty test paints no frame nobody asked for, under load
created: 2026-11-16T15:28:16Z
author: claude-code/plan
status: done
scope:
  - crates/ank-tui/**
blocked_by: []
done_criteria: |
  The cause of the intermittent failure of ank-tui graph::off_the_page_under_a_document_or_with_nothing_in_progress_nothing_is_painted is named in an ank log entry with the measurement that shows it. Seen twice on 2026-11-16 at ank done under load: once as a pty byte count of 17564 against 17545, once as 'a frame was painted with nobody typing'. The fix removes the cause, or arranges the timing the test depends on rather than retrying or widening a wait. The test then passes 50 consecutive runs while a second process loads every core, and the count is recorded with ank log.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@0b4a294
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/5f2b26d5c171@62cd05e
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/2b15bd87dbcb@62cd05e
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/589c57024151@ab98b1c
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-aeab-roles
  - type: test
    ref: local/a51e6e0d6610@0b4a294
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/7efa020e0060@0b4a294
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/7013b6cf3044@0b4a294
    tree: scope/6600b4db7939
    criteria: 03cbf3cdee99
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Two failures on 2026-11-16, both on ank done in a worktree while other workers ran: w-release (TASK-f754268004d2) and w-roles (TASK-a85d4a2325e7). Rerun alone it went 1 of 3 green once. Arrange the race; do not retry it.
