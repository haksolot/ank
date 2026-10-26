---
id: LOG-ee9ce44fc034
type: log
title: "Measured through the binary: init_bootstrap 5/5 green (fresh: added [AGENTS.md, CLAUDE.md],"
created: 2026-10-26T13:32:02Z
author: claude-code/opus-5.5+917a-init-bootstrap
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/init_*.rs
  - crates/ank-cli/tests/golden-json/init.json
  - crates/ank-cli/tests/golden-json/init-again.json
  - crates/ank-contract/src/verbs.rs
  - docs/quickstart.md
  - docs/claims.md
  - docs/integrating.md
  - .ank/entities/SPEC-3bccb8aee5b7.md
  - crates/ank-cli/tests/main.rs
about: TASK-917a8f9a1440
seq: 3
schema: 4
version: 1
---

 CLAUDE.md == '@AGENTS.md\n'; existing @AGENTS.md and @./AGENTS.md left byte-identical; terminated and unterminated CLAUDE.md keep their bytes and gain @AGENTS.md; second init leaves both files identical, added []). Red first: 4/5 failed before init.rs changed; the already-imports case was green from the start since nothing touched CLAUDE.md. cargo test --workspace: 0 failed (ank-cli integration 836 passed). The block keeps the old pointer line verbatim as its first line, so cli.rs's pointer tests (held by TASK-1641) stay green untouched.
