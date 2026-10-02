---
id: TASK-ea86d1cc4af4
type: task
slug: the-adrs-that-amend-another-are-found-through-th
title: The ADRs that amend another are found through the index, and show stops parsing every accepted ADR
created: 2026-09-27T20:46:02Z
author: claude-code/opus-5.5+plan
status: in_progress
scope:
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/archive.rs
  - crates/ank-cli/tests/amends_cost.rs
blocked_by: []
done_criteria: |
  On this repository's own corpus, ank show on an ADR and ank archive --dry-run read, per the measurement TASK-fe548f3dd587 logged, no more entity files than they did before that task (3 and 1), while printing the same amended-by relation and the same cold list; the counts are measured again and recorded with ank log. A corpus built by an older index still answers correctly, the index being rebuilt rather than trusted. A test in crates/ank-cli/tests/amends_cost.rs asserts through the binary that show prints amended-by after an amendment is added, accepted, and after the index is deleted.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Follow-up of TASK-fe548f3dd587 (issue #503). Its log records the cost it chose to
pay: the index does not store `amends`, so finding what amends an ADR parses
every accepted ADR. `ank show` on an ADR went from 3 entity reads to 67 and
`archive --dry-run` from 1 to 66, on this corpus, with nothing yet amending
anything. The number grows with the corpus.

ADR-f3d1dea65d84 is the rule at stake: a verb pays for the answer it gives. The
index is a cache from which nothing is believed over the files, so adding the
relation to it and bumping its own version costs a rebuild and nothing else.
Measure the reads the way TASK-fe548f3dd587 did (read its log first) so the
before and after are comparable.
