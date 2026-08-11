---
id: TASK-9ff86a0950bf
type: task
slug: the-over-constrained-signal-reports-a-budget-it
title: The over-constrained signal reports a budget it does not test against
created: 2026-08-11T03:47:37Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  The over-constrained signal names the threshold it actually applies, so that the number a reader compares against is the number that fired. Verified through the binary: an integration test builds a corpus whose constraints exceed half the configured context_budget without reaching it, runs ank check, and asserts the reported figures are consistent -- the quantity shown and the limit shown stand in the relation the signal tested.
criteria_by: creator
schema: 2
version: 3
---

`check` reports `over-constrained scope: 5527 characters of constraint against
a budget of 8000`. 5527 is under 8000, so a reader either concludes the tool is
wrong or goes looking for the real threshold in the source. Both were measured
on this repository: the signal fires on every `check` run here, and one session
read `human.rs` to find out why.

The threshold is not the budget. The test is `weight * 2 > cfg.context_budget`
-- constraints alone eating more than *half* the budget, which the code comment
beside it states plainly and the message does not. The number the reader needs
is 4000, and the message shows 8000.

The wording is the whole defect. Nothing about the rule is wrong: half the
budget spent on constraints before a single task body is read is worth a
signal, and §5 is where that reasoning lives.

Worth fixing beyond tidiness because this signal is noise today. It fires on
seven tasks in this corpus, every reader who checks it concludes it is
miscounting, and a signal nobody believes is a signal that hides the next real
one.

## Log
- 2026-08-11T05:04:22Z seanl@sean-laptop — No specification change: section 5 already states the rule correctly, over-constrained being constraints alone consuming more than half the budget in execution mode. Only the message lied. The fix is one variable for the threshold tested and the threshold reported, not a rewording -- the test was weight*2 > context_budget and the message printed the budget, so the two numbers came from different places and were free to disagree. Now both come from the same binding and cannot. Integer division is exact here in both directions: weight > b/2 floored and weight*2 > b select the same set, so the threshold moved presentation only. The test parses the two numbers back out of the message and asserts the relation rather than the wording -- a test on a fixed string would have passed on the old message, which was well-formed and wrong. Proven to bite: restoring the old format fails with '301 characters of constraint against a budget of 400'. No existing test asserted the old wording, which is why it survived from revision c.
