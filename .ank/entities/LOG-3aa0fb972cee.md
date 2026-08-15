---
id: LOG-3aa0fb972cee
type: log
title: Read the criterion's two clauses together rather than separately. "The top-level flat listing stays
created: 2026-08-09T06:03:10Z
author: seanl@sean-laptop
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-84cfad83c308
seq: 0
schema: 3
version: 1
---

 one flat listing, unchanged" and "ank help amend no longer presents --criteria as an available flag" cannot both hold under a byte-identical reading, because the flat listing was one of the two places presenting it. Taken as structure -- still flat, no headings, no grouping, no summary lines added there -- both hold. Measured: the diff against published 0.1.3 is exactly one token, --criteria gone from amend's row, and nothing else on any line.
