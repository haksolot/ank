---
id: LOG-6b9be60fc31a
type: log
title: "Run 40079123447 (aed0008f): windows-latest failed in the mise step, job 120828805953: sha256sum"
created: 2026-10-24T08:45:33Z
author: claude-code/opus-5.5+w44e6
scope:
  - .github/workflows/ci.yml
  - .config/nextest.toml
about: TASK-44e6e1b47479
seq: 4
schema: 4
version: 1
---

 given a path under RUNNER_TEMP (D:\a\_temp) prints its line prefixed with a backslash, so got was '\32bcfda5...' against the pinned '32bcfda5...' -- the digest itself matched. Fix: hash from stdin. ubuntu-latest and macos-latest passed the mise step and reached the suite under mise x. Same run's versions step on windows-latest printed 'rustc 1.99.1 (58cf8f9c3 2026-10-13)' and 'rustc is 1.99.1, as rust-toolchain.toml pins it'.
