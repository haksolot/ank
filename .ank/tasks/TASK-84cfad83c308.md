---
id: TASK-84cfad83c308
type: task
slug: ank-help-verb-is-silent-on-what-the-verb-refuses
title: ank help <verb> is silent on what the verb refuses
created: 2026-08-07T16:59:47Z
author: seanl@sean-laptop
status: in_progress
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  docs/ank-spec-v1.1.md section 9 is rewritten first: ank help <verb> carries what the verb does and the state conditions on which it refuses, with the exit code, alongside the usage, flags and globals it already carries. The top-level flat listing stays one flat listing, unchanged. ank help amend no longer presents --criteria as an available flag. ank help done names that a proof is required on state and enumerates the four accepted proof types. ank help edit names that EDITOR is a command line run through sh, not a program name. A test in crates/ank-cli/tests/cli.rs walks every verb through the binary and fails if any flag its help lists is refused unconditionally.
criteria_by: creator
schema: 2
version: 3
---

Not an implementation drift: section 9 specifies ank help <verb> as usage,
flags with their value placeholders, and globals. The current output is
exactly that. The gap is in the specification.

Section 9 bets on a division of labour across three surfaces -- SKILL.md
carries the mental model, ank help carries the flags, errors carry the next
command -- and the bet is that between them a caller never needs the source.
One session of dogfooding falsified it five times, and each recovery went
through crates/. In another repository ank arrives as a binary and SKILL.md:
no source, no specification. The same five become dead ends.

What was needed and absent, in order of severity:

amend lists --criteria among its flags while refusing it unconditionally, by
name and by design. Help does not merely say too little there, it misinforms,
and the refusal's own hint points at ank release, which a task that was never
claimed cannot run. Learned by reading human.rs.

done requires --proof on state, which no surface states; learned from exit 5.
The proof grammar <type>:<ref> and its four types -- commit, human-review,
assertion, test -- live only in done.rs, and the error gives one type by
example rather than the set.

claim --criteria overwrites an existing criterion silently and flips
criteria_by to claimer. No surface says so; found by experiment on a scratch
repository.

EDITOR is a command line run through sh -c, not a program name. Here the
self-correcting error is worse than incomplete: the hint EDITOR=vi suggests a
bare binary, and a caller who follows it with a GUI editor gets no --wait, the
editor returns immediately, ank writes the unedited file, and nothing signals
it.

The structural point is that nothing owns what a verb refuses and why. That is
the tool's own claimed identity -- section 4 carries a whole subsection on
refusals being on state -- documented for the reader of the document and
nowhere for the caller of the binary.

The test matters as much as the fix. Without something that fails in CI when a
listed flag cannot succeed, this decays back within two revisions. The shape
is the one msrv-tight already uses: a negative test that attributes the drift
rather than leaving it for a human to notice later.

## Log
- 2026-08-09T06:03:10Z seanl@sean-laptop — Read the criterion's two clauses together rather than separately. "The top-level flat listing stays one flat listing, unchanged" and "ank help amend no longer presents --criteria as an available flag" cannot both hold under a byte-identical reading, because the flat listing was one of the two places presenting it. Taken as structure -- still flat, no headings, no grouping, no summary lines added there -- both hold. Measured: the diff against published 0.1.3 is exactly one token, --criteria gone from amend's row, and nothing else on any line.

Measured every refusal before stating it, and three of the eleven I had written were wrong. close refuses 7 without --reason, not 2: the mandatory flag is checked before the id is resolved. accept refuses 9 when the default branch cannot be determined at all, which I had not listed and which is what a fresh repository with no origin actually hits. My third apparent miss was the measurement and not the claim: edit does refuse 9 for an unset EDITOR, but only once the id resolves, so probing it with a missing id shows a 2. Verified the rest by construction -- done 5 with a claim and no verifier, claim 4 held by another and 7 blocked, accept 7 off the default branch.

The test needs valid values per flag, and finding that out was the useful part. A generic "x" makes context --limit a code 1 and find --type a code 1 -- neither is a refusal, it is the verb correctly rejecting a value that is not a number and not a kind. A test reading those as refusals would have failed on four verbs that are right.

One exception survives in the invariant and it is the flag doing its job: when the baseline is a 7, a missing prerequisite, a flag may change the code, because supplying the prerequisite is exactly what close --reason and release --reason are for. Measured that those two are the only verbs where it happens rather than assuming it.

Falsified by re-listing --criteria on amend, which is the defect as it shipped: the test fails naming the verb, the flag, and both codes.
