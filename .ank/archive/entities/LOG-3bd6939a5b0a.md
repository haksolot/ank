---
id: LOG-3bd6939a5b0a
type: log
title: "mise 2026.10.17 sha256 per asset, measured: SHASUMS256.txt of the release, the GitHub asset digest"
created: 2026-10-24T08:43:05Z
author: claude-code/opus-5.5+w44e6
scope:
  - .github/workflows/ci.yml
  - .config/nextest.toml
about: TASK-44e6e1b47479
seq: 3
schema: 4
version: 1
---

 field, and sha256sum of the downloaded linux-x64 and windows-x64.zip all agree (linux-x64 fe6dd17e..., macos-arm64 9e27709d..., windows-x64.zip 32bcfda5...). The windows zip holds mise/bin/mise.exe. Locally, in an empty MISE_DATA_DIR: mise install --locked github:nextest-rs/nextest installs only nextest (1.9s), and mise x -- cargo nextest --version then runs 0.9.148 without fetching mdBook or node (installs/ still lists one tool); mise x -- rustc --version gives 1.99.1. rustup 1.29.2 toolchain install with no argument installs the rust-toolchain.toml channel. Pushed aed0008f, PR #675 opened early for its CI run.
