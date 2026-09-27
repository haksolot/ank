---
id: TASK-ad407ccdae3a
type: task
slug: ank-tui-repo-reads-the-corpus-it-names-the-child
title: "ank tui --repo reads the corpus it names: the child call puts the address flags after the verb"
created: 2026-09-27T07:41:01Z
author: claude-tui-495
status: done
scope:
  - crates/ank-tui/src/ank.rs
  - crates/ank-tui/tests/**
blocked_by: []
done_criteria: |
  The binary run as `ank tui --json --repo <repo>` from a cwd outside that repository answers the corpus of <repo>, and the same with --worktree; a test invokes the binary to show it. Every command line the reader composes, spawned or shown in a confirmation, is `ank <verb> <args> <address flags> --json`, the order the CLI accepts (GitHub issue #495).
criteria_by: creator
verify: [cargo-test, fmt-check]
method: diagnose
proof:
  - type: test
    ref: local/55b29a189ebf@a6b44e5
    tree: scope/38e17e517211
    criteria: 36f903e7faba
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@a6b44e5
    tree: scope/38e17e517211
    criteria: 36f903e7faba
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---

GitHub #495. crates/ank-tui/src/ank.rs argv() composes `ank --repo <dir> <verb> ... --json`; the CLI reads global flags after the verb only and refuses the first word with exit 1 (measured: `ank --repo <repo> status --json` exit 1 'unknown command --repo', `ank status --repo <repo> --json` exit 0). Every child call fails as soon as --repo or --worktree is set, so the reader stays at 'the corpus has not been read'. The golden test the_spelling_carries_the_address_the_child_is_given pins the rejected order because it compares strings and never runs the binary. Out of scope: showing a refused child call on screen (separate task).
