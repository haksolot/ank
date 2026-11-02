---
id: TASK-7ebfabed5ddc
type: task
slug: bench-family-one-binding-decision-among-500-foun
title: "Bench family: one binding decision among 500, found by the path it applies to"
created: 2026-10-26T17:44:59Z
author: haksolot@vmi3223161
status: done
scope:
  - bench/**
blocked_by: [TASK-8d66b1c12820, TASK-d529b62e91b7]
done_criteria: |
  bench carries a scenario whose corpus holds 500 decisions, exactly one of which applies to src/shop/cart.py and refuses a cart reduction above three tenths in words that neither the method name nor the word discount reach by grep; under markdown that file names src/shop/cart.py in its Applies to line, under ank its scope; the same seed writes byte-identical trees twice; ank check exits 0 on the ank tree and ank context src/shop/cart.py there lists that decision; the probe is the adoption probe; a dry run per condition exits 0.
criteria_by: creator
verify: [bench-test]
proof:
  - type: test
    ref: local/048d7e618e4e@c9e2db6
    tree: scope/45ede928fed9
    criteria: 2de323b86431
    verifier: bench-test@290b04030828
    via: verifier
schema: 4
version: 4
---

Planned 2026-10-26: 48 Markdown decisions were read whole by every model in the adoption campaign, so retrieval never mattered. At 500 reading everything stops being an option. ADR-9e7cf2255bb7 governs.
