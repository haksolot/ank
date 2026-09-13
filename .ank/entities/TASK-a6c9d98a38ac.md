---
id: TASK-a6c9d98a38ac
type: task
slug: ank-log-method-records-a-load-and-ank-skills-rep
title: ank log --method records a load, and ank skills reports the rate per sibling
created: 2026-09-13T09:21:46Z
author: claude-code/fable-5.1+planning
status: done
scope:
  - crates/ank-cli/src/**
  - crates/ank-core/src/model.rs
  - crates/ank-contract/src/verbs.rs
  - docs/getting-started.md
blocked_by: [TASK-e0d72ec220a1]
done_criteria: |
  Through the binary: under a claim, ank log --method tdd writes a log entry about the claimed task whose records is method and whose title is tdd, renews the claim, and takes no message; without a claim it is refused at 6 like any write, and a name the binary does not carry exits 7. ank show presents these entries apart from the work trace as it presents edit entries, and the count beside the trace is the trace's. ank check accepts method in the records vocabulary with no finding. ank skills, run in a corpus, prints beneath the catalogue one line per sibling with three counts, designated, fired among those, fired on tasks designating none, and --json carries the same as integers; proved on a scratch corpus of three tasks where the counts are known. docs/getting-started.md shows the report with real output.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/b03c79897e8c@0f46184
    tree: scope/1a2161d3efe8
    criteria: 93a0c036ead7
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@0f46184
    tree: scope/1a2161d3efe8
    criteria: 93a0c036ead7
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 4
---

The entry proves the load by construction only if the instruction to write it
exists nowhere but in the sibling's body; the verb's help describes the flag
and does not tell anyone to use it. The title is the name and nothing else, so
the count keys on a value and never on a message somebody rewrote.

The report reads the corpus once, in a batch, the way every plane is read
(ADR-cc659b2b7bd5), and it pays for what it prints (ADR-f3d1): a verb run for
the catalogue outside a corpus prints the catalogue and no counts.
