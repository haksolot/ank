---
id: TASK-91462ace35fd
type: task
slug: the-golden-suite-covers-schema-3-the-flat-layout
title: The golden suite covers schema 3, the flat layout and the log file
created: 2026-08-11T22:27:12Z
author: claude-code@sean-laptop
status: in_progress
scope:
  - crates/ank-core/tests/golden/**
  - crates/ank-core/tests/golden.rs
  - .gitattributes
  - .github/workflows/ci.yml
blocked_by: [TASK-7fcdd44933f0]
done_criteria: |
  crates/ank-core/tests/golden/valid/ holds the six existing fixtures unchanged
  plus a schema 3 task carrying verified and a typed author, a schema 3 ADR, and a
  log fixture. The CRLF fixture is still in CRLF and its exemption still holds in
  .gitattributes and in the line-endings step of ci.yml.
  
  crates/ank-core/tests/golden/invalid/ gains a case per new rejection, each failing
  with its own named error and not merely failing: an unknown entity kind, an
  actor value that does not match the convention where the convention is enforced,
  a verified entry missing by or at, and a log line the grammar does not accept.
  
  golden.rs asserts round-trip byte identity on the normalised input for every
  valid fixture, still counts exactly one CRLF fixture, and asserts the specific
  error for every invalid one.
  
  The suite fails against the current parser, which has not been changed yet. That
  failure is the deliverable.
criteria_by: creator
schema: 2
version: 3
---

Second step, after the specification and before any parser change. The suite is
written against the format as the specification now defines it, so it goes red on
the tree as it stands — that is the point, and a task that leaves it green has
tested the old format.

The `invalid/` half is where a permissive implementation is caught, and the
existing suite already sets the standard: nine cases, each failing with a distinct
named error rather than merely failing. Hold the new cases to that. A test
asserting only that parsing returned an error will pass for the wrong reason
forever.

The CRLF fixture is the one thing most likely to be broken in passing. Its content
is its line endings, it is exempt in `.gitattributes` and checked from both
directions in `ci.yml` — once that no CRLF survives anywhere else, once that the
fixture still has its carriage returns. Moving fixtures around without moving both
halves of that guard produces a test that passes while testing nothing.

Think about where the log fixture lives. The suite is a directory of entity files
today; a log is a second file keyed by the same id, so the layout of the suite
itself has to say how the two are paired. Whatever is chosen, `docs/format.md`
says a third party can port this suite in an afternoon, and that claim has to
survive the change.

Be careful about what is actually enforced at parse time. The specification makes
a malformed actor a `check` finding, not a parse error, so the invalid fixture for
it belongs only where the convention is genuinely enforced. Getting this wrong
turns a signal into a refusal and locks 96 existing files out of their own format.

## Log
- 2026-08-13T04:26:19Z claude-code@sean-laptop — Suite written and red against the current parser, which is the deliverable. It names four things ank-core does not have: Error::UnknownKind { kind }, a verified field on Task and Adr (Verified { by, at }), parse_log_file and append_log_file, and Error::MalformedLogLine { line }. Layout chosen: valid/ and invalid/ keep entity files exactly where they were, so the CRLF fixture never moves and its exemption in .gitattributes and the ci.yml line-endings step holds untouched; log fixtures live in valid/log/ and invalid/log/, paired to an entity by id alone, which mirrors .ank/entities and .ank/log. No invalid fixture for a malformed actor: ADR-3877 makes it a check finding and never a parse error, so the guard is a positive assertion that a pre-convention author parses. The body log stays tolerant and the file log is strict, because a file whose whole content is the log leaves a stray line nothing else to be.
