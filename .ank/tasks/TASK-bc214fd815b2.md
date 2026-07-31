---
id: TASK-bc214fd815b2
type: task
slug: ank-new-writes-a-task-that-needs-no-hand-finishi
title: ank new writes a task that needs no hand finishing
created: 2026-07-31T22:33:17Z
status: done
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  A task created by ank new is complete: it declares its verifiers and carries the reasoning that justifies it, without a subsequent edit to the file. ank new task accepts the verifiers and the body, writes both into the entity, and the round-trip stays byte-identical on canonical form. The seven verbs are unchanged -- these are flags on an existing verb, not a new one. A test invokes the binary, creates a task, and asserts the file it produced is claimable and carries both, because what is being tested is what lands on disk.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/824f1d89cdbb@28b028d
    tree: scope/75d327263249
    criteria: 24210622f396
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@28b028d
    tree: scope/75d327263249
    criteria: 24210622f396
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/3c579d49cf08@28b028d
    tree: scope/75d327263249
    criteria: 24210622f396
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 5
---

This file is the evidence. `ank new` created it with an empty body and no
`verify:`, and both had to be written in afterwards by opening the file --
which is the practice this task exists to make unnecessary.

Two consequences, and neither is cosmetic.

**A task with no `verify:` is a trap with a delay.** `done` has two modes: run
the declared verifiers, or accept a `--proof` and validate what it can. A task
declaring none takes the second path, so the agent that finishes it submits its
own proof. That is exactly the shape TASK-a99baa078994 ran into -- "with no
verifier declared, done would have written a proof anchoring nothing, which is
the one shape of proof worse than none". Every task `ank new` writes today is
born in that state.

**A task with no body is a criterion with no reason.** The whole argument of
this format is that an agent inherits the *why*, not just the *what*. A backlog
created through the CLI carries none of it, so either the reasoning is lost or
someone opens the file -- and the second is what has been happening.

`--blocked-by` already exists here and works at creation. Adding a blocker to a
task that already exists is a different act, on a plan that is not yours: it
belongs to the human surface, not to this task.

No new verb. `new` is one of the seven and stays one of the seven; these are
flags on it, which ADR-2f8a61c04b7d does not restrict.

## Log
- 2026-07-31T22:41:35Z seanl@sean-laptop — Two flags on an existing verb, so ADR-2f8a is untouched: --verify (repeatable) and --body. Verifier names are resolved against config.yml at creation, same doctrine as --blocked-by in the same function -- a name matching nothing would otherwise surface at done, far from its cause; the refusal is a 7 naming what is declared. --verify on an ADR is refused rather than dropped, because an ADR has no verify field and a flag silently ignored teaches the caller it worked. body_of emits the canonical shape directly, blank line after the frontmatter and a trailing newline, so creation cannot write a file the first rewrite would reformat. new now takes cfg, which meant threading it through dispatch.
- 2026-07-31T22:42:01Z seanl@sean-laptop — done, proof test:local/824f1d89cdbb@28b028d test:local/e3b0c44298fc@28b028d test:local/3c579d49cf08@28b028d
