---
id: TASK-4dab9aa4573d
type: task
slug: check-signals-a-ref-under-refs-ank-in-a-namespac
title: check signals a ref under refs/ank/ in a namespace no reader serves
created: 2026-09-14T06:40:13Z
author: haksolot@vmi3223161
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  On a corpus carrying a forged refs/ank/foo/TASK-x, check reports one signal, never a fault, that names the ref and the command deleting it on the remote, one ref by name and no wildcard; check --json reports it under the existing findings shape with no field retyped; a corpus whose refs are all in a served namespace reports nothing new. cargo test --workspace, cargo fmt --check and ank check stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 1
---

This repository carries 311 refs under `refs/ank/remote-check/`, 2.7 MB,
newest record 2026-08-26, and no source in this tree has ever written or
read that name (`git log --all -S remote-check` finds nothing). Every verb
moved them through its batch and skipped them by name; after
TASK-dd3ab6cb2dcc they are not moved, and nothing says they exist.

The rule is ADR-4b45f344344f: ank does not delete what it did not write, the reader
names the refs and the command, and a human runs it. The message is
modelled on the previous-layout signal (`human.rs:375-400`): a signal and
never a fault, because such a corpus answers every verb. The served
namespaces are `claims`, `proof` and `watch/<remote>/claims`; the deletion
is `git push origin :<ref>`, one per line, because a wildcard push over
`refs/ank/*` from a worktree force-reverts the attestations a CI wrote.
