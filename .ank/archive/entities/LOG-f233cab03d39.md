---
id: LOG-f233cab03d39
type: log
title: "Specification first, then the code, as ADR-63b5 orders. Section 4 now states the route: amend"
created: 2026-08-11T01:38:28Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/editor.rs
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/**
about: TASK-7c2fa14284ff
seq: 2
schema: 3
version: 1
---

 --criteria refused only while a live claim freezes the criterion, criteria_by untouched because an amend is not a claim, and claim --criteria sets an absent criterion and never replaces one. Revision l records it. The state test is claim::live, one function both amend and edit call, so the two cannot drift; edit had it privately and the specification said nothing about it. Two consequences beyond the criterion. SKILL.md line 65 became false the moment amend changed -- it taught that amend will not touch done_criteria, that release is the route -- so the sentence is corrected and metadata.revision bumped to 8295e2081364; this is a correction, not growth, the verbs and modes it teaches are unchanged and it stays inside the 140/1200 ceiling. And cli.rs::refused is now called by nothing, amend having been its only user: left in place, with its doc comment on the pattern, since the crate root allows dead code and deleting it would take listed and listed_flags with it. Verified through the binary on a scratch repository: the correction goes through unclaimed, is refused under a claim naming the holder, and claim --criteria refuses with amend as the next command.
