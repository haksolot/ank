---
id: TASK-f6d5e197dab4
type: task
slug: adr-43cbb20dcbf1-names-crates-ank-cli-src-hooks
title: ADR-43cbb20dcbf1 names crates/ank-cli/src/hooks.rs, the file it governs, before it is ratified
created: 2026-10-25T08:25:34Z
author: claude-code/plan
status: done
scope:
  - crates/ank-cli/src/hooks.rs
blocked_by: [TASK-afc96307cb81]
done_criteria: |
  ank show ADR-43cbb20dcbf1 lists crates/ank-cli/src/hooks.rs in its scope while the ADR is still proposed; ank check exits 0 and reports that scope as scope-not-yet naming an open task that covers it, or as matching a file once one exists; ank context crates/ank-cli/src/hooks.rs, read by an identity holding no claim, serves ADR-43cbb20dcbf1.
criteria_by: creator
verify: [check-repo]
proof:
  - type: test
    ref: local/ed78161fa067@14954f5
    tree: scope/e3b0c44298fc
    criteria: 96f2ccc49ce7
    verifier: check-repo@5734e9cf9d3d
    via: verifier
schema: 4
version: 7
---

The dogfood of ADR-91f486072d68: the scope removed from ADR-43cbb20dcbf1 at planning time because check faulted on it. ank amend ADR-43cbb20dcbf1 --scope crates/ank-cli/src/hooks.rs. Lands before ADR-43cbb20dcbf1 is ratified, so the signature covers the real perimeter and no accept --rescope follows.
