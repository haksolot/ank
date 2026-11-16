---
id: LOG-52843d4c8cff
type: log
title: "Measured through the binary on a fresh ank init: config --list prints 13 lines, exit 0; --list"
created: 2026-11-16T09:09:44Z
author: claude-code/opus-5.5+task-a59c-config
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - docs/config-keys.md
  - crates/ank-contract/src/shape.rs
about: TASK-fa89fb00791f
seq: 3
schema: 4
version: 1
---

 --json 13 entries, (type, JSON type of typed) pairs observed: boolean/bool, integer/int, string/str, and null only where source is unset; config verifier_lock --json -> value "true", type boolean, typed true. Integration test with two verifiers and a peer: every verifier field and peers.<name> listed, inputs typed as a list. Golden: config-list.json new, config-read.json and help.json reblessed; conforms() checks a Type::Any field against the type beside it; fixture count 38 -> 39. Suites: ank-contract, ank-core, ank-mcp ok, ank-cli bin 377 ok, integration config|golden|help|conform|schema|zero 120 ok, fmt ok.
