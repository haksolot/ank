---
id: TASK-ed497c76f50a
type: task
slug: adoption-family-a-campaign-over-three-models-and
title: "Adoption family: a campaign over three models and three conditions, committed and summarised"
created: 2026-10-26T13:00:45Z
author: haksolot@vmi3223161
status: open
scope:
  - bench/**
blocked_by: [TASK-917a8f9a1440, TASK-9ae9b0959c9d]
done_criteria: |
  bench/ carries the adoption scenario (a generated corpus whose one binding decision is not findable by grepping the method name, a feature request that never mentions ank) under the three conditions of ADR-bafbd28498cc; bench/results/ holds a JSONL campaign of at least five runs per cell for Haiku, Sonnet and Opus under no record, plain Markdown and ank, every line passing the scorer's schema check; bench/README.md tables constraint compliance, CLI use, hand reads of .ank/ and turns per cell, and says how to replay the campaign.
criteria_by: creator
verify: [bench-test]
schema: 4
version: 2
---

The campaign spends quota: run it after TASK-917a8f9a1440 lands, with the ank built from main, so the ank condition measures what init writes rather than a hand-written block.
