---
id: LOG-d040d8bbbe58
type: log
title: "mise pins measured with mise 2026.10.17: mise.toml declares github:nextest-rs/nextest 0.9.148 (="
created: 2026-10-24T07:50:57Z
author: claude-code/opus-5.5+wd725
scope:
  - mise.toml
  - mise.lock
  - mise.local.toml
  - rust-toolchain.toml
  - .gitignore
  - CONTRIBUTING.md
about: TASK-d7253853996c
seq: 3
schema: 4
version: 1
---

 NEXTEST_VERSION in ci.yml), github:rust-lang/mdBook 0.5.4 (= MDBOOK_VERSION in docs.yml) and node 22.23.4 (newest 22.x per 'mise ls-remote node 22'; release.yml asked for "22"). 'mise lock --platform linux-x64,macos-arm64,macos-x64,windows-x64' wrote 12 checksum entries (3 tools x 4 platforms). Cross-checked against upstream: nextest linux 223ba936..., darwin(universal, both macos) 6c23b7fb..., windows zip 7a931b4a... equal the release's .sha256 files; mdBook linux 3f28de05... equals docs.yml's hand-kept MDBOOK_SHA256, arm64 03e8a6d8..., x64 a47d7bf0..., windows 8a6b2421... equal GitHub's asset digests; node linux b8492284..., darwin-arm64 c9e1dbc7..., darwin-x64 efcc2b0d..., win-x64 cbc74b5e... equal nodejs.org SHASUMS256.txt. The registry's aqua backend for mdbook mapped macos-arm64 to the x86_64-apple-darwin archive, so mdBook uses the github backend, which picks aarch64-apple-darwin.
