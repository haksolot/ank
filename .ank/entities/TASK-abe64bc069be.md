---
id: TASK-abe64bc069be
type: task
slug: rename-to-ank
title: Rename the project to ank, code, documentation and repository
created: 2026-07-29T10:40:00Z
status: done
scope:
  - crates/**
  - docs/**
  - skill/**
  - README.md
  - CLAUDE.md
  - AGENTS.md
  - Cargo.toml
  - Cargo.lock
  - .gitattributes
  - .gitignore
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  The binary is called ank, the crates ank-core and ank-cli in directories of
  the same name, the state directory .ank/, the ref namespace refs/ank/*, the
  identity variable ANK_AGENT, the specification docs/ank-spec-v1.1.md. The
  repository on the host is called ank.
  Outside .ank/, the command
  grep -rIni ankor --exclude-dir=.git --exclude-dir=target . returns no line.
  Inside .ank/, it returns only three kinds, each an anchor that rewriting
  would destroy: log entries already written carrying claude-code@ankor, the
  proof reference ci://haksolot/ankor/runs/30324400136 on TASK-ca4714f5c719,
  and the bodies of the two entities documenting this rename.
  File moves go through git mv, so that each file's history stays tracked.
  cargo test, cargo fmt --check and cargo run --example check_repo are green,
  and the CI is green on all three operating systems — the rename touches
  .gitattributes, whose only known failure is Windows-specific.
criteria_by: claimer
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: commit
    ref: 4fb2040
  - type: test
    ref: ci://haksolot/ank/runs/30563030874
schema: 3
version: 4
---

Ratified by ADR-85e6bbb195b8, which carries the decision and the list of what was
deliberately not rewritten.

The rename is not cosmetic from the format's point of view: `.ank/` is the name
any third-party tool must know, `refs/ank/*` the one the host must fetch,
`ANK_AGENT` the one the harness must set. The ordering of ADR-63b59c5c26f7
therefore applies, and it is verifiable here at no cost: the golden files contain
none of those three strings, which is the proof that the entity format does not
move.

The criterion stayed in prose rather than becoming a named verifier. A
`no-ankor` in `config.yml` was tempting — it is exactly the mechanisation §11
encourages — but it would have reported as a fault the bodies of this ADR and
this task, which must name the old name in order to explain what was kept. A
constraint whose mechanical form accuses the documentation of the constraint
itself is not ready to be mechanised.

`criteria_by: claimer`: the criterion was set at pickup time, an assumed signal
as for TASK-244a842bc0cc and TASK-c8637488773c.
