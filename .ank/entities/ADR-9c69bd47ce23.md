---
id: ADR-9c69bd47ce23
type: adr
slug: tool-versions-come-from-mise-and-rust-toolchain
title: Tool versions come from mise and rust-toolchain.toml, in CI and in every verifier
created: 2026-10-24T07:36:19Z
author: haksolot@vmi3223161
status: proposed
scope:
  - .ank/config.yml
  - .github/workflows/ci.yml
  - .github/workflows/docs.yml
  - .github/workflows/release.yml
  - CONTRIBUTING.md
  - .gitignore
constraint: |
  Tool versions are declared in mise.toml and locked with checksums in mise.lock, and Rust is pinned to an exact version by rust-toolchain.toml; the test verifiers run cargo nextest run through mise x --, exit 9 before anything else when mise is absent, and declare mise.toml, mise.lock and rust-toolchain.toml among their inputs; CI installs mise from its published binary pinned by version and sha256 per runner OS, without a third-party action, then mise install --locked, and every other tool reaches a workflow through mise; tasks in mise.toml are generic and a machine's own recipes live in a git-ignored mise.local.toml; the suite runs under cargo nextest run, tests that must not run concurrently are serialised by a nextest test group and never by an in-process lock, and the count of tests listed under nextest equals the count listed under cargo test.
supersedes: ADR-8c12a38531e3
schema: 4
version: 2
---

## Context

The tools this repository needs are installed four different ways, and the
corpus records what that has cost.

- **Locally, nothing installs them.** Two tasks stopped on one evening because
  no Rust toolchain was reachable (LOG-640d57812fb2, LOG-a93726835014, both
  noting "mise lists no rust install"); a third stopped because cargo-nextest
  was absent (LOG-32abf5f0ecd2); `ank done` failed at exit 9 because `sh -c`
  does not read the login shell's PATH (LOG-c0e182d18aa2). The decision this
  one replaces made a missing runner an honest exit 9, which it is, and left
  the runner to each contributor to install.
- **In CI, each workflow pins its own way.** `ci.yml` pipes
  `get.nexte.st/<version>` into tar under `~/.cargo/bin`, which broke on every
  macos-26-arm64 20261021 runner because the directory was absent
  (TASK-4095e3143f18); `docs.yml` pins mdBook by version and a hand-copied
  sha256; `release.yml` uses `actions/setup-node`.
- **Rust floats with the runner image.** No `rust-toolchain.toml` exists: CI
  compiled with rustc 1.99.0 on one image and 1.99.1 on the next, and a
  maintainer's machine with 1.97.1, while all three called it "stable".

The maintainer decided on 2026-10-24 to standardise on mise.

## Decision

1. **`mise.toml` and `mise.lock` at the root are the single source of tool
   versions** for cargo-nextest, mdBook and node. The lockfile carries each
   tool's checksum per platform, which replaces the sha256 kept by hand in
   `docs.yml`. A tool a workflow or verifier needs is declared there or not
   used.
2. **Rust is pinned by `rust-toolchain.toml` to an exact version**, with the
   components the repository runs (rustfmt). rustup reads it natively, so mise
   does not manage Rust. The MSRV stays `rust-version` in the manifests and the
   msrv jobs keep naming their toolchain with `cargo +<version>`.
3. **Verifiers run through `mise x --`.** The test verifiers in
   `.ank/config.yml` run `mise x -- cargo nextest run ...`, so the runner they
   need is the one the lockfile pins, whatever the caller's PATH. Each exits 9
   before anything else when `mise` is absent, so a missing tool manager is
   reported as environment and never as a failing test. Each verifier's
   `inputs` names `mise.toml`, `mise.lock` and `rust-toolchain.toml`.
4. **CI installs mise from its published binary, pinned by version and by
   sha256 for each runner OS, without a third-party action**, then runs
   `mise install --locked`. Every other tool reaches a workflow through mise.
5. **Shared tasks are generic; a machine's recipes are local.** `mise.toml`
   carries tasks every contributor can run (the suite, `fmt` and `ank check`,
   refreshing the dogfooded `ank` copy by build then atomic rename). What
   depends on one person's keys or shell, such as a git config that turns
   signing off while keeping credentials, belongs in `mise.local.toml`, which
   git ignores.
6. **Carried forward unchanged**: the suite runs with `cargo nextest run`, not
   `cargo test`; tests that must not run concurrently are serialised by a
   nextest test group, never by an in-process lock; the count of tests listed
   under nextest equals the count listed under `cargo test`; the job names, and
   so the required checks, do not change.

## Consequences

- mise becomes a contributor requirement, in the place cargo-nextest held.
  CONTRIBUTING says how to install it.
- Changing a verifier's `run` and `inputs` changes its definition hash; earlier
  proofs name the old definitions and `check` reports them as it reports any
  verifier change.
- `release.yml` keeps taking the newest npm on purpose (its comment explains
  why); only node itself moves under mise.
- `install.yml` stays out: it tests the end user's install path, and a managed
  toolchain on the runner would hide what a user without one meets.
- Bumping Rust or a tool is a pull request that changes `rust-toolchain.toml`
  or `mise.toml` and `mise.lock`.

## Rejected alternatives

- **`jdx/mise-action`.** Simpler and cached, but a third-party action in the
  supply chain for something a pinned download already does.
- **mise managing Rust.** It delegates to rustup anyway, and
  `rust-toolchain.toml` is what cargo, rustup and every editor already read.
- **`rust-toolchain.toml` with `channel = "stable"`.** Fixes the components and
  leaves the version to drift with the image, which is the defect.
- **Verifiers as `mise run <task>`.** The command would live in `mise.toml`,
  outside the verifier definition its hash covers; `mise x --` keeps the
  command in `config.yml`.
