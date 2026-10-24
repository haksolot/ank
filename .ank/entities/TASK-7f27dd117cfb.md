---
id: TASK-7f27dd117cfb
type: task
slug: a-random-entity-id-beginning-with-0000-escapes-t
title: A random entity id beginning with 0000 escapes the golden redaction and fails the run
created: 2026-10-24T11:05:10Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/tests/fixture/mod.rs
blocked_by: []
done_criteria: |
  The redaction in crates/ank-cli/tests/fixture/mod.rs masks as <HEX> every 12-hex-digit entity id the run generated, including one whose digits begin with 0000 such as ADR-0000f14329b5, and keeps verbatim the fixed ids the goldens name literally such as TASK-000000000001; a test calls the redaction on both forms and asserts each outcome, watched red before the fix; no golden under crates/ank-cli/tests/golden-json changes; the cause and the measured odds of the failure are recorded with ank log.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
method: diagnose
proof:
  - type: test
    ref: local/e3b0c44298fc@7ce3c79
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: fmt-check@695ca51e4ab9
    via: verifier
  - type: test
    ref: local/066f8136a9dd@c8f21d0
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-core@662d5656b204
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/51456532d207@c8f21d0
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-contract@12feb96cb9d5
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/8dcdb4163f6d@c8f21d0
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-daemon@a6df02a5c56c
    via: verifier
    reused: local
    by: claude-code/opus-5.5+w1fc7
  - type: test
    ref: local/3a70f2dc4fa0@7ce3c79
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-mcp@426b743c5c6b
    via: verifier
  - type: test
    ref: local/dadb8d0cf736@7ce3c79
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-tui@5df657e6f69e
    via: verifier
  - type: test
    ref: local/489970884283@7ce3c79
    tree: scope/3014c4868be7
    criteria: 56b4cc26b3ea
    verifier: test-ank-cli@95de94ea57e3
    via: verifier
schema: 4
version: 3
---

Seen on PR #676, macos-latest job 120831674607 (run 40080086448): tui::the_json_frame_is_pinned_by_a_golden failed because .entities[1].id was ADR-0000f14329b5 where the golden pins ADR-<HEX>. fixture/mod.rs redacts a whole 12-hex word only when it does not start with 0000, an exclusion that keeps fixed ids readable and also exempts any random id with that prefix, about one in 65536 per id. A rerun went green. Distinguish fixed from generated ids by something other than a prefix a random id can share.
