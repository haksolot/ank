---
id: TASK-ae64d1c5678d
type: task
slug: the-protocol-surface-reports-the-version-the-tag
title: The protocol surface reports the version the tag gates
created: 2026-08-25T21:29:25Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-mcp/**
  - .github/scripts/check-version.sh
blocked_by: []
done_criteria: |
  The version a client reads in serverInfo, and the one in the ank-mcp/<version> identity, is a number the release gates against the tag. Either check-version.sh holds crates/ank-mcp's version to the tag again, or the surface reports ank-cli's version and the crate's own number stops reaching a client; the choice is stated in the code where it is made. A test drives the built binary and shows the version a client is told matches the one ank --version prints. cargo test is green and cargo fmt --check passes.
criteria_by: creator
proof:
  - type: commit
    ref: "1e84563"
    criteria: 5234baab472f
    via: submitted
schema: 4
version: 3
---
