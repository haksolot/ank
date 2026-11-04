---
id: LOG-22ada0f79d8e
type: log
title: "discrepancy for planning: the criterion limits each body diff to the named passages, so the"
created: 2026-10-24T18:19:59Z
author: claude-code/opus-5.5+w7956
scope:
  - docs/specification.md
  - crates/ank-contract/src/verbs.rs
  - crates/ank-tui/src/view.rs
  - crates/ank-cli/tests/skill.rs
about: TASK-79566ec42789
seq: 8
schema: 4
version: 1
---

 successors carry forward two sentences that still state the old rule ADR-bb9ccdba9be7 item 2 changes, and will contradict the new passages once ratified: SPEC-66d5 (CLI surface) paragraph on amend, 'it refuses the scope of an accepted ADR or spec ... changing a ratified decision is a succession' (predecessor body line 126), and its verb list (line 248) shows accept --reanchor and no accept --rescope; SPEC-ed5f (data model) line 377 'where --scope on the same document is refused' and line 333 'the accepted -> superseded transition is the only legal write on an accepted ADR'. Left as written, per the frozen criterion; a further successor or a widened criterion is owed before the signature.
