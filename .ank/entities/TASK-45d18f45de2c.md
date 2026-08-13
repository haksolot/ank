---
id: TASK-45d18f45de2c
type: task
slug: dispatch-to-verb-modules
title: Dispatch routes to the verb modules, and the stubs stop claiming it already does
created: 2026-07-31T03:12:00Z
status: done
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
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
proof:
  - type: commit
    ref: 90909dc
schema: 3
version: 3
---

The comment at the head of `done.rs`, `context.rs`, `index.rs`, `commands.rs`,
`human.rs` and `verify.rs`, and the one at line 22 of `main.rs`, all say that
dispatch already routes to the verb modules and that no verb task will have to
touch `cli.rs`. That was false when TASK-c3d4e5f6a7b8 came to rely on it:
`dispatch` returns `not_implemented` for every verb but `init`. The command
table parses `claim` correctly, and the arm that would call it does not exist.

The cost of the false comment is not the missing line of routing — it is that
four separate tasks were told they had a foundation they did not have, and each
would have discovered it alone. Correcting the comments is therefore part of the
criterion and not a courtesy: a stub that lies about its surroundings is worse
than a stub.

Split off rather than absorbed into TASK-c3d4e5f6a7b8, whose `done_criteria` sits
at module level deliberately — `done.rs` was out of its scope, and forcing that
task toward the binary would have meant editing a frozen criterion to widen it.
The rule is the reverse: a discovered subtask is a new task with a `blocked_by`.

The scope was widened to `crates/ank-cli/src/**` before the claim, the criterion
untouched. Two things the first reading had wrong. The false comment is in six
modules and not four — `commands.rs`, `human.rs` and `verify.rs` carry it as
well, and the criterion says "no comment anywhere". And `claim.rs`, listed as out
of scope on the grounds that it had already corrected its own header, is the file
whose header the routing arm makes false in the other direction: it states that
dispatch does not reach it. The widening changes no applicable constraint, every
ADR bearing on this code being scoped `crates/ank-cli/**` or wider.

`crates/ank-cli/tests/**` enters the scope for the same reason: the criterion is
tested through the binary, `CARGO_BIN_EXE_ank` exists only for an integration
test, and the crate has no library target through which a unit test could reach
the process. Testing the exit code without spawning the process would be exactly
the shortcut the rule was written against.
