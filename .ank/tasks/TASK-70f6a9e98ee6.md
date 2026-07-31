---
id: TASK-70f6a9e98ee6
type: task
slug: a-weak-proof-that-no-append-can-clear-is-noise-n
title: A weak proof that no append can clear is noise, not a finding
created: 2026-07-31T21:50:43Z
status: done
scope:
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  check signals a weak proof only when the task carries no strong proof beside it. A task holding both an assertion and a test entry is reported clean; a task holding only weak entries is still signalled, with the same wording. A test covers all three shapes -- weak alone, strong alone, weak and strong together -- and asserts the finding rather than the absence of a crash.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/801a413e25ec@3969211
    tree: scope/fb4254f1bce9
    criteria: 7730e10d6347
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@3969211
    tree: scope/fb4254f1bce9
    criteria: 7730e10d6347
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/2e266865aec7@3969211
    tree: scope/fb4254f1bce9
    criteria: 7730e10d6347
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 5
---

Found while planning TASK-c2fae25adc66, whose criterion cannot be satisfied
while this stands. `check_task` walks the proof list and signals every weak
entry it finds, with no condition on what else the task carries:

    for p in &t.proof {
        if p.proof_type.is_weak() {
            report.findings.push(Finding::signal(
                &t.id, format!("weak proof '{}': it anchors nothing", ...)));

§3 makes the proof list append-only, and TASK-85e6bbb195b8's reasoning forbids
substituting an entry to make history look better. Put together, the two mean a
task closed before `ank done` existed can **never** clear this signal: the
assertion has to stay, and the assertion is what fires. A finding nobody can
act on is not a finding — it is a line every reader learns to skip, which is
how a corpus goes from fifteen signals to fifteen signals nobody reads.

The signal is worth keeping for what it was meant to say: *this task's
completion rests on nothing verifiable*. That statement is false the moment a
test proof sits beside the assertion, and true while it does not. The condition
belongs on the task, not on the entry.

Wording unchanged on purpose. The message is right; it is the trigger that is
wrong, and a reader who has seen it before should not have to learn it twice.

## Log
- 2026-07-31T21:53:46Z seanl@sean-laptop — The condition moved from the proof entry to the task: check now signals only when every entry is weak. Read on the entry it was unclearable by construction -- section 3 makes the list append-only and ADR-85e6bbb195b8 forbids rewriting an entry, so a task closed before ank done existed could never drop the signal, because the assertion has to stay and the assertion was what fired. One finding per task rather than one per entry, since the task is what is being judged. Wording untouched. The corpus still reports TASK-a1b2c3d4e5f6 as weak, correctly: it has no strong proof yet, which is TASK-c2fae25adc66's job.
- 2026-07-31T21:54:09Z seanl@sean-laptop — done, proof test:local/801a413e25ec@3969211 test:local/e3b0c44298fc@3969211 test:local/2e266865aec7@3969211
