---
id: LOG-3dce22b0f0cd
type: log
title: "Chose the first branch of the criterion: the lease is a property of the claim, recorded in the"
created: 2026-08-11T04:23:09Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/claim.rs
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/tests/**
about: TASK-1b45f41e7b99
seq: 0
schema: 3
version: 1
---

 record, and every renewal recomputes from it re-capped by claim_ttl_max at renewal time. Deriving it was ruled out rather than overlooked -- a renewal moves expires and leaves claimed alone, because check reads claimed to report blockers the holder created after taking the task, so expires minus claimed stops being the lease at the first renewal. Maintainer waived the cross-version compatibility cost explicitly, nobody outside this repository depends on the tool yet; the field is serde default anyway, so a record written before it reads as the default, which is the promise section 7 now states. Section 7's description of the record was wrong in a second way and is corrected: it never named the claim timestamp either, which check has read all along. Test proven to bite before being trusted: reverting the one line reproduces 7199s at claim and 1800s after the log, and the failure message names both. One existing unit test had to change its assertion -- log_renews_the_ttl asserted after > before, which passed for the wrong reason, the renewal always writing the thirty-minute default over a sixty-second lease; with the lease honoured both sit sixty seconds from instants that are usually the same second, so it now asserts the recomputation rather than clock granularity.
