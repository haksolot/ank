---
id: TASK-6ce2115cd8f0
type: task
slug: publish-npm-authenticates-by-trusted-publishing
title: publish-npm authenticates by Trusted Publishing, and no npm token is stored
created: 2026-11-06T12:41:38Z
author: claude-trusted-publishing
status: in_progress
scope:
  - .github/workflows/release.yml
  - docs/releasing.md
  - npm/README.md
blocked_by: []
done_criteria: |
  release.yml's publish-npm job grants id-token: write and publishes with no NPM_TOKEN secret and no _authToken in any npmrc, docs/releasing.md and npm/README.md describe Trusted Publishing and the per-package setting on npmjs.com in place of NPM_TOKEN, and the workspace suite stays green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 2
---
