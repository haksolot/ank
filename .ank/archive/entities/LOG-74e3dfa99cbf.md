---
id: LOG-74e3dfa99cbf
type: log
title: "scope amended: tests/schema.rs#a_schema_2_config_with_requires_ank_is_read wrote requires_ank:"
created: 2026-10-24T15:14:51Z
author: claude-code/opus-5.5+wc79b
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/requires_ank.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/tests/schema.rs
about: TASK-c79bde03e102
seq: 4
schema: 4
version: 1
---

 1.2.3 and asserted find says only 'no match'; 1.2.3 is above the running 0.9.2, so the warning this task adds is correct there. The test is about the read, so its value moves to 0.1.0, at or below any running version, and its silence assertion stays true.
