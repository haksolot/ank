# Contributing to ank

The contribution model of this repository is ank itself. The development plan
lives in `.ank/`, work is taken with `ank claim` and finished with `ank done`,
and the rules that bind a change are ratified ADRs served by `ank context`. This
file points at them; it deliberately does not restate them, because a second,
looser copy of a rule is how the two drift apart.

Everything a contributor needs beyond the two sections below is on the
documentation site, under **Maintaining**:

- [Project conventions](https://haksolot.github.io/ank/conventions.html): working
  the loop here, changing the format, documentation, testing, English only, style
- [CI jobs and required checks](https://haksolot.github.io/ank/ci-jobs.html): the
  six checks `main` requires, and what every workflow does
- [Ratifying and archiving](https://haksolot.github.io/ank/ratifying.html): the
  one command that ratifies, the recipe for `ank archive` on a protected `main`,
  and the older anchors the merge-commit rule still protects
- [Releasing](https://haksolot.github.io/ank/releasing.html),
  [the MSRV](https://haksolot.github.io/ank/msrv.html),
  [signing keys](https://haksolot.github.io/ank/signing.html) and
  [re-recording the demo](https://haksolot.github.io/ank/demo.html)

If you have never run the tool, start with [the
quickstart](https://haksolot.github.io/ank/quickstart.html). The normative answer
to anything is in [the specification](https://haksolot.github.io/ank/specification.html).

## The three gates

`ci.yml` runs these on Linux, macOS and Windows, and they are the three to run
locally before opening a pull request -- the same commands in both places:

```
cargo fmt --check
cargo nextest run --workspace
cargo run -q --bin ank -- check
```

Beside [rustup](https://rustup.rs), the one tool to install is
[mise](https://mise.jdx.dev). rustup reads `rust-toolchain.toml`, which pins the
exact Rust; mise reads `mise.toml`, which pins everything else the repository
runs -- [cargo-nextest](https://nexte.st), under which the suite runs, mdBook
and node -- each locked by checksum in `mise.lock`. Once mise is installed:

```
mise install --locked
```

`mise run test`, `mise run check` and `mise run ank:refresh` are the suite, the
first and third gates, and a rebuild of the `ank` on your PATH, replaced by a
rename so that a running copy is never written over.

What only your machine needs goes in `mise.local.toml`, which git ignores. For
example, when your signing key is passphrase-protected and no agent holds it,
the suite's signing tests need a git config that keeps your credentials and
turns signing off:

```toml
# mise.local.toml
[env]
GIT_CONFIG_GLOBAL = "{{config_root}}/target/nosign.gitconfig"
```

```
# target/nosign.gitconfig
[include]
    path = ~/.gitconfig
[commit]
    gpgsign = false
[tag]
    gpgsign = false
[user]
    signingkey =
```

The empty `signingkey` is needed: `ank accept` signs whenever one is set,
whatever `gpgsign` says.

The third is the same line as the `check-repo` verifier in `.ank/config.yml`, on
purpose: a CI that validated `.ank/` differently from `ank done` would let a
corpus pass one and fail the other. Exit 8 means findings, and findings are a
failure.

They are three of the six checks `main` requires; the other three, and the one
script worth running by hand before a version bump, are in [CI jobs and required
checks](https://haksolot.github.io/ank/ci-jobs.html).

## Working from a fork

Claims are git refs under `refs/ank/claims/*`. Pushing them to a shared remote
is level 1 in section 7 of the specification, and it is what makes a claim
visible to anyone else.

**A contributor without push access to `refs/ank/*` runs at level 0.** The claim
is real and it is entirely local: nobody upstream can see it, and nobody
upstream is prevented from claiming the same task. That is the design working as
specified, not a defect: level 0 is the default mode and needs no
configuration.

The consequence is procedural. From a fork, coordination happens **in the issue
or in the pull request**: say which task you are taking before you start, and
say it where a maintainer can read it. `ank claim` still does its job in your
clone, freezing the criterion, which is the half that protects your work.
It announces nothing.

## Reporting

A suspected vulnerability goes to [private advisory
reporting](https://github.com/haksolot/ank/security/advisories/new), never to a
public issue; [SECURITY.md](SECURITY.md) says what is in scope. Anything else is
an issue, and the forms ask for the exact command, its exit code, `ank
--version` and `git --version`. Participation is governed by the [Code of
Conduct](CODE_OF_CONDUCT.md).
