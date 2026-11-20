---
id: TASK-a352f4e8c32b
type: task
slug: init-and-watch-declare-the-checkout-they-run-in
title: init and watch declare the checkout they run in, and the watcher re-reads watch.yml (#806)
created: 2026-11-16T15:23:39Z
author: octob@ordann
status: done
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/watch_declare.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-contract/src/env.rs
  - crates/ank-contract/src/verbs.rs
  - docs/watch.md
  - docs/environment.md
  - .cargo/config.toml
  - crates/ank-cli/tests/golden-json/**
blocked_by: []
done_criteria: |
  Through the binary: ank init in a repository with a commit adds it to watch.yml, creating the file; ank watch started in an undeclared checkout declares it then warms it; a corpus added to watch.yml while ank watch runs is warmed without a restart; ANK_WATCH_DECLARE=off writes nothing; a repository with no commit is not declared and init still exits 0; an existing watch.yml keeps its comments.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/81b44a8086f7@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/1273763c0de1@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/01c1c83bd2dd@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/ac2464269477@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
    reused: local
    by: claude-code/opus-5.5+closer
  - type: test
    ref: local/063424576632@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/c2fcae1a6c86@a8c429e
    tree: scope/6079fdce43c9
    criteria: 7bc4e374e3e7
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 4
---

Implements ADR-f6b8e6539335 for issue #806.
