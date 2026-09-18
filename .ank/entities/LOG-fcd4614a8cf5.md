---
id: LOG-fcd4614a8cf5
type: log
title: "Diagnosed and fixed, measured rather than argued. The red was a flake: the windows-latest job of"
created: 2026-09-18T16:03:31Z
author: claude-code/opus-5+noop-clock
scope:
  - crates/ank-cli/tests/renewal_noop.rs
about: TASK-faf554e366c5
seq: 1
schema: 4
version: 1
---

 run 35358187023 on 298c01a failed with 'ten claims never shared a second with their renewal', and a rerun of that same job on that same commit passed, while f89a9bf -- the head of the branch 298c01a merged, same tree -- had passed minutes earlier and the eleven CI runs before it were green. So nothing in TASK-cf81d7d57d1e caused it. The mechanism is the test's: a renewal is a no-op only when it lands in the wall-clock second of the write it is compared against, and the old loop reached that by claiming and hoping ten times, where each attempt was decided by the phase of the second at which claim happened to write its record. cross_a_second ran before the claim and aligned nothing, since the claim then took a whole verb, push included at level 1, to reach that write. The fix arranges the state instead: forge_expiry, a generalisation of the forge_expiry_ahead the test already had, writes onto the ref and the origin the expiry a renewal in a named second computes, and wait_for_second enters that second at its start, so one context has a whole second to reach its renewal. The twenty-character stamp is restated in the test because the binary exposes no library, so assert_stamp_matches_the_binary confronts it with a stamp the binary wrote before anything rests on it. Measured on this box: 25 consecutive runs of the test, 25 passes and 0 failures, against a red that reproduced once in twelve on a Windows runner. The recorded process counts are unchanged and still printed under --nocapture: unchanged renewal 5, changing renewal 14, verb before its renewal 5. cargo test --workspace exit 0, cargo fmt --check exit 0.
