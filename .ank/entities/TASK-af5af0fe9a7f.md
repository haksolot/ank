---
id: TASK-af5af0fe9a7f
type: task
slug: help-new-says-per-kind-what-is-required-and-refu
title: help new says per kind what is required and refused, and help new <kind> answers for that kind
created: 2026-09-27T16:43:29Z
author: claude-code/opus-5.5+plan
status: done
scope:
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/help_new.rs
  - crates/ank-cli/tests/cli.rs
  - docs/integrating.md
blocked_by: []
done_criteria: |
  Through the binary: ank help new states, for each of task, adr and spec, the flags that kind requires and the flags it refuses, and lists the exit-7 refusal of an adr with no --constraint. ank help new adr, ank help new task and ank help new spec each exit 0 and print only what applies to that kind; ank help new <anything else> and ank help <verb> <x> for a verb with no kinds still refuse, naming the form that works. The text and JSON forms of help agree. A test in crates/ank-cli/tests/help_new.rs checks every requirement and refusal the help states against what ank new actually does, through the binary, so the help cannot drift from the refusals.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/7c158b24d857@74e4ca6
    tree: scope/16559f232716
    criteria: 71526cb26700
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@74e4ca6
    tree: scope/16559f232716
    criteria: 71526cb26700
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 6
---

Issue #504. `ank help new` lists `--constraint` like any optional flag, and
`ank new adr` refuses without it at exit 7 ("--constraint is required: the
binding rule, in one sentence"). You learn the rule from the refusal. And
`ank help new adr` is refused ("'help' accepts 1"), so there is no per-kind help
to read first.

The help comes from the verb table in `crates/ank-contract/src/verbs.rs`, and
the golden `tests/golden-json/help.json` holds its JSON form. ADR-2b62b9a1fe67 already
says a reference table is generated from the table it describes, which is why
the criterion requires the test to confront each stated requirement with the
binary's actual refusal: a hand-written list of requirements would be a second
copy, and it would drift.

The task for issue #503 (`--amends`) adds an adr-only flag after this one and waits
on it, so leave the per-kind shape easy to extend.
