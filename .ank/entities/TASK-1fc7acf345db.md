---
id: TASK-1fc7acf345db
type: task
slug: docs-yml-takes-mdbook-from-mise-instead-of-a-han
title: docs.yml takes mdBook from mise instead of a hand-pinned archive
created: 2026-10-24T07:36:47Z
author: haksolot@vmi3223161
status: open
scope:
  - .github/workflows/docs.yml
blocked_by: [TASK-d7253853996c]
done_criteria: |
  docs.yml installs mise as ci.yml does (pinned version, sha256 per OS, no third-party action) and gets mdBook from mise install --locked; MDBOOK_VERSION and MDBOOK_SHA256 are gone from the workflow; mise x -- mdbook build of docs/ produces a book whose file list equals the one the current workflow produces, compared and recorded with ank log; a pull request run of docs.yml is green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Applies ADR-9c69bd47ce23 once it is ratified; do not claim before.
