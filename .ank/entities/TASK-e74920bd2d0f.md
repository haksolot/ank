---
id: TASK-e74920bd2d0f
type: task
slug: ank-hooks-status-says-which-agents-have-ank-entr
title: ank hooks status says which agents have ank entries installed, pointing at which binary, and which guard levels are in force
created: 2026-11-18T15:03:08Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/hooks_status.rs
  - crates/ank-cli/tests/skill.rs
  - docs/integrating.md
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/hooks_install.rs
blocked_by: [TASK-911fe94dd281, TASK-1601a5555adf]
done_criteria: |
  ank hooks status lists every agent ank hooks install knows, and for each whether its entries are installed in that agent user-level settings, the binary path they call, and whether that path resolves to the running binary; it then lists guard.corpus and guard.edit as declared by the corpus and as in force after ANK_GUARD_CORPUS and ANK_GUARD_EDIT, saying when an override was ignored as looser. Outside a corpus it lists the agents and says no corpus is in force. It writes nothing: the settings files and .ank/ are byte-identical after it. --json carries the same facts with a golden fixture, and ank help lists the subcommand. Tested through the binary (GitHub #790 item 3).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/6d5cb3317ac9@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/735c574801c1@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/20b9bc0e082a@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/277150321680@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/2c34281ccc56@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/f8e65cf9e6ca@c1efa7e
    tree: scope/b0ecf62ee950
    criteria: 38567a8ef61e
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 5
---

Implements ADR-89c1e7146fe5 (supersedes ADR-376bed7fa39a), which is proposed: do not claim before a human has ratified it. The superseding ADR is what admits the subcommand to the frozen surface.
