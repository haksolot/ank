---
id: LOG-d89c0219fae2
type: log
title: how I convinced myself somewhere other than one green suite, since that is what failed last time.
created: 2026-08-15T13:08:33Z
author: claude-code/log-entities
scope:
  - crates/ank-core/**
  - crates/ank-cli/**
  - docs/**
about: TASK-166626ed8095
seq: 3
schema: 3
version: 1
---

 Three measurements rather than one run. First, a harness outside the test suite that writes four entries about one task through the binary and reads them back: 12 runs of 12 shared a second and 0 came back in the wrong order, against 10 of 12 wrong before. Second, the falsification: removing seq from the order key makes the ordering tests fail on 6 runs out of 6, where the old tests failed about 1 run in 3 and passed on Windows CI by luck. A guard that fails every time is evidence, a guard that fails a third of the time is a coin. Third, repeated runs of the tests themselves, 12 of 12 green, which is what caught the last flake: the test helper log_text sorted by created and the identifier and reproduced the exact defect it exists to catch, failing 1 run in 10 until it was taught seq. Also fixed on the way: the fixture repository had no gitignore for index.db, and since close, attest, amend and done now open an index to compute a rank, a merge in one test began refusing over an untracked file the tool owns.
