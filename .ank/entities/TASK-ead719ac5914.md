---
id: TASK-ead719ac5914
type: task
slug: the-prose-that-calls-ank-help-a-flat-listing-cat
title: The prose that calls ank help a flat listing catches up with the binary
created: 2026-08-14T18:17:54Z
author: claude-code/03fd
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-03fd4b2c27bc]
done_criteria: |
  docs/ank-spec-v1.1.md no longer describes ank help as one flat listing with no headings and no grouping, and cites ADR-f61e2d2c75e8 where it cited the superseded clause. The summary of the help verb no longer says 'in one flat listing'. cargo test --workspace is green, which means the test asserting the listing and the per-verb page print one string has been carried across the change.
criteria_by: creator
proof:
  - type: commit
    ref: f372df550f867c3f38ead26658b9e9e14f0dd24e
    criteria: 6a98f7885f5d
    via: submitted
schema: 3
version: 3
---

TASK-03fd4b2c27bc grouped the listing and stopped inside its own perimeter, which is crates/ank-cli. Two pieces of prose outside it now describe a binary that no longer exists.

Section 9 of the specification says, in as many words, that ank help is one flat listing with no headings and no grouping, and closes with 'None of this is a grouping (ADR-c656cbcc33a9, ADR-e17e1bbd93ff, neither superseded). The listing stays flat.' ADR-f61e2d2c75e8 supersedes ADR-e17e1bbd93ff on exactly that clause, and the specification is the source of truth, so it is the document that has to move rather than the one to leave contradicting the ADR it is meant to carry.

The second is smaller and is the reason this task also holds crates/ank-cli. The summary of the help verb reads 'every verb in one flat listing, or one verb in full', and it is what ank help help prints. It was left alone deliberately: the criterion frozen on TASK-03fd4b2c27bc required ank help <verb> to stay byte-identical, and editing that string is the one way to make the criterion false while making the output true. The freeze is gone once that task is done, so the string is corrected here instead.

Both surfaces print one string, not two: the listing and the per-verb page share spec.summary, and a test walks both through the binary to keep it that way. Changing the summary changes both at once, which is the property working.
