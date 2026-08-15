---
id: LOG-54f9ae737e3e
type: log
title: "No specification change: section 5 already states the rule correctly, over-constrained being"
created: 2026-08-11T05:04:22Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
about: TASK-9ff86a0950bf
seq: 0
schema: 3
version: 1
---

 constraints alone consuming more than half the budget in execution mode. Only the message lied. The fix is one variable for the threshold tested and the threshold reported, not a rewording -- the test was weight*2 > context_budget and the message printed the budget, so the two numbers came from different places and were free to disagree. Now both come from the same binding and cannot. Integer division is exact here in both directions: weight > b/2 floored and weight*2 > b select the same set, so the threshold moved presentation only. The test parses the two numbers back out of the message and asserts the relation rather than the wording -- a test on a fixed string would have passed on the old message, which was well-formed and wrong. Proven to bite: restoring the old format fails with '301 characters of constraint against a budget of 400'. No existing test asserted the old wording, which is why it survived from revision c.
