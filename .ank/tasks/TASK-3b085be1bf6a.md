---
id: TASK-3b085be1bf6a
type: task
slug: the-msrv-job-hardcodes-the-number-the-manifests
title: The msrv job hardcodes the number the manifests declare
created: 2026-08-04T18:17:37Z
author: seanl@sean-laptop
status: in_progress
scope:
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  The msrv job derives the toolchain it installs and builds with from rust-version in the manifests rather than naming it literally, so that bumping rust-version cannot leave the job measuring a toolchain the tree no longer declares. Asserted by changing the declared value and observing the job follow it, without editing the workflow.
criteria_by: creator
schema: 2
version: 3
---

Found while implementing TASK-d81a05ef8e8d, and deliberately left out of it:
that criterion is about the tightness of the declared MSRV, and this is the
job next door.

The msrv job names 1.95 in three places -- the job name, the rustup install,
and the cargo +1.95 invocation -- while rust-version lives in two manifests.
Bump the manifests and the job keeps installing and building 1.95, which is no
longer the declared floor. It stays green while measuring a toolchain the tree
no longer names, which is the same rot TASK-daf25ab8a9b7 corrected when the
MSRV was an assertion nobody ran: the claim and the check drift apart, and the
green tick is what hides it.

The new msrv-tight job already derives the number from
crates/ank-cli/Cargo.toml with a sed and one subtraction, so the shape is
settled and the cost is a step and two env references. Doing both jobs the same
way is also what stops the file from teaching two different habits.

Worth deciding while claiming: the two manifests declare rust-version
independently, and nothing checks they agree. Deriving from ank-cli alone, as
msrv-tight does, is correct only while they match. Whether this task also makes
the disagreement visible -- a line in the job, or a check finding -- or whether
that is a third piece of work, belongs to whoever picks it up.

## Log
- 2026-08-04T18:32:03Z seanl@sean-laptop — Three decisions. The job name: a GitHub job name is evaluated before any step runs, so it cannot read a step output -- carrying the number there would need a separate floor job whose only work is a sed, adding a runner spin-up and a dependency edge that serialises both msrv jobs behind it. Dropping the number from the name instead: 'msrv / <os>'. It also makes the check name stable across a bump, which a derived name would not be. Checked that nothing depends on the current name -- main has no branch protection, so no required check references 'msrv 1.95 / ubuntu-latest'. Second, the open question the body left me, and I am the one who picked it up so I am answering it rather than deferring again: the two manifests declare rust-version independently and nothing checked they agree, which is exactly the hole that deriving from ank-cli alone opens. Added the comparison to the derivation step, exit 9 naming both files. It belongs in the msrv job rather than msrv-tight because that is the job about the declared number, and learning about a manifest disagreement from the tightness job would send the reader to the wrong place. Third, shell: bash on every derived step. This job runs the three-platform matrix and the default shell on windows-latest is pwsh, where $DECLARED is a PowerShell variable and an env var is $env:DECLARED -- so a bare run: would silently expand to nothing on Windows and install a toolchain named ''. The test job already sets shell: bash for the same reason.
