---
id: TASK-cf075f0e287d
type: task
slug: re-point-every-citation-the-two-supersessions-or
title: Re-point every citation the two supersessions orphaned
created: 2026-08-25T17:33:12Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-contract/src/lib.rs
  - crates/ank-mcp/**
  - crates/ank-daemon/**
  - .github/**
  - install.sh
  - install.ps1
  - npm/**
  - docs/**
blocked_by: []
done_criteria: |
  No file in the workspace cites ADR-372b82af1ec7 or ADR-e39a44f80e0e: every site names the decision that binds it today, or drops the citation and leaves the history to ank show. no_superseded_document_is_cited_in_the_workspace passes. Nothing else changes: no behaviour, no interface, no test assertion beyond the citations themselves, and cargo test --workspace is green with cargo fmt --check passing.
criteria_by: creator
proof:
  - type: commit
    ref: 737d204179536e8ef2244dd6b37ced362eb03e94
    criteria: 6dbd4b930046
    via: submitted
schema: 4
version: 4
---
