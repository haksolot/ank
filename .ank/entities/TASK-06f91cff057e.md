---
id: TASK-06f91cff057e
type: task
slug: what-ank-done-costs-per-verifier-on-this-reposit
title: What ank done costs per verifier on this repository, cold and warm, and why each run was not reused, measured by a committed script
created: 2026-11-24T13:23:28Z
author: claude-code/opus-5.5+plan
status: open
scope:
  - bench/verifier-cost/**
  - .ank/config.yml
blocked_by: []
done_criteria: |
  bench/verifier-cost/ holds a script, committed, that takes the path of the repository to measure as an argument, this repository when omitted, and a work directory the caller names (never /tmp or a TMPDIR it did not set). It reads the verifier names, their inputs and which are default from that repository's .ank/config.yml, and names no verifier of this repository anywhere in its text. It clones the repository there with refs/ank/* fetched, and in a fresh worktree with an empty build directory runs, in order: (1) ank verify on the unchanged tree, cold; (2) ank verify again on the same content, warm; (3) a one-line edit to a tracked file the inputs of exactly one declared verifier match, picked by the script from config.yml, then ank verify; (4) ank done on a throwaway task declaring the repository's default verifiers; (5) ank done again after the registry entries are forged older than proof.reuse_max_age; (6) ank done --fresh; (7) a second fresh worktree on the same content, ank done; (8) that second worktree after the same one-line edit, ank done. For every verifier at every step it prints one row: step, verifier, ran or reused, the reason a run was not reused (no-entry, key-changed-whole-tree for a verifier with no inputs or inputs "**", key-changed-inputs, key-changed-definition, expired, fresh, last-failed) derived from the registry under <git-common-dir>/ank/runs/ read before and after the step and never from the scenario alone, the wall seconds ank printed, the count of rustc invocations through a counting RUSTC_WRAPPER, and the count of git processes from GIT_TRACE. Run on this repository at a commit the log names, the task's log carries the table and: how many verifiers config.yml declares, how many declare inputs, and how many of those declare "**"; whether ank verify run twice on unchanged content reruns or reuses a verifier that passed, shown by the second run's printed status and its rustc count; the share of step 4's wall time taken by its costliest verifier; the machine (cores, verifier_jobs, verifier_lock); for a commit that changes at least one file under crates/, so that the what-changed job selects the test work and does not skip it (the commit adding the script alone is not a valid sample), every CI job that ran on it with its queue time and its run time stated separately, each with where it was read (a CI log, or entered by hand by whoever read it), no forge API being called by the script or the measurement; and what narrowing test-ank-cli's inputs from "**" to the paths its crate builds and tests from (listed in the log) would save, measured by running steps 3 and 8 again with that narrowed definition: how many of its runs become reused and the wall seconds that removes. Every number in the log, CI times aside, is one the committed script printed.
criteria_by: creator
schema: 4
version: 5
---

The first task of the verifier-speed plan, and the only one not blocked: every
other task of the plan waits on these numbers, so the work that follows is
ordered by data rather than by the brief that started it.

Measured, not read (CLAUDE.md, "A fact is measured"). Process counts are the
cost measure; wall seconds are recorded for ratios only, on one machine, never
as a bound (the Windows-runner lesson).

The script is generic: it takes the repository to measure as an argument and
learns every verifier from that repository's config.yml, so it runs unchanged
on any large repository that uses ank. The run this task records is on this
repository (eight test verifiers, cargo-test at a 20m timeout, test-ank-cli
declaring inputs "**"), which is the one with real verifiers on hand.

Two numbers in the log are specific to this repository and are not the
script's business. CI wall time per job for the measured commit, read from the
CI logs or entered by hand, never through a forge API: without it, nobody can
judge whether proofs shared across machines would save more than local reuse
already does. And what narrowing test-ank-cli's inputs from "**" would save,
because that gain needs no rule change and should be weighed before any that
does.

What the numbers are for, besides ordering the tasks that follow: they decide
whether ADR-7224dea611c1 (done reuses local runs only, no environment field)
is worth reopening for cross-machine proofs. That reopening is out of scope
here and is not drafted.

Close: --no-verify, because no verifier can judge a measurement; the close is
--proof commit:<sha> of the commit that adds the script, with the log entries
as what a human reads.
