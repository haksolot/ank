---
id: TASK-7ebfabed5ddc
type: task
slug: bench-family-one-binding-decision-among-500-foun
title: "Bench family: one binding decision among 500, found by the path it applies to"
created: 2026-10-26T17:44:59Z
author: haksolot@vmi3223161
status: open
scope:
  - bench/**
blocked_by: [TASK-8d66b1c12820]
done_criteria: |
  bench carries a scenario whose corpus holds 500 decisions, exactly one of which applies to src/shop/cart.py and refuses a cart reduction above three tenths in words that neither the method name nor the word discount reach by grep; under markdown that file names src/shop/cart.py in its Applies to line, under ank its scope; the same seed writes byte-identical trees twice; ank check exits 0 on the ank tree and ank context src/shop/cart.py there lists that decision; the probe is the adoption probe; a dry run per condition exits 0.
criteria_by: creator
verify: [bench-test]
schema: 4
version: 1
---

Planned 2026-10-26: 48 Markdown decisions were read whole by every model in the adoption campaign, so retrieval never mattered. At 500 reading everything stops being an option. ADR-9e7cf2255bb7 governs.
