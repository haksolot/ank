---
id: TASK-f6cea0cfcf58
type: task
slug: ci-runs-clippy-with-warnings-denied-on-all-three
title: CI runs clippy with warnings denied on all three platforms, and the workspace is clean under it
created: 2026-11-16T11:14:07Z
author: claude-clippy
status: open
scope:
  - .github/workflows/ci.yml
  - Cargo.toml
  - crates/ank-cli/Cargo.toml
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/hooks.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/update.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/tests/accept_tag.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/dead_scope_awaited.rs
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-cli/tests/push_refused_before_update.rs
  - crates/ank-cli/tests/ratify_ref.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/Cargo.toml
  - crates/ank-core/Cargo.toml
  - crates/ank-core/src/id.rs
  - crates/ank-core/src/model.rs
  - crates/ank-core/src/parse.rs
  - crates/ank-core/tests/golden.rs
  - crates/ank-daemon/Cargo.toml
  - crates/ank-mcp/Cargo.toml
  - crates/ank-mcp/src/lib.rs
  - crates/ank-tui/Cargo.toml
  - crates/ank-tui/src/view.rs
  - rust-toolchain.toml
blocked_by: []
done_criteria: |
  rust-toolchain.toml installs clippy with the pinned toolchain; the test job of ci.yml runs 'cargo clippy --workspace --all-targets --locked -- -D warnings' on ubuntu, macOS and Windows and that step is green on all three; every lint the workspace allows is named, with its reason, in [workspace.lints.clippy] and inherited by every crate through [lints] workspace = true; the full suite stays green.
criteria_by: creator
verify: [fmt-check, test-ank-core, test-ank-contract, test-ank-daemon, test-ank-mcp, test-ank-tui, test-ank-cli]
schema: 4
version: 1
---

Clippy was neither installed by rust-toolchain.toml nor run in CI, and the workspace carried about forty warnings and one deny-by-default error (`never_loop` in `hooks.rs::install`, a `loop` every path of which broke on its first pass).

Two findings were more than style: an orphaned doc comment in `claim.rs`, left by a pruning function that was removed, attached itself to `claim::run`'s rustdoc; and the Windows build of `tests/tui.rs` carried dead-code warnings for fixtures only the `cfg(unix)` session uses.

`too_many_arguments` and `type_complexity` are allowed workspace-wide rather than refactored here: eleven signatures, a separate change with its own review.
