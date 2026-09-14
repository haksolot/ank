---
id: TASK-da978b214eca
type: task
slug: the-store-and-the-index-read-ank-archive-and-a-v
title: The store and the index read .ank/archive/, and a verb reaches it only when asked
created: 2026-09-14T06:40:13Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  An entity file moved by hand to .ank/archive/entities/ disappears from find, context, scope and graph, and is answered whole by show, listed by log on its subject and by find --all, all through the binary. Opening the index on a corpus with 1000 archived files hashes none of them on the second open and the git process count of graph is unchanged. check verifies an archived file against the digest the index holds and reports one whose bytes changed as a fault, without parsing it. The find --json document gains nothing retyped. cargo test --workspace, cargo fmt --check and ank check stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/a9cdcf9cd7b2@c0eaf33
    tree: scope/09a6b2bf6e70
    criteria: d7ef587037c1
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@c0eaf33
    tree: scope/09a6b2bf6e70
    criteria: d7ef587037c1
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

The decision is ADR-306fdb75e265; this task is the reading half. The store already
reads two roots, canonical first (`read_path_of`, `store.rs:438`;
`LEGACY_DIRS`, `store.rs:419`), and the index scan already walks a list of
directories (`index.rs:625`). The archive is a third root read on demand:
`show`, `log` and `find --all` resolve into it by id, and nothing else
walks it. Rows for archived entities carry a flag so that `find --all`
answers from the index and not from a second walk; the flag is a schema
change and rebuilds every index once. The digest `check` verifies is the
content hash the index holds for the file (ADR-1556aaffe0c5 records it); an archived
entity is immutable, so a changed hash is a fault and a matching one is
the whole of the verification.
