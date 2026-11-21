---
name: ank-diagnose
description: Work a defect back to its cause before changing anything, and close it with a regression test. Use when a claimed task's criterion names a defect in a repository with a .ank/ directory.
metadata:
  revision: "98cd5d5badff"
---

# ank-diagnose

A criterion that names a defect names something somebody observed. It does not
name the cause, and the distance between the two is the whole of this file.

The ank skill is the contract and applies here in full. This file adds the
diagnosis policy only.

## The loop

    reproduce      make it happen on demand, from a command you can rerun
    minimise       cut the reproduction until nothing left in it can go
    hypothesise    one cause the minimal case explains, and what would refute it
    instrument     measure until the hypothesis is confirmed or dead
    fix            change the cause, and nothing beside it
    close          a regression test that goes red without the fix, then `ank done`

The order is the method. Each step hands the next one something it could not
otherwise have had, and a step skipped is paid for at the close, where the
price is a green suite over a defect still in the tree.

## Reproduce

Until you can make it happen you hold a report and not a defect. Produce a
command that fails every time, and write it down whole: the invocation, the
environment it needs, and the failure exactly as it prints. That command is
what the fix will be measured against and what the regression test is derived
from, so a paraphrase of it costs you both.

A defect that will not reproduce is a finding rather than a task. `ank log`
what you ran and what came back instead, then `ank release --reason "<why>"`.
A criterion wrong about what the tree does is wrong, and softening it is the
one repair the contract forbids.

## Minimise

Take the reproduction apart. Remove one element -- a flag, a file, a step, a
platform -- and rerun; keep the removal if it still fails, put it back if it
does not. What survives is the statement of the defect, and everything you took
out is noise you would otherwise have built a hypothesis about.

A removal that makes the failure stop is not a dead end. It has just named
something the cause needs, which is the first real fact of the session. Log it.

## Hypothesise

One cause at a time, phrased so that it could be wrong. *The status cache is
keyed on file state alone and the orphaned claim lives in the refs* is a
hypothesis. *Something is off in the caching* is a mood: nothing refutes it, so
every observation will appear to confirm it.

Name the observation that would kill it before you go looking. Where several
survive, order them by what is cheapest to refute and never by what feels
likeliest -- the ranking you trust is the one that put you here.

## Instrument

**A fact is measured, not read.** Reading the code and concluding is how a
wrong hypothesis survives contact with evidence: you find the line that agrees
with you, and the line that would not have agreed is never executed. Run it and
count what comes back. `GIT_TRACE` at an absolute path prints one line per
process, so processes are counted; an empty `TMPDIR` and then a listing says
what the suite left behind; a pseudo-terminal and then a frame comparison says
what the binary drew.

`ank log` what you observed while you hold it, and log the observation rather
than the verdict: the command you ran, the numbers it printed, expected against
actual. *The key omitted the ref tip* is a conclusion the next reader has to
take on faith. *Key computed over 41 files, ref tip 8f2c not in it, stale
verdict served on the second call* is a measurement they can reach a different
conclusion from, which is what they will need if yours was wrong.

A refuted hypothesis is the ordinary outcome and worth exactly as much as a
confirmed one. Log it too, then go back one step with a smaller space to search.

## Fix

Only once a measurement confirmed something. Change the cause, where the
measurement pointed, and stop there. A second defect the diagnosis exposed is a
new task with `blocked_by`, never a widening of this diff; the criterion you
froze at claim is still the one you are answering.

## Close with a regression test

The test is derived from the minimal case, not from the original report: the
report carries the noise you spent the second step removing.

**Verify it goes red without the fix.** Undo the change, run the test, watch it
fail with the failure from the first step, restore. A regression test that has
never been seen red asserts the tree as it happens to be, and it will stay
green through the return of the very defect it was written for. Then `ank done`
runs the declared verifiers and records the proof.

## Patch-first guessing

The anti-pattern, and the reason the loop is written down. The defect is read,
a plausible cause is guessed out of the source, an edit is made, the symptom
stops, the task closes. It is faster than everything above, and it costs three
things.

The symptom can stop because it moved. Ordering, timing, one caller of three,
a platform not in the suite: the change perturbed the conditions and the cause
is still there, now with a passing test standing over it.

Nothing was measured, so nothing outlives the session. The next holder finds a
diff and a green suite, and has to redo from the report every step you skipped.

The regression test comes last and is written against the fix. Never confronted
with a red, it encodes the behaviour you just produced rather than the defect
you were sent after, and it is green by construction.

When you notice you are editing and no measurement has confirmed anything, you
are here. Go back to hypothesise.

## Where the method stops

No verb enforces any of this. `done` runs the declared verifiers and measures
the tree as it stands; the order you worked in is not a fact it holds, and
nothing asks it to. There is no route to grade, deliberately: an agent graded
on its process learns to produce the process.

So what you hold at the close is what you measured. A cause located, logged
with the numbers that located it, and pinned by a test somebody watched fail --
that survives the session. A symptom that stopped appearing survives until the
conditions move again, and so does whoever inherits it.
