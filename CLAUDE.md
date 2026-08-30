# Ank, a development guide for agents

Ank is a CLI (Rust, Apache-2.0) that makes tasks and architecture decisions
readable by agents, directly in the repository. This repo dogfoods its own
format: the specification and the development plan are entities in `.ank/`,
reached through the CLI and never by opening the files.

**No constraint is summarized here, and that is deliberate.** A summary drifts;
the corpus does not. `ank context <path>` serves the decisions and rules that
bind a perimeter, in full, before you touch it; `ank find --type spec` reaches
the specification; `ank show <id>` prints any entity whole. How to work is
taught by the skills in `skill/`: the contract in `skill/SKILL.md`, and one
policy per activity beside it (ank-plan, ank-drift, ank-loop).

## Commands

- `cargo test`: full suite. Must be green before any commit.
- `ank check`: validates `.ank/`. Must be green after any edit to `.ank/`.
  Exit 8 means findings; signals exit 0.
- `cargo fmt --check`: formatting.

## Dogfooding on Windows

Run `ank` from a copy outside `target/`. `ank done` runs `cargo test
--workspace`, which has to relink `target/debug/ank.exe`, the very process
running the verifier, and Windows locks a running executable. Cargo reports the
locked link as exit 101, which `done` reports as code 5, indistinguishable from
a failing test. Copy the binary elsewhere and invoke that copy. Not an ank
defect; it bites only a project dogfooding ank on itself.

## Proof

No task in this corpus declares a `verify:` list, so `ank done` requires
`--proof`. Give it one you already hold, `commit:<sha>`; never wait for a CI
run id. Once the task lands on the default branch, the `attest` job records
`test:<run-id>` itself, on `refs/ank/proof/<id>`, and turns red rather than
skipping when the proof does not reach the remote. The recipe and the contract
it rests on are in `docs/getting-started.md`.

## Landing

`main` is protected and takes no direct push. Work reaches it through a pull
request: commit on your branch, `gh pr create --fill --base main --head
<branch>`, then `gh pr merge --merge`. Ratifying has its own recipe, in
`CONTRIBUTING.md`, because `ank accept` runs on the default branch and its
commit still has to arrive through a PR.

**A merge commit is the only allowed method, and that is load-bearing.** A
ratification is located by the subject of its commit, `ratify <id>`, walked with
`rev-list --full-history` and no path restriction. A merge preserves that commit
whole. A squash rewrites the subject to the pull request title and the anchor is
never found again, which `check` reports as a signal at exit 0: the corpus
quietly stops being verifiable while CI stays green. A rebase keeps the subject
and drops the signature, which is a fault. Both are disabled on the repository
and in its ruleset. Do not re-enable them.

## Testing

A criterion that talks about the binary is tested through the binary. When a
`done_criteria` says "the binary does X", the test must invoke the binary, not
only the function meant to produce X: twice, green unit tests covered code that
was right on a path the binary never reached. The same rule applies to
platforms: OS-dependent behaviour is not verified until it has run on all
three.

**A fact is measured, not read.** Establish what the code does by running it and
counting what comes back, never by reading it and concluding: the suite into an
empty `TMPDIR` and then list what it left behind, the binary through a
pseudo-terminal and then compare the frames, `GIT_TRACE` at an absolute path and
then count the processes. Prove a process cost by counting processes and never
by timing: a wall in milliseconds measures the runner, where creating a process
costs about 25ms on a Windows runner against 2-3ms on Linux and swings by half
between two runs of one image. Record the numbers with `ank log` while you hold
the claim. A task that closes with a proof and no measurement is green and
leaves nothing behind for whoever asks later what was actually checked.
