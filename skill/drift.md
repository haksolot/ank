---
name: ank-drift
description: Audit the decisions in .ank/ against the current code and report what no longer holds. Use when asked whether ADRs, specs, or tasks are still accurate, after a milestone, or when the corpus and the code seem to disagree.
metadata:
  revision: "36cf5808e95e"
---

# ank-drift

Decisions age. The corpus says what was true when each one was ratified; the
code says what is true now. This skill compares the two and reports. It never
rewrites.

The ank skill is the contract and applies here in full. This file adds the
audit policy only.

## Gather

    ank review              what is proposed, and which scopes have gone dead
    ank check               what is already known to be wrong
    ank find --type adr     the decisions; --type spec, the rules
    ank graph               an order that may no longer make sense
    ank log <id>            what executors recorded against the entity

Logs are the richest source: a discrepancy entry is a finding somebody already
made for you, recorded mid-task and read by nobody since.

Then read the code each scope covers. ank scope <path> names what claims to
govern a file; the file says whether the claim still holds.

## Judge

For each accepted decision, three questions:

- Does the constraint still describe the system?
- Does the assumption it rests on still hold?
- Does anything now contradict it: another entity, the code, a test?

Evidence or it is not a finding. Name the entity, quote the assumption, point
at the code, test or log that breaks it.

## Report

Findings ordered by impact, each in this shape:

    ADR-xxxx  assumed:    what the decision took as true
              measured:   what the repository shows now
              evidence:   file, test, or log entry
              recommend:  supersede, keep, or needs a decision

Never edit an accepted entity and never write the superseding one uninvited.
Superseding is planning: hand the findings to the human, and what they decide
enters through ank-plan.
