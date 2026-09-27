---
id: LOG-4178c1decc76
type: log
title: "Hypothesis: in_perimeter hands the peer-qualified string 'bb:x' to ScopeSet as a local path, so"
created: 2026-09-27T17:04:01Z
author: claude-code/opus-5.5+c666
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/peer_scope.rs
about: TASK-c666eb306102
seq: 3
schema: 4
version: 1
---

 matching is plain string globbing: '**' matches any string, 'src/**' fails on the 'bb:' prefix, and 'bb:**' matches only because its literal prefix happens to equal the path's. Refuted if a glob with a colon-free wildcard prefix like '*/**' or '*:**'... fails to match; test: an ADR scoped 'b*:**' should also match bb:x under this hypothesis.
