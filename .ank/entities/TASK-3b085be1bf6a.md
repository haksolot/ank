---
id: TASK-3b085be1bf6a
type: task
slug: the-msrv-job-hardcodes-the-number-the-manifests
title: The msrv job hardcodes the number the manifests declare
created: 2026-08-04T18:17:37Z
author: seanl@sean-laptop
status: done
scope:
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  The msrv job derives the toolchain it installs and builds with from rust-version in the manifests rather than naming it literally, so that bumping rust-version cannot leave the job measuring a toolchain the tree no longer declares. Asserted by changing the declared value and observing the job follow it, without editing the workflow.
criteria_by: creator
proof:
  - type: commit
    ref: 62cb9c6
    criteria: 5297297678eb
  - type: test
    ref: "30939683900"
    criteria: 5297297678eb
schema: 3
version: 6
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
