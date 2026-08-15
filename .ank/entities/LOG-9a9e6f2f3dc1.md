---
id: LOG-9a9e6f2f3dc1
type: log
title: "PR #41 merged (8f44ce2). Four of the five clauses hold on main: bug.yml requires the exact command,"
created: 2026-08-07T16:02:15Z
author: seanl@sean-laptop
scope:
  - .github/ISSUE_TEMPLATE/**
  - .github/pull_request_template.md
about: TASK-90442c8f0ca2
schema: 3
version: 1
---

 the exit code as a dropdown, ank --version and git --version; spec-divergence.yml asks which section; config.yml routes vulnerabilities to the private advisory; pull_request_template.md lists the three gates and asks which task closes. The fifth clause cannot hold as written. community/profile now reports pull_request_template non-null, but issue_template stays null and always will: that field reflects only the legacy single-file .github/ISSUE_TEMPLATE.md, never a directory of forms. Measured against three repositories that use a directory -- cli/cli, rust-lang/rust, denoland/deno -- all three report null. Satisfying it would mean adding a redundant legacy template purely to move an API field, which the directory layout supersedes and which config.yml routing needs. The work is done; the measurement is wrong.
