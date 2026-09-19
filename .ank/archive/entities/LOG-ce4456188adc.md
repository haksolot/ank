---
id: LOG-ce4456188adc
type: log
title: "DECISION: .ank/config.yml stays at schema: 1, and the reader stays at SUPPORTED_SCHEMA = 1. Three"
created: 2026-08-30T19:01:05Z
author: claude-code/opus-5+schema
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/schema.rs
  - .ank/config.yml
about: TASK-742cd978a806
seq: 4
schema: 4
version: 1
---

 reasons, in the order they bind. (1) A bump buys nothing: measured above, every released binary prints the same unknown-field message at schema 1 and schema 2, so no reader alive is helped. (2) A bump would cost a corpus. Making the current binary accept a bumped file means SUPPORTED_SCHEMA = 2, and the config check is an equality (schema != SUPPORTED_SCHEMA), not the MIN_SCHEMA..=SCHEMA_VERSION range the entity parser carries -- measured: schema 0 and schema 2 are refused alike today. Every corpus still at schema 1 would stop loading, which is the promise format.md makes and this task exists to keep: a corpus is never migrated by a tool that refuses to read it. Giving config.yml a version range is a real design decision, no ADR has made it, and it is outside this scope. (3) The honesty argument for bumping does not survive contact with what a bump is for. format.md bumps the version when an old reader would silently give a wrong answer -- its example is the log leaving the body, where a reader shows an empty history for a task that has one, silently, with nothing reading as an error. config.yml has no silent branch: deny_unknown_fields means an unknown key is always loud. So the bump's whole purpose is already served, and what a bump would change is only which loud message a reader gets -- which is what this task fixes in the reader instead, at no cost to any existing corpus. Stated plainly: a version-1 file does now carry a key version 1 did not define, and that tension is real rather than argued away. It is the cheaper of the two, because the key is optional, absent until set (ADR-443590981e41), and never silently misread.
