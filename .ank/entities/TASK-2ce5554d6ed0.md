---
id: TASK-2ce5554d6ed0
type: task
slug: a-shallow-clone-cannot-explain-a-dead-scope-and
title: A shallow clone cannot explain a dead scope, and check says so instead of faulting
created: 2026-08-13T17:28:47Z
author: claude-code/2.1.229+main-checkout
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
  - .github/workflows/ci.yml
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  check tells a history that says there was no rename from a history that is not present. On a shallow repository an unexplained dead scope that would fault is a signal instead, worded as unverifiable here and naming the command that deepens the clone; it is never reported as a rename that did not happen, and never as a corpus defect.
  
  The question is asked of git once per invocation, not once per dead scope, and only where a dead scope is already unexplained: a healthy corpus spawns no extra process.
  
  The jobs of .github/workflows/ci.yml that run ank check check out with the history those questions need, so the pipeline verifies rather than reporting that it cannot.
  
  Section 4 of docs/ank-spec-v1.1.md carries the third state before the code moves, beside the two severities it already states.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs: a repository whose history holds the rename, cloned to depth 1, exits 0 with the unverifiable wording and no claim about where the path went; the same repository cloned whole exits 0 naming the destination; and a deleted file in a whole clone still exits 8, so the fault is not lost.
criteria_by: creator
schema: 3
version: 2
---

A defect introduced by TASK-27cf26cbc414, measured on the pipeline within the
hour: PR #108 went red with six faults and no notes at all, because
`actions/checkout@v5` clones to depth 1 and there is no history to walk.

**The severity of a corpus finding now depends on how the repository was
cloned.** The same corpus reads healthy in a working clone and broken in CI.
That is the property to remove, and the CI setting is only half of it -- any
consumer with a shallow clone gets the same wrong answer.

**The project already settled this exact question, in the opposite direction,
and that precedent is the design.** `ratification_at` faces the same three-way
split: intact, altered, or a history that cannot answer. TASK-03eaa26bddd1
recorded the reasoning when it chose the third state -- *a shallow clone, a
rewritten history, a corpus moved between repositories: none of them is a broken
freeze, and a check that cries divergence over a shallow clone is a check people
learn to ignore.* A dead scope is the same shape and deserves the same answer.

`git rev-parse --is-shallow-repository` is the direct question and `rev-parse` is
already in the PLUMBING allow-list, so nothing new is admitted there. Do not try
to infer shallowness from an empty `rev-list` result: an empty result also means
a path git genuinely has nothing to say about, and collapsing the two is how the
unverifiable state would silently become the explained one.

**Once per invocation, and only when something is already dead.** The cost clause
of ADR-97beaf55e73a stands: a healthy corpus pays nothing, and a corpus with
eight dead scopes must not spawn eight `rev-parse` processes. The memo pattern
next to `RATIFICATIONS` in `git.rs` is the shape, keyed on the working directory
for the same reason.

**The wording must not become a fourth thing to read.** It says the history here
cannot answer and names `git fetch --unshallow`. It must not suggest the file was
deleted, must not suggest it moved, and must not imply the corpus is at fault --
the same silence rule `scope_moved` already carries, applied to a state that now
has a name.

The CI half is not a workaround and should not be written as one. `ank check`
walks history by design, so a job that runs it needs history: `fetch-depth: 0` on
the checkouts of the jobs that call it, which is the test job and the attest job.
Without it the pipeline would report "cannot verify" forever and verify nothing,
which is a worse failure than the red it replaces -- it is green and empty.

Both halves ship together for that reason. The code half alone makes CI
permanently unable to check; the CI half alone leaves every shallow consumer with
a corpus that reads broken.
