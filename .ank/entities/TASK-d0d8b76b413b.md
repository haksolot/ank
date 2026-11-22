---
id: TASK-d0d8b76b413b
type: task
slug: status-and-review-open-the-index-once-in-a-corpu
title: status and review open the index once in a corpus that holds an archive
created: 2026-11-21T22:54:53Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/status.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-tui/tests/graph.rs
blocked_by: []
done_criteria: |
  Through the binary, counted with ANK_TRACE_READS and never by a wall clock, in a corpus whose .ank/archive/entities/ holds at least one entry: ank status and ank review each open the index once, as they do in a corpus with no archive, and print the bytes they printed before. The count crates/ank-tui/tests/graph.rs pins for the reader's wide opening returns to one open per verb.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@633e905
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/2d71e13ed8a1@d21f5c8
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-73e2-claim-method
  - type: test
    ref: local/ac98e3dd493a@d21f5c8
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-73e2-claim-method
  - type: test
    ref: local/e0693ff88389@d21f5c8
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-73e2-claim-method
  - type: test
    ref: local/4ca6eb3b6759@633e905
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/867721973884@633e905
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/8a5c48d0030a@633e905
    tree: scope/392ff8c3bdea
    criteria: c3b89ad4a43f
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

Issue #832, found by the worker of TASK-ed7990dab03c on 2026-11-21 and measured on a lab corpus, index opens before and after one `ank close`:

    find 1/1   status 1/2   review 1/2   graph 1/1

The second open is `inspect()`'s `Index::open_with_archive` (human.rs), taken whenever `.ank/archive/` exists. It predates that task, and this repository already pays it. Since ADR-302efb9c5e40 every corpus with a finished task holds an archive, so the cost reaches all of them: ADR-f3d1dea65d84 asks that a verb pay for the answer it gives. TASK-ed7990dab03c pinned the reader's count at the two-open cost and left this as the follow-up.
