---
id: LOG-28cddd2b7155
type: log
title: Cause measured (TASK-c84e1300be5a). doc_replay::is_minted takes a 12-hex word with four leading
created: 2026-11-16T16:32:08Z
author: claude-code/opus-5.5+task-fffd-flakes
scope:
  - crates/ank-cli/tests/doc_replay.rs
  - docs/format.md
about: TASK-c84e1300be5a
seq: 1
schema: 4
version: 1
---

 zeros for a seeded id; fixture::redact was fixed for the same odds (TASK-7f27dd117cfb) and masks it as minted. So when the replay mints TASK-0000xxxxxxxx (1 in 65536 per id), masked(page)==masked(replay) on 'created TASK-5f1e0a9c2b7d Migrate auth' (both TASK-<HEX>), the step passes, but minted(replay) drops the id, learn pairs nothing, and translate leaves 'ank claim TASK-5f1e' untranslated: 'entity not found: TASK-5f1e', as in run 43857426507 attempt 1. Measured with doc_replay::a_minted_id_with_the_zero_prefix_is_paired_and_translated, replay id TASK-00003c7e91a4: masks equal (assert passed), translate returned 'ank claim TASK-5f1e' against expected 'ank claim TASK-0000'. Red before the fix.
