---
id: TASK-e0d72ec220a1
type: task
slug: a-task-carries-method-new-and-amend-set-it-and-c
title: A task carries method, new and amend set it, and context names it after the claim
created: 2026-09-13T09:21:37Z
author: claude-code/fable-5.1+planning
status: done
scope:
  - crates/ank-core/src/**
  - crates/ank-core/tests/**
  - crates/ank-cli/src/**
  - crates/ank-contract/src/verbs.rs
  - docs/format.md
blocked_by: [TASK-38dabf191c1a, TASK-544ec9655570]
done_criteria: |
  Through the binary: ank new task --method diagnose writes method: diagnose in the task's canonical position and ank show prints it; --method with a name the binary does not carry exits 7 naming the ones it does; ank amend <id> --method tdd replaces the value and journals the edit, on a done task too. After ank claim, ank context prints one line METHOD <name> beneath the criterion, with the sentence that it is the skill to load before the first edit, and no line at all on a task carrying none; the line is charged before the log so an entry never yields to it. ank done on a task with a method and no log entry closes green with its verifiers, proving done never reads the field. SCHEMA_VERSION stays 4, a golden fixture carries the field and round-trips byte for byte, a fixture without it still parses, docs/format.md's table matches the serializer, and ank help --json carries --method on new and amend.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/860f779f7f98@a1f599a
    tree: scope/e30aa8638637
    criteria: 5d3682b46677
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@a1f599a
    tree: scope/e30aa8638637
    criteria: 5d3682b46677
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 5
---

The names the binary carries come from the embedding TASK-544ec9655570: the
sibling directory names, validated the way `--verify` validates against
config.yml, at write time, with the same exit code and a hint naming the
choices.

The context line sits in execution mode only, beneath DONE_CRITERIA and above
CONSTRAINTS, one line, never truncated: it costs what a short line costs and it
is the whole point of the field. Measure the line's chars against the budget in
the fit tests rather than assuming it is free.

No bump, on `via`'s terms (SPEC-e258796162c4): an older reader that meets
`method:` refuses the file by name, which is the honest refusal, and one that
never meets it concludes only that none was designated.
