---
id: TASK-afc96307cb81
type: task
slug: check-reports-an-adr-scope-no-commit-ever-carrie
title: check reports an ADR scope no commit ever carried as awaited while an open task covers it
created: 2026-10-25T08:25:27Z
author: claude-code/plan
status: in_progress
scope:
  - crates/ank-cli/src/human.rs
  - docs/check.md
  - crates/ank-cli/tests/dead_scope_awaited.rs
  - crates/ank-cli/tests/main.rs
blocked_by: []
done_criteria: |
  Under ADR-91f486072d68, tested through the binary on a fixture repository, for a proposed ADR and for an accepted one: a scope src/new.rs that no commit ever carried, with an open task whose scope is src/new.rs, and again with one whose scope is src/**, gives ank check exit 0 and a scope-not-yet signal naming that task; with no task covering it, a dead-scope fault and exit 8; once the only covering task is done with the file still absent, a dead-scope fault naming the task and done, and once it is closed instead, one naming the task and closed; once the file is committed, no finding on that scope; a scope git records as deleted, with an open task covering it, is reported as ADR-3094538d831e says and not as awaited; check --json carries no finding kind that did not exist before; the git processes check spawns on the fixture, counted with GIT_TRACE, are the same with and without the covering task, and the counts are recorded with ank log; docs/check.md says when an ADR scope ahead of the code is a signal and when it is a fault; cargo test and ank check are green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 3
---

The judgement is in human.rs around the ahead_of_the_code / retired / governs branches (~2290). Reuse Kind::ScopeNotYet and Kind::DeadScope. dead_scope_ratified.rs has fixture helpers for an accepted entity.
