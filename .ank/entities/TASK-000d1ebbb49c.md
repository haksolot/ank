---
id: TASK-000d1ebbb49c
type: task
slug: vale-pinned-by-mise-lints-docs-readme-md-and-ski
title: Vale, pinned by mise, lints docs/, README.md and skill/ from the suite, under a negation budget per page and a skill count held to skill/
created: 2026-10-26T18:40:28Z
author: haksolot@vmi3223161
status: in_progress
scope:
  - mise.toml
  - mise.lock
  - .vale.ini
  - .vale/**
  - crates/ank-cli/tests/prose.rs
  - docs/**
  - README.md
  - skill/**
  - crates/ank-cli/tests/main.rs
  - crates/ank-core/src/registry.rs
blocked_by: [TASK-b53c9c748d24, TASK-e577204143ae]
done_criteria: |
  mise.toml pins Vale and mise.lock carries it for linux-x64, macos-arm64, macos-x64 and windows-x64. .vale.ini applies a style under .vale/ank/ to docs/**/*.md, README.md and skill/**/SKILL.md with these rules at level error: Dash (U+2014 and U+2013), History (the phrases 'used to', 'previously', 'at the time', 'before .* joined', 'captured (before|with)'), EntityId ((ADR|TASK|SPEC|LOG)-[0-9a-f]{4,12}, scoped by an ini section to docs/install.md and docs/quickstart.md only); and NoLonger ('no longer') at level suggestion. crates/ank-cli/tests/prose.rs holds three tests: one runs 'mise x -- vale --output=JSON' over those paths through the binary's working tree and fails on any error-level alert, naming file, line and rule; one counts matches of \b(not a|never|rather than|is not|are not|instead of|not the)\b per page against a table in the test file with one row per page, failing when a page exceeds its row, when a page has no row, or when a row names no file; one finds every number word or digit standing before 'skill', 'skills' or 'SKILL.md' in those pages and fails unless it equals the count of skill/SKILL.md plus skill/*/SKILL.md, and every one before 'sibling' or 'siblings' unless it equals that count minus one. All three are green at close, which means: the 23 em dashes at f997ea88 are rewritten in context (format.md 13, check.md 3, skill/plan/SKILL.md 2 with its revision bumped as tests/skill.rs requires, ci.md, peers.md 1 each, the rest already gone); alternatives.md and multi-agent.md count seven skills and six siblings, with the token figure in alternatives.md re-measured over the seven files and the command that measured it written beside it; the history phrases in format.md:116, format.md:498 and ci-jobs.md:71 are rewritten; the budget table records each page's count as the tree stands when the test is written, with the two rewritten pages at or under 10 and 11. cargo fmt --check is clean and cargo test --workspace is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 3
---

The enforcement half of ADR-0dd192525166. The pages rewritten first
(install.md, quickstart.md) land before this so the lint's first run is green
on them and their budget rows start low; every other page enters the table at
its current count and is lowered by whoever rewrites it next.

Why from the suite: docs.yml is not a required check (ci-jobs.md), and
test-ank-cli already runs through `mise x`, so a tool mise pins is reachable
from `cargo test` with no new job and no hook. The test should skip with a
clear message when mise is absent, the way test-ank-cli's own verifier does,
and never pass silently.

The budget regex is the one the ADR's measurement used; keep it identical so
the table's starting rows are the numbers in the ADR. `no longer` stays a
suggestion: ratifying.md:139 and msrv.md:34 use it for a scenario, not for
history, and an error there would force a worse sentence.

The skill-count test reads number words (one to twelve) and digits. Expect
'Found 6' style lines to be gone with TASK-b53c9c748d24; if a page quotes a
third-party count legitimately, the fix is the page, not an allowlist.

Measure the baseline before touching a page, and `ank log` the full table.
