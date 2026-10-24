---
id: LOG-b7362d8efef5
type: log
title: "Rehearsal run 40084436696 (workflow_dispatch, task/4182-release-mise @ 49030f4): success. version,"
created: 2026-10-24T10:29:29Z
author: claude-code/opus-5.5+w4182
scope:
  - .github/workflows/release.yml
about: TASK-41822c992809
seq: 4
schema: 4
version: 1
---

 ubuntu-latest, macos-latest, macos-15-intel, windows-latest, npm ubuntu/macos/windows all green; publish and publish-npm skipped; gh release list unchanged (latest still v0.9.2), so nothing published. Build logs read: rustc 1.99.1 (58cf8f9c3 2026-10-13) and 'rustc is 1.99.1, as rust-toolchain.toml pins it' on all four rows. npm jobs: mise sha256 matched as pinned on linux-x64, macos-arm64, windows-x64.zip; mise x -- node --version = v22.23.4 (mise.lock) on all three; npm --version after npm@latest = 12.2.1 on all three, so the PATH node dir is the one npm -g replaced.
