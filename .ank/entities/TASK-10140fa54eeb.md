---
id: TASK-10140fa54eeb
type: task
slug: this-repository-declares-requires-ank-0-9-3-once
title: This repository declares requires_ank 0.9.3 once 0.9.3 is published
created: 2026-11-06T10:55:01Z
author: claude-requires-ank
status: done
scope:
  - .ank/config.yml
  - crates/ank-cli/tests/schema.rs
blocked_by: []
done_criteria: |
  the repository's .ank/config.yml declares schema 2 and requires_ank 0.9.3, and the test pinning the repository's own config asserts that schema and that key, so the workspace suite is green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/21d513b3dfc8@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/14fdae95bfc8@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/3339547d01a1@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/61dc6b8187f2@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/ea28b606432f@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/3035380b6741@1123d5b
    tree: scope/7f93abfa5720
    criteria: 19cdad211bf5
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---
