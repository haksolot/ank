---
id: TASK-5bb91a4cec00
type: task
slug: show-json-carries-amends-and-amended-by-for-an-a
title: show --json carries amends and amended_by for an ADR
created: 2026-09-27T20:46:03Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/integrating.md
  - crates/ank-cli/tests/amends_json.rs
blocked_by: [TASK-ea86d1cc4af4]
done_criteria: |
  Through the binary: ank show --json on an ADR that amends others carries an amends array of their ids, and on an amended ADR an amended_by array naming every accepted ADR that amends it; both are present and empty on an ADR with neither, so a consumer never has to test for the key. A spec and a log keep their current JSON shape, byte for byte. The golden fixtures and the JSON contract in docs/integrating.md say so. A test in crates/ank-cli/tests/amends_json.rs asserts each case through the binary.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/9c066620de9e@b97278c
    tree: scope/9dcc6a1ce127
    criteria: cf75192ff795
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@b97278c
    tree: scope/9dcc6a1ce127
    criteria: cf75192ff795
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Follow-up of TASK-fe548f3dd587 (issue #503), left out on purpose there: the
SHOW_OTHER output contract is shared by adr, spec and log, and the relation was
only in the content field on the amending side. A consumer over `--json` (the
MCP surface, the TUI, a script) cannot see from the amended ADR that part of it
no longer holds, which is the whole point of ADR-9ee76b578257.

Waits on the index task because both edit `show` in human.rs, and amended_by
should come from the index rather than from a second walk.
