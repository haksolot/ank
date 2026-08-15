---
id: LOG-2cfad3c0dbad
type: log
title: "Located the hint sites the criterion names: done.rs:322 (absent verifier), commands.rs:601 and :676"
created: 2026-08-09T14:50:22Z
author: claude-code@ank
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
about: TASK-797d64113614
seq: 0
schema: 3
version: 1
---

 (new's and amend's unknown --verify, two separate functions check_verifiers and verifiers_of), and four default_branch sites -- context.rs:330, git.rs:504, human.rs:319, status.rs:127. That is seven call sites, not six; the criterion says 'six hints' and then enumerates seven. Fixing all seven. Placement decided: config goes after check and before init in section 4's Commands block, since it shares the startup exemption with init and help. --unset takes -u, which is free, because section 4's short-form table lists letterless flags only where a letter collided.
