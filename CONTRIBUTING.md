# Contributing to ank

The contribution model of this repository is ank itself. The development plan
lives in `.ank/`, work is taken with `ank claim` and finished with `ank done`,
and the rules that bind a change are ratified ADRs served by `ank context`. This
file points at them; it deliberately does not restate them, because a second,
looser copy of a rule is how the two drift apart.

If you have never run the tool, read [Getting started](docs/getting-started.md)
first. If you need the normative answer to anything below, it is in the
specification, which is the source of truth: ten documents in `.ank/`, listed by
`ank find --type spec` and printed whole by `ank show <id>`. Each one says in its
own body which sections of the old monolith it carries, so a rule that reads
`(§7)` is resolved by the document that claims §7.

## The three gates

`ci.yml` runs these on Linux, macOS and Windows, and a pull request is green
when all three are:

```
cargo fmt --check
cargo test --workspace
cargo run -q --bin ank -- check
```

The third is the same line as the `check-repo` verifier in `.ank/config.yml`, on
purpose: a CI that validated `.ank/` differently from `ank done` would let a
corpus pass one and fail the other. Exit 8 means findings, and findings are a
failure. Two further jobs build the workspace on the declared MSRV and prove
that the minor below it fails. See the MSRV section.

Run all three locally before opening a pull request. They are the same commands
in both places.

## Working the loop

```
ank context                   # what binds here, and what is free to take
ank claim <id>                # takes the task, freezes its criterion by hash
ank log "<what you learned>"  # renews the claim; working is what holds it
ank done                      # runs the verifiers itself and writes the proof
```

Three things about this loop are not conventions but properties of the tool.

**The criterion is frozen at claim.** Editing `done_criteria` to unblock
yourself unblocks nothing: the hash is held in the claim record, and `ank check`
reports the divergence (ADR-6b3f19e08a24). A subtask you discover is a new task
with a `blocked_by`, never a softened criterion. If the criterion is wrong, say
so with `ank release --reason "<why>"`.

**Never edit `status:` by hand.** `ank done` runs the declared verifiers and
writes the proof of what actually ran, hashed. Setting the field yourself
produces a task that claims to be finished with nothing behind the claim, and an
agent, or a human, that grades its own work can simply be wrong.

**`.ank/` is reached through the CLI, not by opening the files**
(ADR-01b6dd05f0db). `ank show <id>` gives an entity whole, `ank find` lists,
`ank context` binds. This constrains agents, not people: a human with an editor
keeps every power they had, and `ank check` remains what notices.

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

## Changing the format

The format is the specification, and `ank-core` is its reference
implementation. Every format change happens in this order, and it is ratified
(ADR-63b59c5c26f7):

1. **the specification**: the `spec` document that states the rule, which for a
   format change is *The data model*, one of the ten `ank find --type spec`
   lists. No field exists in the code without existing there first.
2. **the goldens**: `crates/ank-core/tests/golden/`. `valid/` must round-trip
   byte for byte once normalised, `invalid/` must be rejected with the expected
   error.
3. **the code**.

The round-trip is byte-identical on canonical form; valid but non-canonical
input is read correctly and normalised on first rewrite. CRLF is read, never
written, and one golden is in CRLF on purpose and must come back in LF.

## The MSRV is measured, never chosen

`rust-version` is declared in both `crates/ank-cli/Cargo.toml` and
`crates/ank-core/Cargo.toml`, and enforced by two CI jobs: `msrv` builds on the
declared toolchain, proving it is sufficient, and `msrv-tight` requires the
minor below it to fail, proving it is not higher than the tree needs.

**Never edit that number to make a build pass.** The floor is a consequence of a
dependency, not a target held on purpose; it was measured by walking toolchains
upward against the tree, and re-measuring means re-running that walk:

```
cargo +<toolchain> build --workspace --locked --ignore-rust-version
```

An unexpectedly successful build names a number to lower. It does not lower it.
If a job goes red here, read the diagnostic and open an issue or a task; the
walk is what decides, and a human runs the walk.

## English only

English is the only language of the project (ADR-d3a8dcf38817): prose,
identifiers, comments, CLI output, error messages, entity titles, bodies, slugs
and log entries. Non-English text is a finding, not a matter of taste. The one
exception is a string whose meaning is its literal value: an external proof
reference, a quoted third-party message, a fixture asserting a byte sequence.

## Style

- **Self-correcting errors.** Every refusal prints the exact command to run
  next, never generic help.
- **Terse output**, in the shape of `git status`. `--json` everywhere, strictly
  opt-in, and never colored.
- **No emojis** in messages, documentation or comments.
- **No new dependency without necessity.** A static binary is the goal.
- **A criterion that talks about the binary is tested through the binary.** When
  a `done_criteria` says "the binary does X", the test invokes the binary, not
  only the function meant to produce X. Two real defects shipped past green unit
  tests that way. The same rule applies to platforms: OS-dependent behaviour is
  not verified until it has run on all three.

## Reporting

A suspected vulnerability goes to [private advisory
reporting](https://github.com/haksolot/ank/security/advisories/new), never to a
public issue. [SECURITY.md](SECURITY.md) says what is in scope and what section
1 already answers.

Anything else is an issue. The forms ask for what the tool already produces: the
exact command, its exit code, `ank --version` and `git --version`. Exit codes
carry meaning and they are the fastest triage available.

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md).
