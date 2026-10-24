---
id: LOG-e6aeefeb2562
type: log
title: "reproduced: redacted(\"ADR-0000f14329b5\") returns it unchanged; the regression test"
created: 2026-10-24T11:42:46Z
author: claude-code/opus-5.5+w7f27
scope:
  - crates/ank-cli/tests/fixture/mod.rs
about: TASK-7f27dd117cfb
seq: 2
schema: 4
version: 1
---

 fixture::a_minted_id_is_masked_whatever_its_digits_and_a_seeded_one_is_kept fails left ADR-0000f14329b5 right ADR-<HEX>. Cause: redact() masks a whole 12-hex word only when !word.starts_with("0000"), and a minted id is 48 random bits, so 1/65536 of them carry that prefix. Measured with a temporary ANK_REDACT_LOG append in redact over one full nextest run of ank-cli::integration (766 tests, -j2, no-fail-fast): 429 twelve-hex words reached the redaction, 125 distinct; 109 distinct minted (non-0000), 15 distinct seeded (000000000001-5, 0000000000ab/ba/cc/cd/ef/fe, 00000000f001/f002/f003/f01d) plus the test's own literal. Odds: 1-(1-1/65536)^109 = 0.166% per run, about 1 in 600, about 1 in 200 per CI push across three platforms. The 15 seeded words are exactly the distinct 12-hex literals in tests/golden-json, so membership in that set separates them without a prefix.
