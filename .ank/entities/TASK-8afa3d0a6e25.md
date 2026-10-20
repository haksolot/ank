---
id: TASK-8afa3d0a6e25
type: task
slug: a-shallow-clone-verifies-a-ratification-carried
title: A shallow clone verifies a ratification carried by a ref
created: 2026-10-17T14:35:05Z
author: haksolot@omarchy
status: done
scope:
  - crates/ank-cli/tests/**
blocked_by: [TASK-6e3f8f0049e9]
done_criteria: |
  A test through the binary: a repository ratifies by ref; a second clone made with --depth 1 fetches +refs/ank/*:refs/ank/* and runs ank check, which reports that ratification verified with its signature judged, and reports a legacy commit anchor in the same corpus as unverifiable rather than altered. Green on Linux, macOS and Windows.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@44aa7fd
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/a62fc9cd0fe5@3917e91
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-core@11c0192e3520
    via: verifier
    reused: local
    by: claude-code/6e3f
  - type: test
    ref: local/c1e10fac0370@3917e91
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-contract@60b1ecdf7b92
    via: verifier
    reused: local
    by: claude-code/6e3f
  - type: test
    ref: local/42a444150156@3917e91
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-daemon@c8f40b42bd7b
    via: verifier
    reused: local
    by: claude-code/6e3f
  - type: test
    ref: local/10a6f160106f@44aa7fd
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-mcp@23a803416bc4
    via: verifier
  - type: test
    ref: local/4a5c24cf1d1b@44aa7fd
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-tui@06060d7e7df5
    via: verifier
  - type: test
    ref: local/a70466dc6a92@44aa7fd
    tree: scope/97e6a93a4cd5
    criteria: 90c46a8d0688
    verifier: test-ank-cli@b817bb082d2d
    via: verifier
schema: 4
version: 3
---

This is the property a hub of corpora consumes: files at a commit plus
refs/ank/*, no history. It is also the test that says when CI may drop
fetch-depth: 0.
