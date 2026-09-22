# The MSRV

The minimum supported Rust version is measured, never chosen. It is the oldest
toolchain the workspace builds on, found by walking toolchains upward against
the tree, and it moves when a dependency moves it.

## Where it is declared

`rust-version` is declared in every crate of the workspace, six manifests
carrying the same number:

    crates/ank-contract/Cargo.toml
    crates/ank-core/Cargo.toml
    crates/ank-cli/Cargo.toml
    crates/ank-mcp/Cargo.toml
    crates/ank-daemon/Cargo.toml
    crates/ank-tui/Cargo.toml

## What enforces it

Two CI jobs, both required on `main` ([CI jobs and required checks](ci-jobs.md)):
`msrv` builds on the declared toolchain, proving it is sufficient, and `msrv is
tight` requires the minor below it to fail, proving it is not higher than the
tree needs. Both read the number out of a manifest rather than carrying it, so
neither job is edited when the floor moves.

They read two of the six, `crates/ank-cli/Cargo.toml` and
`crates/ank-core/Cargo.toml`, and `msrv` fails when those two disagree. Nothing
compares the other four, so moving the floor means moving all six by hand, and
the two directions fail differently. A manifest left *above* the new floor is
caught, because `msrv` builds on the declared toolchain without
`--ignore-rust-version` and cargo refuses the package outright, with `rustc
<running> is not supported by the following package`. One left *below* it is
caught by nothing, and goes on declaring a floor the workspace no longer has.

## Moving it

**Never edit that number to make a build pass.** The floor is a consequence of a
dependency, not a target held on purpose, and re-measuring means re-running the
walk, one toolchain at a time from the current floor upward:

    cargo +<toolchain> build --workspace --locked --ignore-rust-version

The first toolchain that builds is the floor. Write it into all six manifests,
in one change, and let both jobs confirm it: `msrv` that it builds, `msrv is
tight` that the one below does not.

An unexpectedly successful build on an older toolchain names a number to lower.
It does not lower it. If a job goes red here, read the diagnostic and open an
issue or a task; the walk is what decides, and a human runs the walk.
