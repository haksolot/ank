---
id: LOG-c7ee79329ac2
type: log
title: "Measured on PR #668, run 39977333250 (commit 21b9bfe4), with the versions step printing command -v"
created: 2026-10-23T15:35:57Z
author: claude-code/opus-5.5+w4095
scope:
  - .github/workflows/ci.yml
about: TASK-4095e3143f18
seq: 2
schema: 4
version: 1
---

 rustc/cargo, CARGO_HOME/RUSTUP_HOME and ls -la ~/.cargo. macos-latest job 120495344709, image macos-26-arm64 20261021.0445.8 (the failing image): rustc 1.99.1 at /opt/homebrew/opt/rustup/bin/rustc, cargo at /opt/homebrew/opt/rustup/bin/cargo, CARGO_HOME and RUSTUP_HOME unset, ~/.cargo holds only registry/ (written by the cache restore step just before), no bin/. The toolchain moved to Homebrew's rustup on that image, so nothing creates ~/.cargo/bin, and tar -C on it fails. ~/.cargo/bin is still on PATH there: the step's 'was not on PATH' branch did not print. With mkdir -p added, the cargo-nextest step passed and printed 'cargo-nextest 0.9.148 (cd1d6d546 2026-10-07)'. Same run, for comparison: windows-2025-vs2026 20261021.278.1 cargo at /c/Users/runneradmin/.cargo/bin/cargo, ~/.cargo/bin present; ubuntu-26.04 20261012.163.1 cargo at /home/runner/.cargo/bin/cargo, ~/.cargo/bin present; both passed the step, no PATH line printed. Location on a passing macOS 20261013.0424.1 runner not yet measured (none of the old logs print it).
