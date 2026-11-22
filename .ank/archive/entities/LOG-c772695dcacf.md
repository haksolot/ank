---
id: LOG-c772695dcacf
type: log
title: "Measured through target/debug/ank at the branch head: ank skills tdd prints 4115 bytes, diagnose"
created: 2026-11-21T22:44:12Z
author: claude-code/opus-5.5+task-73e2-claim-method
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/skills.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - skill/**
  - docs/**
about: TASK-73e2322792c3
seq: 3
schema: 4
version: 1
---

 6052, loop 4888; claim on a tdd task prints the claim line, DONE_CRITERIA with the criterion indented as context prints it, then those 4115 bytes, compared byte for byte in cli.rs#claim_prints_the_criterion_then_the_method_text_and_nothing_without_one. Reading taken for --json: method_body is present only on a task naming a method, absent otherwise, because claim.rs#a_caller_with_no_declaration_sees_the_bytes_it_saw_before pins ADR-ed3e14d0f991's rule that a plain claim gains no key; a nullable field broke it. So claim declares four documents (plain, elsewhere, method, both) and claim.json is unchanged. Under --quiet the criterion and method text still print, like peer constraints. skill/: the load-record lines left tdd, diagnose and loop, revisions recomputed; the two record tests in tests/skill.rs became one asserting no file under skill/ contains log --method; log --method itself is untouched and cli.rs#log_method_records_the_load_on_the_claimed_task_and_renews_the_claim still passes. Targeted runs green: ank-contract 24, ank-cli bin 385, integration skill/doc_replay/prose/claim/method/golden/mcp/tui, ank-mcp, ank-tui.
