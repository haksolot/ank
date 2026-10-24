---
id: TASK-7f27dd117cfb
type: task
slug: a-random-entity-id-beginning-with-0000-escapes-t
title: A random entity id beginning with 0000 escapes the golden redaction and fails the run
created: 2026-10-24T11:05:10Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/tests/fixture/mod.rs
blocked_by: []
done_criteria: |
  The redaction in crates/ank-cli/tests/fixture/mod.rs masks as <HEX> every 12-hex-digit entity id the run generated, including one whose digits begin with 0000 such as ADR-0000f14329b5, and keeps verbatim the fixed ids the goldens name literally such as TASK-000000000001; a test calls the redaction on both forms and asserts each outcome, watched red before the fix; no golden under crates/ank-cli/tests/golden-json changes; the cause and the measured odds of the failure are recorded with ank log.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
schema: 4
version: 1
---

Seen on PR #676, macos-latest job 120831674607 (run 40080086448): tui::the_json_frame_is_pinned_by_a_golden failed because .entities[1].id was ADR-0000f14329b5 where the golden pins ADR-<HEX>. fixture/mod.rs redacts a whole 12-hex word only when it does not start with 0000, an exclusion that keeps fixed ids readable and also exempts any random id with that prefix, about one in 65536 per id. A rerun went green. Distinguish fixed from generated ids by something other than a prefix a random id can share.
