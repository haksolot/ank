---
id: ADR-3b6ba766a42e
type: adr
slug: a-supersession-does-not-land-while-the-workspace
title: A supersession does not land while the workspace still cites what it retires
created: 2026-08-25T23:23:36Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
constraint: |
  accept refuses a supersession while any tracked file outside a .ank/ directory still cites the document that supersession retires. The refusal is computed before anything is written, names every site with its line, and gives the two ways out: write the successor instead, or drop the citation and leave the history to ank show. It carries the code section 4 gives a missing prerequisite, it is not suppressed by --quiet, and it takes no bypass flag -- accept has none and gains none.
  
  check reports the same condition as a fault, over the same walk, so a corpus can be judged without cargo and on any platform. One implementation answers both: the verb and the verifier never disagree about whether a citation is stale.
  
  A citation naming a proposed successor is not a defect and is never reported as one. It is the state this refusal exists to produce: it is honest, because ank show reports that document as proposed, and it becomes correct at the signature that follows -- where a citation of the retired document becomes wrong at that same instant.
ratified: 67e3a44bb66f
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-26T00:07:29Z
schema: 4
version: 2
---

`accept` already computes this. It walks the tracked files, skips every `.ank/`
wherever it sits, skips what is not text, and prints the sites it found. What it
does with the answer is the defect: it prints it as a warning, *after* the commit
that made it true, and `--quiet` suppresses it entirely.

## Measured three times in one day

ADR-372b82af1ec7 and ADR-e39a44f80e0e were ratified and `main` went red on all
three platforms; TASK-cf075f0e287d repaired it afterwards. SPEC-20357e21a45a was
caught in time only because TASK-d86955470592 had been filed by hand for the
purpose. ADR-0c8ab846d262 was ratified and `main` went red again, nineteen
citations in nine files, with the repair already filed and not yet landed.

Three ratifications, two red branches, and the difference between them was
whether somebody remembered. That is the shape of a rule that belongs in the
tool.

## Why a refusal and not a louder warning

`accept` is the one verb that writes into history rather than into the working
tree, and it is authoritative the instant it exists, on the branch where it
exists. There is no pull request between it and the default branch and no CI in
front of it -- which is exactly why the branch precondition is a refusal and not
advice. The condition here has the same shape: something must be true *before*
the commit, because after it there is nothing to review and no gate left to fail.

A warning cannot be the answer for the same reason. It arrives after the fact,
and it is silenced by a flag somebody running in a script has every reason to
pass.

## The order this makes mandatory, and the position it reverses

The refusal forces one order: re-point the citations, then sign. That was argued
against twice today, on the grounds that the workspace would name as binding a
document nobody has signed, and the argument was wrong.

A citation naming a proposed successor is provisional and honest -- `ank show`
answers `status: proposed`, so a reader following it learns exactly where the
document stands. A citation naming the retired one is not provisional: it
becomes false at the signature and stays false until somebody sweeps it. Between
a citation that is briefly incomplete and one that will be wrong, the choice is
not close, and the window is closed by the very act that follows it.

## check carries it too, over one walk

The guard that has been failing the build lives in `crates/ank-cli/tests/cli.rs`
and runs only under `cargo test`, over the crates alone. `ank check` -- the verb
whose stated job is the mechanical invariants -- reported `0 faults` on a red
tree, twice, because it has never looked. A corpus that can be validated without
cargo is one of the things `check` is for, and its walk reaches `docs/`,
`.github/` and the installers, which the test never did.

One walk serves both. Two would be two answers to "is this citation stale", and
the day they disagreed the one nobody ran would be the one that was right.
