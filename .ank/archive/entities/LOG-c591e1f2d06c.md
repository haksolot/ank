---
id: LOG-c591e1f2d06c
type: log
title: "Measured, not read: built ank at 4a50e48^ (the released 0.6.0 parser) into a scratch tree and ran"
created: 2026-08-30T19:00:39Z
author: claude-code/opus-5+schema
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/schema.rs
  - .ank/config.yml
about: TASK-742cd978a806
seq: 2
schema: 4
version: 1
---

 it against two corpora differing only in the schema line, each carrying default: under a verifier. schema: 1 and schema: 2 both produce the byte-identical 'verifiers.cargo-test: unknown field `default`, expected `run` or `timeout` at line 5 column 5' at exit 1. Cause: in the released ConfigFile, verifiers is a typed BTreeMap<String, VerifierFile>, so deny_unknown_fields fires inside the outer serde_yaml::from_str, before raw.schema is ever compared. Bumping the schema is inert for every shipped binary -- the task's premise, now measured rather than assumed.
