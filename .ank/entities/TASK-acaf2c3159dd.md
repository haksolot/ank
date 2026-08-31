---
id: TASK-acaf2c3159dd
type: task
slug: ank-init-creates-no-directory-the-binary-never-w
title: ank init creates no directory the binary never writes into
created: 2026-08-31T03:11:45Z
author: claude-code/opus-5+drift
status: in_progress
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/init.json
blocked_by: []
done_criteria: |
  Run in an empty git repository: after `ank init`, `find .ank -type d` prints exactly `.ank` and `.ank/entities`, and the line `ank init` prints names those two and no third directory. Measured today the same run prints 'created .ank/entities .ank/log' and leaves .ank/log empty for the life of the corpus, because ADR-25f977377fa0 moved entries to .ank/entities/LOG-<id>.md. A test in crates/ank-cli/tests/cli.rs drives the binary, not the function, and fails if .ank/log is created again.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 3
---
