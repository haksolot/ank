---
id: TASK-45d18f45de2c
type: task
slug: dispatch-to-verb-modules
title: Dispatch routes to the verb modules, and the stubs stop claiming it already does
created: 2026-07-31T03:12:00Z
status: open
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/index.rs
blocked_by: [TASK-c3d4e5f6a7b8]
done_criteria: |
  cli.rs::dispatch routes to the verb module of every command whose module
  exposes an entry point, passing it the repository, the config and the
  identity that startup already resolves; the commands whose module is still
  a stub keep answering not_implemented and keep naming their task. Invoking
  the binary with claim on a temporary repository takes the ref
  refs/ank/claims/<id> and moves the task to in_progress, which a test
  establishes by running the binary and then reading the ref with git — not
  by calling claim::run. The exit code of a refusal reaches the process:
  claiming a task held by another agent exits 4, and claiming one without a
  done_criteria exits 7. No comment anywhere in crates/ank-cli/src still
  states that dispatch routes to a module it does not reach.
criteria_by: creator
verify: [cargo-test]
schema: 1
version: 1
---

The comment at the head of `claim.rs`, `done.rs`, `context.rs` and `index.rs`,
and the one at line 28 of `main.rs`, all say that dispatch already routes to the
verb modules and that no verb task will have to touch `cli.rs`. That was false
when TASK-c3d4e5f6a7b8 came to rely on it: `dispatch` returns `not_implemented`
for every verb but `init`. The command table parses `claim` correctly, and the
arm that would call it does not exist.

The cost of the false comment is not the missing line of routing — it is that
four separate tasks were told they had a foundation they did not have, and each
would have discovered it alone. Correcting the comments is therefore part of the
criterion and not a courtesy: a stub that lies about its surroundings is worse
than a stub.

`claim.rs` is not in the scope: TASK-c3d4e5f6a7b8 corrected its own header while
it was there, and `claim::run` already carries the signature this task's routing
arm calls.

Split off rather than absorbed into TASK-c3d4e5f6a7b8, whose `done_criteria` sits
at module level deliberately — `done.rs` was out of its scope, and forcing that
task toward the binary would have meant editing a frozen criterion to widen it.
The rule is the reverse: a discovered subtask is a new task with a `blocked_by`.
