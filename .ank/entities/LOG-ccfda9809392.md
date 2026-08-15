---
id: LOG-ccfda9809392
type: log
title: "Forms written and YAML parsed clean. Note on the criterion: gh api"
created: 2026-08-06T18:40:53Z
author: seanl@sean-laptop
scope:
  - .github/ISSUE_TEMPLATE/**
  - .github/pull_request_template.md
about: TASK-90442c8f0ca2
schema: 3
version: 1
---

 repos/haksolot/ank/community/profile reads the default branch only -- issue_template and pull_request_template stay null until this branch lands on main, so that clause cannot be verified from the branch. Verify after merge, then done.
