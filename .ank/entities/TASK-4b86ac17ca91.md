---
id: TASK-4b86ac17ca91
type: task
slug: ratify-require-signature-makes-accept-refuse-an
title: ratify.require_signature makes accept refuse an unsigned ratification and check fault one
created: 2026-11-16T08:08:03Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/accept_tag.rs
  - crates/ank-cli/tests/cli.rs
  - docs/signing.md
  - docs/config-keys.md
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/tests/reference_pages.rs
  - crates/ank-cli/tests/golden-json/config-list.json
blocked_by: [TASK-5f3d3d4374ea]
done_criteria: |
  With ratify.require_signature: true, ank accept in a repository that cannot sign with a key declared under signers: refuses before writing any ref or file, naming the key, and ank check reports an unsigned ratification, or one signed by an undeclared key, as a fault. Absent or false, accept behaves as today. ank config refuses to set it true while signers: declares no key. Tested through the binary (GitHub #791 section 3).
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@b98787b
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: fmt-check@695ca51e4ab9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4b86-signature
  - type: test
    ref: local/78a709191ba8@b98787b
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-core@835b758e7ce5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4b86-signature
  - type: test
    ref: local/5780c6433aca@b98787b
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-contract@0af4f641583a
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4b86-signature
  - type: test
    ref: local/1150e8b07fc1@b98787b
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-4b86-signature
  - type: test
    ref: local/1e87fd3cc226@7e8b5f6
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/7d9463f4de40@7e8b5f6
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/084c2a6f601d@7e8b5f6
    tree: scope/ab30fc9a4c85
    criteria: 7557b99bd808
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 5
---

Implements ADR-2e777475c4de, which is proposed: do not claim before a human has ratified it. GitHub #791 section 3.
