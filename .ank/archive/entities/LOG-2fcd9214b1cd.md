---
id: LOG-2fcd9214b1cd
type: log
title: "closed: Owner decision on 2026-08-19: every package-manager channel is dropped (brew, scoop, apt,"
created: 2026-08-19T16:17:53Z
author: claude-code/5
scope:
  - .github/workflows/release.yml
  - .github/workflows/publish-brew.yml
  - .github/workflows/publish-scoop.yml
  - .github/workflows/publish-apt.yml
  - .github/workflows/publish-winget.yml
about: TASK-2078ab116f63
seq: 5
schema: 3
version: 1
---

 winget). The four workflows this criterion measures will not exist on the next tag; the release-event fix already in release.yml goes with them. The surviving channels (npm, curl|sh, and a PowerShell one-liner to come) do not consume the release event.
