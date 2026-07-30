# Ank — development guide for agents

## Context

Ank is a CLI (Rust, GPL-3.0) that makes tasks and architecture decisions
readable by agents, directly in the repository. The full specification is
`docs/ank-spec-v1.1.md` — it is the source of truth, read it before any design
decision. This repo dogfoods its own format: the development plan lives in
`.ank/`, maintained by hand for as long as the CLI cannot do it.

## Commands

- `cargo test` — full suite. Must be green before any commit.
- `cargo run --example check_repo` — validates `.ank/` (parse, byte-for-byte
  round-trip, blocked_by references). Must be green after any edit to `.ank/`.
- `cargo fmt --check` — formatting.

## Working loop

1. Pick a task in `.ank/tasks/`: `status: open`, every `blocked_by` in `done`.
   At equal priority, the one that unblocks the most tasks, then `created`
   ascending.
2. Read the ADRs whose `scope` covers the files you are about to touch: the
   `constraint` field is binding.
3. Move the task to `in_progress`, increment `version`, add a log line
   (`- <ISO-UTC> claude-code@<ctx> — message`).
4. Work. The `done_criteria` is frozen: never edit it to unblock yourself. A
   discovered subtask is a new task with a `blocked_by`, not a weakened
   criterion.
5. Finish: `verify` checkers green (declared in `.ank/config.yml`),
   `status: done`, a `proof` entry (type `commit` with the SHA, until
   `ank done` exists), a log line, `version` incremented.

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
- The agent surface is frozen at 7 verbs (`context claim log done new find
  release`). Any new functionality goes to the human side or into the format,
  never into the agent surface.
- Immutability is verifiable, not defended: freezes are anchored by hash, and
  the CLI is not a gatekeeper.
- Claims live in git refs `refs/ank/claims/<id>`, one per task, never in the
  files. The ref is not deleted at `done`: it becomes a completion ref, and is
  pruned by `check` only once the task appears `done` or `closed` on the
  default branch.
- `accept` only runs on the default branch (`default_branch` in `config.yml`,
  falling back to `refs/remotes/origin/HEAD`), with no way around it.
- Ank never commits, except `accept`.
- No new dependency without necessity; a static binary is the goal; the MSRV is
  loose but Cargo.lock pins for rustc 1.75 (liftable if needed — note it).

## Style

- Self-correcting errors: always the exact command to run next, never generic
  help.
- Terse `git status`-style output; `--json` everywhere, strictly opt-in.
- No emojis in messages, documentation or comments.
