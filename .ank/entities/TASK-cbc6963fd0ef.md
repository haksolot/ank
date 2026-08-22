---
id: TASK-cbc6963fd0ef
type: task
slug: a-task-cannot-account-for-its-versions-and-the-a
title: A task cannot account for its versions, and the accounting is silent about it
created: 2026-08-22T16:44:55Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  ank check reports a task whose version exceeds what its records account for, on the same terms it reports an adr today: a signal naming both numbers, never a fault, and not one finding on a corpus that predates the regime. What accounts for a transition is decided and written down before the code, since a claim and a release leave nothing durable behind as things stand. Driven through the binary: a task claimed, released, claimed again, amended and finished answers check with no finding from this rule, and the same task with its file rewritten by hand and its version bumped is reported with both counts. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 4
version: 1
---

The half of ADR-16813b3bcf37 that TASK-dfe5a1bb0857 could not deliver, and the
reason is a property of the corpus rather than a shortcut somebody took.

An ADR and a spec are written by exactly two things: the verbs that change
content, each of which leaves a machinery entry, and `accept`, whose two
possible writes each leave a field behind. The count closes exactly, so the
accounting runs there.

A task is written by `claim`, `release`, `done`, `close` and `attest` as well.
`claim` and `release` each write the file and leave nothing durable behind, so
a task claimed and released five times carries ten versions that no reader can
evidence afterwards. Measured on TASK-3c12e0ced2c0, the first entity in this
repository to carry a machinery entry: version 4, one entry covering 2 to 3, the
two remaining being the claim and the `done`. A rule counting those would fire
on its own first subject and on every task ever amended, which is the volume
section 11 names as what teaches a reader to stop reading `check`.

**What must not be built here is a second machinery entry per transition.**
TASK-3c12e0ced2c0 settled that and tested it: a transition has a record of its
own, and tracing it twice would put mechanical lines on every task in the
corpus. What is missing is that none of those records names a version. Whether
the answer is a version in the claim record, in the completion ref, or something
else is a decision, and it is worth making before the code.
