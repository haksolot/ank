# Ank — development guide for agents

## Context

Ank is a CLI (Rust, GPL-3.0) that makes tasks and architecture decisions
readable by agents, directly in the repository. The full specification is
`docs/ank-spec-v1.1.md` — it is the source of truth, read it before any design
decision. This repo dogfoods its own format: the development plan lives in
`.ank/`, and it is reached through the CLI, never by opening the files
(ADR-01b6dd05f0db). `ank show <id>` prints an entity whole, `ank find` lists,
`ank context` binds.

## Commands

- `cargo test` — full suite. Must be green before any commit.
- `ank check` — validates `.ank/` (parse, byte-for-byte round-trip, blocked_by
  references). Must be green after any edit to `.ank/`. Exit 8 means findings;
  signals exit 0.
- `cargo fmt --check` — formatting.

**Dogfooding on Windows: run `ank` from a copy outside `target/`.** `ank done`
runs `cargo test --workspace`, which has to relink `target/debug/ank.exe` — the
very process running the verifier — and Windows locks a running executable.
Cargo reports the locked link as exit 101, which `done` reports as code 5,
indistinguishable from a failing test. Copy the binary elsewhere and invoke that
copy. This is not an ank defect and is not fixable in ank; it bites only a
project dogfooding ank on itself.

## Working loop

1. `ank context` — it lists what is claimable and serves the constraints
   covering this perimeter in full. `ank find --status open` lists without a
   query, `ank show <id>` gives you a task whole, body included. The body is
   where the reasoning is; read it before you start.
2. `ank claim <id>` — takes the task and freezes its `done_criteria` by hash.
3. Work. The criterion is frozen: never edit it to unblock yourself. A
   discovered subtask is a new task with a `blocked_by` (`ank new task
   --blocked-by <id>`), not a weakened criterion. If the criterion itself is
   wrong, `ank release --reason "<why>"` and say so.
4. `ank log "<what you learned>"` when you discover something, not when you
   finish. It renews the claim; working is what keeps the lock.
5. `ank done` — it runs the verifiers declared in `.ank/config.yml` itself and
   writes the proof. Never edit `status:` by hand, and never report your own
   result: an agent that grades itself can simply be wrong.

**Do not push, wait for a green run, and copy its id into `done`.** No task in
this corpus declares a `verify:` list, so `done` does still require a `--proof` —
but the one to give it is a proof you already hold, `commit:<sha>`, not a number
you had to wait for. The external anchor is the pipeline's work: once the task
lands on the default branch, `check` reports `done with no test proof` and the
`attest` job in `ci.yml` records `test:<run-id>` on `refs/ank/proof/<id>` with
`ank attest --detached` — a ref, no commit, and a red run rather than a silent
skip when the proof does not reach the remote. `assertion` and `human-review` are
weak proofs and `check` says so; `commit` and `test` are not. The recipe, and the
contract it rests on, are in `docs/getting-started.md`.

**A criterion that talks about the binary is tested through the binary.** When
a `done_criteria` says "the binary does X", the test must invoke the binary —
not only the function meant to produce X. Two real defects slipped through
green unit tests: a lock whose release failed under concurrency, and a `--repo`
resolution that dispatch never reached because it rejected the verb first. In
both cases the code under test was right, and the real path was not. The same
rule applies to platforms: OS-dependent behaviour is not verified until it has
run on all three.

## Implementation constraints (summary of the ADRs — the ADRs are authoritative)

- English only, everywhere: prose, identifiers, comments, CLI output, entity
  bodies (ADR-d3a8dcf38817).
- The format is the specification: `ank-core` is the reference implementation,
  and the round-trip stays byte-identical on canonical form. Any format change
  goes through the specification first, then the goldens, then the code.
- The CLI exposes one surface (ADR-f61e2d2c75e8, carrying ADR-c656cbcc33a9
  forward): every verb is available to every caller, and it refuses on state,
  never on identity. What is frozen is not the dispatch table but the content of
  `skill/SKILL.md` — the loop (`context claim show log done`, with `new find
  release` off-loop) and the planning verbs (`new adr`, `amend`, `review`,
  `graph`, `check`) — it is loaded permanently, so growing what it teaches costs
  a superseding ADR and a human signature. `ank help` lists every verb, grouped
  by the moment a verb is used, within a group in the order of §4; a group never
  says who may use a verb.
- `.ank/` is reached only through the CLI, never by opening the files
  (ADR-01b6dd05f0db). A `PreToolUse` hook in `.claude/` refuses it.
- Immutability is verifiable, not defended: freezes are anchored by hash, and
  the CLI is not a gatekeeper.
- Claims live in git refs `refs/ank/claims/<id>`, one per task, never in the
  files. The ref is not deleted at `done`: it becomes a completion ref, and is
  pruned by `check` only once the task appears `done` or `closed` on the
  default branch.
- `accept` only runs on the default branch (`default_branch` in `config.yml`,
  falling back to `refs/remotes/origin/HEAD`), with no way around it.
- Ank never commits, except `accept`.
- No new dependency without necessity; a static binary is the goal; **the MSRV
  is 1.95**, declared in both manifests and pinned in `ci.yml`. It was measured
  by walking toolchains against the tree — 1.78 through 1.94 all fail — and
  `libsqlite3-sys` alone sets it. The alternative was measured too and rejected
  (TASK-973e9dc3f9ce): one major back the floor is 1.82, at the price of eleven
  crates including a wasm stack this binary has no target for. **The floor is a
  consequence of a dependency, not a target to hold.** It moves when the
  dependency moves, `ci.yml` is what turns red, and re-measuring means re-running
  the walk with `--ignore-rust-version` — never editing the number.
  `ci.yml` turns red in **both** directions: `msrv` builds on the declared
  toolchain, proving it is sufficient, and `msrv-tight` requires the minor below
  it to fail, proving it is not higher than the tree needs. The second is a
  negative test, so it attributes the failure rather than merely observing it —
  a positive control on `ank-core` first, then a required rustc diagnostic code.
  A build that unexpectedly succeeds names the number to lower; it never lowers
  it, because the walk is what decides and a human runs the walk.

## Style

- Self-correcting errors: always the exact command to run next, never generic
  help.
- Terse `git status`-style output; `--json` everywhere, strictly opt-in.
- No emojis in messages, documentation or comments.
