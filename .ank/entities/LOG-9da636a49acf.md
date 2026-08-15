---
id: LOG-9da636a49acf
type: log
title: "No specification change: section 3 already specifies the three-way answer, and the binary collapsed"
created: 2026-08-11T05:58:00Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/**
about: TASK-5bd23835d5a0
schema: 3
version: 1
---

 it to one refusal. What made the defect two defects is that done.rs::resolve_head and commands.rs::held_by each scanned the refs themselves with the same holder-and-not-expired filter, so the same rule was wrong twice. Both now call claim::on_task, which returns the lapsed case rather than dropping it, preferring a live claim over a lapsed one so the tie is broken rather than left to ref order. Two subtleties worth recording. Re-acquisition carries the anchors over and never recomputes them: re-freezing the criterion would erase a divergence introduced while the claim was down, which is the one thing done checks the hash to catch. And done retakes before running verifiers rather than relying on the compare-and-swap at completion, so a ten-minute suite does not run on a task another agent is free to claim underneath it; losing that CAS reports through claim::taken_over_since, which is lost_the_race exposed rather than a second wording of the same answer. log needed nothing beyond seeing the lapsed record: the renewal it already performs is the re-acquisition. Status was changed because this change changed it: it used to say 'no claim' where done now works, so it names the lapsed state in words and --json carries a lapsed field, rather than leaving a past timestamp for the reader to compare against their own clock. The test forges the expiry an hour into the past instead of waiting, because expiry carries a two-minute drift tolerance on top of the TTL and the shortest honest wait would exceed the whole suite's runtime.
