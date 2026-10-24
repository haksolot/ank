---
id: LOG-7f26bc7a926a
type: log
title: "fix: is_seeded(word) = membership in the set of whole 12-hex words in tests/golden-json/*.json,"
created: 2026-10-24T11:50:17Z
author: claude-code/opus-5.5+w7f27
scope:
  - crates/ank-cli/tests/fixture/mod.rs
about: TASK-7f27dd117cfb
seq: 3
schema: 4
version: 1
---

 read once per process (OnceLock); no prefix is consulted. Measured: regression test red on the old rule (left ADR-0000f14329b5, right ADR-<HEX>), seen twice, once on the final file with only the condition reverted; green with the fix. ANK_BLESS_GOLDEN=1 over the whole ank-cli::integration binary (766 passed, -j2) rewrote all 34 goldens with git status empty for tests/golden-json and 0 non-.json files left. Review, criterion axis: every clause points at a hunk; one hunk the criterion did not name, pin writing via <name>.json.<pid>.tmp then rename, kept because under nextest each test is a process that reads the directory for literals while others bless, and a half-written golden would drop a seeded id from that reader's set. A bless-time prefix fallback was drafted and removed: it made the regression test fail under bless. Constraint axis: ank context on fixture/mod.rs again, 7 constraints; ADR-2b62 is the one touched, doc_replay.rs masks over fixture::redacted and passed in both full runs; its own 12-digit zero-prefix check now only sees words redacted kept, so the minted case is covered upstream. Nothing else found.
