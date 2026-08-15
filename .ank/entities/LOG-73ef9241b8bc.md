---
id: LOG-73ef9241b8bc
type: log
title: "init.rs dropped from the scope: ank init prints a report about files, not a transition on an"
created: 2026-08-08T22:55:32Z
author: claude-code@ank
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/tests/cli.rs
  - docs/**
about: TASK-4601ed18d84e
seq: 1
schema: 3
version: 1
---

 entity, so section 4's grammar (word, identifier, landing state) has nothing to bind to there. A scope kept for a file the work does not touch is a check finding, and painting 'wrote .gitignore' green would be decoration rather than the rule.
