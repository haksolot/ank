---
id: TASK-6ce2115cd8f0
type: task
slug: publish-npm-authenticates-by-trusted-publishing
title: publish-npm authenticates by Trusted Publishing, and no npm token is stored
created: 2026-11-06T12:41:38Z
author: claude-trusted-publishing
status: done
scope:
  - .github/workflows/release.yml
  - docs/releasing.md
  - npm/README.md
blocked_by: []
done_criteria: |
  release.yml's publish-npm job grants id-token: write and publishes with no NPM_TOKEN secret and no _authToken in any npmrc, docs/releasing.md and npm/README.md describe Trusted Publishing and the per-package setting on npmjs.com in place of NPM_TOKEN, and the workspace suite stays green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/70574c0d7d87@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/7eb60a029986@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/3339547d01a1@1123d5b
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-requires-ank
  - type: test
    ref: local/48006d95585d@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/489c84e13df6@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/9658f161e764@0cd9d3a
    tree: scope/96a1f3696bc3
    criteria: e993a828e339
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---
