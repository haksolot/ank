---
id: LOG-0fdd210244fb
type: log
title: "Bound written: the ci override naming the cold-readers test now carries slow-timeout { period 60s,"
created: 2026-11-22T09:01:35Z
author: claude-code/opus-5.5+task-3dcf-cold-readers-bound
scope:
  - .config/nextest.toml
about: TASK-3dcf8cfc2111
seq: 2
schema: 4
version: 1
---

 terminate-after 20 } = 1200s; the profile's 600s stays for the rest. 'cargo nextest list --profile ci -p ank-cli --test integration' parses it, and the test passes under --profile ci with ANK_TEST_COLD_ROUNDS=1 (1 round, 0 refused of 8, 24.83s on this box). That the bound reaches that one test and no other is counted, not read: with a copy of the file whose two periods are 1s instead of 60s (--config-file), the cold-readers test at ANK_TEST_COLD_ROUNDS=3 printed 19 SLOW lines and was TIMEOUT at 20.190s (20 periods), and cli::every_ratification_signature_is_verified_in_one_call in the same run 9 SLOW lines and TIMEOUT at 10.071s (10 periods). Follow-up for the per-round cost: TASK-0ba558fe7b4b.
