---
id: LOG-3cfa6b55defe
type: log
title: Regression test human::tests::signing_is_enabled_over_a_key_already_in_the_fixture (enable_signing
created: 2026-10-17T16:57:02Z
author: claude-code/opus-5.5+78ef
scope:
  - crates/ank-cli/src/human.rs
about: TASK-78ef9b4f7cb2
seq: 4
schema: 4
version: 1
---

 twice on one Temp). Without the fix: red on Linux, panic at human.rs:9503 'ssh-keygen: ' and nothing after, the CI signature. With the fix (remove signing-key and signing-key.pub before generating, NotFound ignored): human::tests 69 passed 0 failed. With the removal disabled and the new message: 'ssh-keygen: exit status: 1 / stdout: .../signing-key already exists. Overwrite (y/n)? / stderr: ' -- status and stdout now carried.
