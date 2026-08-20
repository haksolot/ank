---
id: TASK-106dccc7f71c
type: task
slug: seven-verbs-declare-no-refusal-and-nothing-says
title: Seven verbs declare no refusal, and nothing says whether that is meant
created: 2026-08-19T23:21:26Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-contract/**
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  For each of context, find, status, review, graph, scope and check, the refusals the code performs are compared against what the verb declares, and the comparison is recorded. Every refusal a verb performs and does not declare is declared, with its exit code, and ank help <verb> prints it; a verb that genuinely refuses on nothing keeps an empty list and the reason is written where the table declares it. cargo test --workspace and ank check stay green.
criteria_by: creator
proof:
  - type: commit
    ref: 6f073f136ac56b3992ddbaf86594a909a5c1a509
    criteria: 721b0f93413f
    via: submitted
schema: 3
version: 4
---

Measured while closing TASK-e89613d66284, which emptied thirteen of the
fourteen element shapes no golden fixture exercised and could not reach the
fourteenth.

`help.verbs[].refuses` is empty for seven of the twenty-two verbs: `context`,
`find`, `status`, `review`, `graph`, `scope` and `check`. What a verb declares
comes from the table in `ank-contract`, so no corpus makes one of those arrays
carry a row, and the conformance test names the row as unexercised for as long
as the seven declare nothing.

The question is not the fixture, it is whether the seven really refuse on
nothing. Some plainly do not: `scope` and `graph` take a path, `check` takes an
optional one, and a path outside the repository is a state. Others may be
honest as they stand. §9 says a verb's page carries "the state conditions on
which it refuses, each with its exit code", and a verb that refuses on none
satisfies that vacuously, which is either true of these seven or a gap nobody
has looked at.

So this is a reading before it is a change: for each of the seven, what it
refuses today in the code against what it declares. Where they agree, the
declaration is right and the empty array is a fact about the verb; where they
do not, the missing refusals are the defect and the fixture follows for free.
