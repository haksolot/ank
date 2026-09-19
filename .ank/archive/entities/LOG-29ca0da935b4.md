---
id: LOG-29ca0da935b4
type: log
title: "Falsified before it was trusted: with the two filters reverted, a_proposed_spec_alone and"
created: 2026-08-19T07:28:07Z
author: claude-code/2.0
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/status.rs
about: TASK-73e81a8a804d
seq: 0
schema: 3
version: 1
---

 a_corpus_holding_both_kinds go red while a_proposed_adr_alone stays green, so the fix widens the queue rather than moving it. Replayed end to end on the corpus that produced the defect, this repository at 0e3172b where SPEC-4eff92fd80ce and SPEC-80bff12ceae8 sat proposed: ank 0.4.0 answers 'nothing proposed for ratification' and 'queue 0 proposal(s)', the rebuilt binary names both documents and counts 'queue 2 proposal(s)'. LIVE CONSTRAINTS is unchanged in both, which is the clause about the live section, measured rather than read.
