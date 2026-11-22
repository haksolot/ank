---
id: LOG-2a6c6ce1ab04
type: log
title: "Swept at 6d2ffb98: 26 citations of ADR-376bed7fa39a, all in the full form, in 11 tracked files"
created: 2026-11-18T17:04:48Z
author: claude-code/opus-5.5+task-1601-sweep
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/hooks_install.rs
  - crates/ank-cli/tests/peer_aggregate.rs
  - crates/ank-cli/tests/peer_check_required.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-contract/src/verbs.rs
about: TASK-1601a5555adf
seq: 1
schema: 4
version: 1
---

 (verbs.rs x9, context.rs x5, status.rs x3, commands.rs x2, hooks.rs, human.rs, tests/cli.rs, tests/skill.rs, tests/hooks_install.rs, tests/peer_aggregate.rs, tests/peer_check_required.rs) re-pointed to ADR-89c1e7146fe5. Every one sources a --peers or ank hooks claim, both carried forward whole by the successor. After: command grep -rnE 'ADR-376b' (also '376be') outside .ank/ and target/ prints nothing (rc 1); cargo fmt --check green; ank check rc 0. Outside this scope: SPEC-79012ba18769 (accepted) lists ADR-376bed7fa39a in its references field, inside .ank/, not rewritten here; the ratifier should expect it.
