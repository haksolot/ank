---
id: ADR-2b62b9a1fe67
type: adr
slug: an-output-the-documentation-shows-is-measured-by
title: An output the documentation shows is measured by the suite, and a reference table is generated from the table it describes
created: 2026-09-19T18:25:20Z
author: claude-code/opus-5+docs-audit
status: accepted
scope:
  - docs/**
  - README.md
  - crates/ank-cli/tests/**
constraint: |
  A block in docs/ or README.md presented as what ank prints is either regenerated from the binary or compared with it by a test in the workspace suite that fails when they differ. A reference table (exit codes, entity fields and their enums, config.yml keys, environment variables) is generated from the source table it describes, never typed. A hand-typed output block with no test behind it is a finding.
ratified: a46e92c44f15
verified:
  - by: haksolot@omarchy
    at: 2026-09-20T17:19:23Z
schema: 4
version: 2
---

## Evidence

The 2026-09-19 audit's largest class of defect was output pasted by hand and
never re-read:

- `ank skills` revisions: 5 of 6 wrong, in docs/agents.md and
  docs/getting-started.md both.
- `ank init` output from before the flat layout; version strings 0.1.3 and
  0.7.0 in a page that says every output was run against a fresh repository.
- The exit-code list differs between skill/SKILL.md (4 codes),
  docs/getting-started.md (no code 1) and docs/integrating.md; none of them
  matches `crates/ank-contract/src/exit.rs`.
- docs/format.md says schema 3 and two `records` values; the model is at 4
  with three.

Each was right when written. None had anything that would turn red.

## Why generation and tests both

A test catches a drifted example; it cannot keep a table complete, because a
new exit code or field adds a row nobody wrote. ADR-6fd6 already makes the
machine surface one table in ank-contract: the documentation reads that table
rather than copying it. Examples stay hand-written for readability, and the
suite replays them with the golden redactor the JSON fixtures already use.
