---
id: TASK-0aaf0888c9f2
type: task
slug: nothing-notices-a-done-task-that-was-never-attes
title: Nothing notices a done task that was never attested
created: 2026-08-04T17:39:22Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/human.rs
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  ank check reports a task that is done on the default branch and carries no test: proof, naming the task and the exact attest command to run. It is a signal and not a fault: exit stays 0. Asserted through the binary against a fixture corpus holding both cases, a done task with a commit: proof only and a done task with both, and the second must not be reported.
criteria_by: creator
proof:
  - type: commit
    ref: 5dcd3eb
    criteria: 813e525c9ffb
schema: 2
version: 5
---

Asked for on 2026-08-04, from the question of whether CI could attest by itself
after a green run. It can -- attest needs no claim, and github.run_id is the
same number a human pastes by hand -- and writing the proof from CI is the
version this task deliberately does not ask for.

A test: proof today asserts "I looked at this run and it proves this
criterion". Written by CI it would assert "a green run existed on a tree
containing this task". The entry carries the frozen criteria hash, so the
second statement is mechanically true and the collapse is tempting. It is
still the rollup hole of section 3, one level out: CI proves the tree passes
its tests, not that this criterion was met. The link between the two exists
only because someone wrote a test encoding the criterion, and CI cannot know
whether they did. "The parent is finished when the children are finished is
structurally the same hole as assertion:, hidden in the topology instead of
written in a field."

So invert it: the human keeps the assertion, and forgetting to make it is what
gets caught.

The cheap site is check, not the workflow. ci.yml already runs
`cargo run -q --bin ank -- check`, and signals exit 0 while faults exit 8, so a
new signal in check_task surfaces this with no workflow change, no
contents: write on the most-triggered workflow in the repo, and no commit from
CI retriggering on: push. That last set is the whole reason the auto-writer
was rejected on cost as well as on meaning: ci.yml is deliberately
permissions: contents: read, with a comment saying it publishes nothing.

Two design questions for whoever claims this.

A signal and not a fault, and the reason matters: the attest legitimately
cannot exist until the merge run has finished, so a fault would turn CI red on
every merge for a window that closes on its own. What is being reported is a
record that is stale, not a corpus that is broken -- the same asymmetry the
completion-ref signal already draws.

When it should fire. There is already a signal for "finished on another
branch, main has not caught up", which covers the window before the merge. The
new one is only meaningful after that one has gone quiet, so keying on "done on
the default branch and no test: proof" is the shape to start from -- but the
merge run itself takes minutes to go green, and a signal that fires in that gap
is noise on every single merge. Whether the answer is a grace period, a commit
distance, or simply accepting the transient is the decision to record before
implementing.

## Log
- 2026-08-04T17:46:00Z seanl@sean-laptop — Measured the blast radius before designing, with a throwaway signal rather than by guessing: 8 of the 64 done tasks carry no test: proof, all of them commit:-only. That settles the question the body left open about a backlog. I expected something near 60, which would have needed the 'predates the convention' exemption the author-field signal uses; 8 is a worklist that clears, so no exemption is warranted and the design gets simpler. Second decision, on the transient: the signal will fire on the merge commit's own CI run, in the window between the merge landing and the run going green. Accepting that rather than adding a grace period or a commit-distance constant. It is not a false positive -- the statement is true when printed and clears when someone attests -- and the alternative buys quiet at the cost of a magic number, which section 6 only justifies for the flooding thresholds. Third: gating on 'done on the default branch' is load-bearing and not decoration. Without it the signal fires on a feature branch straight after ank done, when attesting is impossible because no merge run exists yet -- a finding the reader cannot act on, which is the failure mode the weak-proof comment in human.rs was written about.
- 2026-08-04T17:52:04Z seanl@sean-laptop — The specification had already settled the design question, at section 4 line 553: 'nothing calls attest automatically. A CI provider appending its own run reference at the end of a pipeline is an integration, not a verb.' Found it while looking for where to record the new signal, after the reasoning was written rather than before -- so the detector shape was re-derived rather than read off, and the two agree. That deferral is the positive half of the same choice; this signal is the negative half, and section 4 now carries both next to each other. Also measured that no existing finding covers this case: commit is not weak, so the weak-proof signal stays silent by design on a commit:-only proof list, which is why the omission was invisible rather than merely unreported.
- 2026-08-04T17:53:57Z seanl@sean-laptop — done, proof commit:5dcd3eb
