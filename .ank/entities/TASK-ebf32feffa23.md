---
id: TASK-ebf32feffa23
type: task
slug: config-yml-declares-guard-corpus-and-guard-edit
title: config.yml declares guard.corpus and guard.edit, and ank config reads and writes them
created: 2026-10-25T08:13:18Z
author: claude-code/plan
status: done
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/guard_config.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/main.rs
blocked_by: []
done_criteria: |
  Under ADR-43cbb20dcbf1, tested through the binary: ank config guard.edit claim writes the key and ank config guard.edit prints claim; on a file without the key, ank config guard.edit prints warn and ank config guard.corpus prints deny, and the file is byte-identical afterwards; ank config guard.edit strict and ank config guard.corpus warn are refused with a nonzero exit that names the accepted values, and leave the file byte-identical; ank config --unset guard.edit removes only that key; docs/config-keys.md is regenerated and lists both keys with their values and defaults; cargo test and ank check are green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/08fcbaec4bce@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/48d5468e7904@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/76e0cc3d4f48@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/a01ba0bb03c3@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/13a9469e19c9@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/543ac81121cb@39563e7
    tree: scope/b59278e66cd8
    criteria: 658b72f9c704
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 5
---
