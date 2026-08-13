---
id: TASK-a7b8c9d0e1f2
type: task
slug: human-surface
title: check, review, accept, close and show
created: 2026-07-27T09:50:00Z
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
  - .ank/config.yml
  - .ank/tasks/TASK-aca0cb103980.md
blocked_by: [TASK-e5f6a7b8c9d0, TASK-f6a7b8c9d0e1]
done_criteria: |
  check covers every invariant and signal listed in the specification and
  exits with 8 on findings, accept produces the signed ratification commit,
  close requires --reason and revokes the active claim, review filters by
  live scopes. accept refuses to run outside the default branch, with code 7,
  with a message naming the current branch and the expected one and giving
  the command to switch; there is no bypass flag, which a test checks by
  asserting that the list of flags accept takes contains neither force nor an
  equivalent. An indeterminable default branch makes accept exit with code 9,
  distinct from 7. check is the only command that prunes completion refs, and
  it does so only for tasks appearing done or closed on the default branch; a
  task carrying a completion ref that the default branch has not caught up
  with is reported as a signal and its ref kept. All four behaviours are
  checked by invoking the binary in temporary repositories placed on the
  right branch, never the function alone.
criteria_by: creator
verify: [cargo-test, check-repo]
proof:
  - type: test
    ref: local/e83df3f77af2@fcd3934
    tree: scope/744109ee80be
    criteria: c7e402e70a47
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/aa5a06cc948b@fcd3934
    tree: scope/744109ee80be
    criteria: c7e402e70a47
    verifier: check-repo@5734e9cf9d3d
schema: 3
version: 10
---

check_repo (examples/) is the draft of check and must disappear in favour of the
real command.

Amended by ADR-bcf222a31525, which gives `accept` its branch precondition and
`check` the pruning of completion refs. The task was not claimed, so the criterion
is amended with no freeze to lift.

The two exit codes are deliberately distinguished. 7 says "you are not in the
right place", and the caller knows what to do. 9 says "I do not know where the
right place is", and it is the repository that needs repairing. Conflating them
would send an agent switching branches over a configuration problem.

The absence of a bypass is tested, not commented: a `--force` on `accept` would
become the default path within two weeks, and the only way the constraint
survives its own convenience is for a test to fail if somebody adds one.

`check` stays the only maintenance point of the coordination plane — `claim` and
`context` never prune. The signal on an old completion ref is not a corpus fault
but the image of a branch never merged; the answer is human, which is the general
rule of §11.

The scope widens before the claim, criterion untouched: five verbs need five
dispatch arms, the binary-level assertions live in `tests/cli.rs`, and retiring
`check_repo` in favour of the real command — which the note above requires —
means deleting the example and repointing the `check-repo` verifier in
`config.yml` at `ank check`. Leaving the manifest out would have made the
retirement unmentionable rather than undone.

**Three signals of §4 are not reachable with the format as it stands**, and this
is a finding about the specification rather than a shortfall of the
implementation. "Blockers created by the holder after claiming" and "burst
creation by a single identity" both require knowing *who* created an entity, and
no entity carries an author: the fields are `id`, `slug`, `title`, `created`,
`status`, `scope` and the type-specific ones. The identity is hashed into the
identifier at creation and is not recoverable from it. "Created well before the
commit that introduces the file" needs `git log`, which ADR-b8884edcebe3 forbids
as porcelain. The same missing field already made §5's proposal line
unimplementable in TASK-d4e5f6a7b8c9. Recorded as its own task rather than
silently dropped.

An accepted ADR with no ratification commit is reported as a signal, not a
fault. Ten of them predate `accept`, `allowed_signers` carries no key, and the
file itself documents the bootstrap exception. Treating it as a violation would
make `check` exit 8 on this corpus from the moment it exists, which would block
every `ank done` until the whole history were re-ratified. The empty key file is
reported once on its own, which is §8's "advisory, displayed rather than
hidden".
