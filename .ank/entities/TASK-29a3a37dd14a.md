---
id: TASK-29a3a37dd14a
type: task
slug: releasing-md-names-the-trusted-publisher-s-publi
title: releasing.md names the trusted publisher's publish-directly action
created: 2026-11-07T11:04:59Z
author: claude-releasing-doc
status: done
scope:
  - docs/releasing.md
blocked_by: []
done_criteria: |
  docs/releasing.md says each package's trusted publisher must allow publishing directly under Allowed actions, names the 403 a stage-only trust returns, and the workspace suite including the prose lint stays green
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
proof:
  - type: test
    ref: local/e3b0c44298fc@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/4cd1b13aa804@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/b97740ca029e@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/bf549fc26797@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
  - type: test
    ref: local/15f665ec945e@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/d83fdc04e41c@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/9faace96a474@455f7bd
    tree: scope/dbaaca769ff7
    criteria: 5319ae57f7de
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---
