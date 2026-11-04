---
id: LOG-b67497d2cedf
type: log
title: "Measured through the binary: before the change, check on a claim record carrying"
created: 2026-11-03T13:40:06Z
author: claude-code/opus-5.5+task-04a6-ref-record-compat
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/tests/ref_record_compat.rs
  - crates/ank-cli/tests/main.rs
about: TASK-04a688353ae4
seq: 1
schema: 4
version: 1
---

 added_by_a_later_ank exits 8 with 'unreadable record ... unknown field' (same for the completion record; the proof record was not reached). After dropping deny_unknown_fields from ClaimRecord, CompletedRecord and ProofRecord, ref_record_compat 3/3 green: check output and exit identical with and without the field for all three records, status exit 0 naming the holder, a second identity's claim exit 4 naming it, done closes; no state / unknown state / missing holder still exit 8 naming the damage (that test was green before the change, existing behaviour). Left strict on purpose: the nested PeerSeen and AttestedProof (and ank_core::Proof, outside this scope) -- the criterion is top-level fields; a field added inside a nested entry still breaks older readers.
