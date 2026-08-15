---
id: LOG-5d366b3bba6a
type: log
title: "Direction settled with the maintainer: amend opens on state, claim closes. amend --criteria is"
created: 2026-08-11T01:25:27Z
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
schema: 3
version: 1
---

 allowed unless a live claim anchors the criterion, leaves criteria_by untouched, and is logged; claim --criteria stops overwriting a criterion that already exists and points at amend, so it only ever sets an absent one -- which is the case section 3 designed it for, and which keeps criteria_by:claimer meaning exactly one thing. Discovery that shrinks the work: edit.rs already implements this rule. check_frozen refuses a moved done_criteria only when live_claim_anchor finds a claim in force, and returns Ok otherwise, with the comment saying refusals are on state and never on identity. So a route through the CLI already existed -- ank edit -- and it was never scriptable and never named in the specification. amend will reuse that helper rather than restate the rule: it moves to claim.rs, where both callers can reach it.
