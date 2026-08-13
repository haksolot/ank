---
id: TASK-2dff950e5d51
type: task
slug: a-ci-recipe-that-names-no-vendor-and-this-reposi
title: A CI recipe that names no vendor, and this repository adopts it
created: 2026-08-11T22:26:13Z
author: claude-code@sean-laptop
status: done
scope:
  - docs/getting-started.md
  - .github/workflows/ci.yml
  - CLAUDE.md
blocked_by: [TASK-6d404f17f56d]
done_criteria: |
  docs/getting-started.md carries a section on running ank in a pipeline that
  states the contract in terms of exit codes and --json alone -- 8 means findings,
  9 means the environment -- and then shows the same recipe three times: GitHub
  Actions, GitLab CI, and a bare shell. No vendor-specific output format is
  produced by the binary, and the section says why.
  
  .github/workflows/ci.yml runs ank check as it does today and, on a green run,
  attests the tasks finished on the branch with ank attest --detached, producing no
  commit. A run whose attestation fails is red rather than silently skipped.
  
  CLAUDE.md stops instructing an agent to carry a CI run id by hand and points at
  what the pipeline now does.
criteria_by: creator
proof:
  - type: commit
    ref: 8debfc8e063934e2c2d92472ff76caf0b9f2d42b
    criteria: f6678d30ca4a
schema: 3
version: 5
---

Blocked on `attest --detached`, because the recipe is the reason that verb exists
and documenting it first would document something that does not work.

Three vendors, one contract, and the order matters: state the contract before any
of the three, so a reader on a fourth vendor is not left translating from GitHub.
The contract is small — `ank check` exits 8 on findings and 9 on a broken
environment, `--json` is stable and opt-in — and that is the whole integration
surface. Everything else is the CI system's own syntax.

Do not add a `--format github` flag or emit `::error::` annotations. It was
considered and refused: annotations are one vendor's protocol, and putting them in
the binary re-introduces exactly the coupling this batch of work removes. A repo
that wants annotations pipes `--json` into whatever it likes, and that is the
point.

For this repository's own pipeline, the thing to get right is what happens when
attestation fails. Silently skipping it would recreate the situation this corpus
is already in — ten `done` tasks with no test proof, each one a run that went
green and was never written down.

Note the ordering constraint in the workflow: the attestation is only meaningful
after the tests pass, and it must not run on a matrix leg. One attestation per
run, after the matrix, or the corpus grows three identical proofs per task.
