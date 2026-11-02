---
id: TASK-ef1d9a4eccce
type: task
slug: ank-herdr-teaches-orchestration-from-a-herdr-cor
title: ank-herdr teaches orchestration from a herdr-core terminal through hc, and every citation of ADR-55449b7f4f8f points to ADR-2fc9b88cc0bd
created: 2026-11-02T10:37:24Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - skill/herdr/SKILL.md
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/golden-json/skills.json
  - docs/multi-agent.md
  - docs/install.md
  - docs/alternatives.md
  - CLAUDE.md
  - .claude-plugin/**
blocked_by: []
done_criteria: |
  Through the binary and the files it carries: skill/herdr/SKILL.md stops when HC_TERMINAL_ID is unset and defers syntax to hc --skill; it says a worker's ANK_AGENT, TMPDIR and PATH are set in the command that starts it, because a worker inherits the herdr-core server's environment. No file under skill/, docs/, crates/ank-cli/tests/ or CLAUDE.md names HERDR_ENV, herdr --skill or a herdr pane. No file outside .ank/ cites ADR-55449b7f4f8f in its long or 9-character form; those citations name ADR-2fc9b88cc0bd. crates/ank-cli/tests/skill.rs asserts each of these, and ank skills prints the new ank-herdr description.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: tdd
proof:
  - type: test
    ref: local/e3b0c44298fc@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/a753c1939819@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-core@835b758e7ce5
    via: verifier
  - type: test
    ref: local/5679d00acc6e@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-contract@0af4f641583a
    via: verifier
  - type: test
    ref: local/fd890b9ed4f8@0a0437a
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-daemon@5bd3959f3ef9
    via: verifier
    reused: local
    by: claude-code/opus-5.5+task-d199-peer-worktree
  - type: test
    ref: local/43954be85539@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-mcp@9352d5d025bc
    via: verifier
  - type: test
    ref: local/a9da3ddff0a1@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-tui@d5ab50cd6075
    via: verifier
  - type: test
    ref: local/d7250ee01319@90e14ed
    tree: scope/f167c30e534d
    criteria: cfbe6ef5e1bf
    verifier: test-ank-cli@f3a5da7d2d74
    via: verifier
schema: 4
version: 3
---

ADR-2fc9b88cc0bd supersedes ADR-55449b7f4f8f: the maintainer's sessions run in
herdr-core (`hc`), where `HERDR_ENV` is unset, `herdr --skill` describes a
runtime that is not there, and a worker started with `hc run` inherits the
server's environment. The ADR body records the measurements.

The citation sweep comes first on this branch, then the rewrite; the ADR is
accepted on this branch by the maintainer before it lands, so check never sees
a citation of a superseded decision on main. Grep both id forms, including in
files you create.

The sibling still teaches the ank layer only. Do not copy hc commands into it:
`hc --skill` is the authority. A working launch template, for reference only,
lives outside the repository at ~/.config/herdr-core/templates/ank-worker.md.
