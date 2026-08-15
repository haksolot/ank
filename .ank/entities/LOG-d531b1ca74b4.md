---
id: LOG-d531b1ca74b4
type: log
title: release.yml builds three targets on a tag and runs the suite per target before packaging. Linux is
created: 2026-07-31T17:33:04Z
author: claude-code@ank
scope:
  - .github/workflows/release.yml
  - skill/**
  - crates/ank-cli/tests/skill.rs
about: TASK-b8c9d0e1f2a3
schema: 3
version: 1
---

 musl and the artefact is static-pie linked, verified by downloading it, not asserted. workflow_dispatch builds the same three and skips publish entirely, so the pipeline was proved twice before a tag existed; without it the first proof would have been a tag needing deletion. gh does the publishing rather than a third-party action, same supply-chain reasoning as ci.yml, and only the publish job holds contents: write. Checksums travel with each artefact because a hash the publisher computed elsewhere is not a check anyone can repeat.
