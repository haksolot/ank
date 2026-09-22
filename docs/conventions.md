# Project conventions

The rules a change to this repository is held to beyond the three gates in
[CONTRIBUTING.md](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md).
The binding ones are ratified ADRs, and `ank context <path>` serves them in full
for the files you are about to touch; this page names where each one lives
rather than restating it.

## Working the loop on this repository

The development plan lives in `.ank/`, and it is worked with the same loop
[the quickstart](quickstart.md) teaches: `ank context`, `ank claim`, `ank log`,
`ank done`. The corpus is reached through the CLI and never by opening its files
(ADR-e45e1a29fe91): `ank show <id>` gives an entity whole, `ank find` lists,
`ank context` binds, and `scope`, `graph`, `status`, `review`, `check` and the
read form of `log` answer the rest. That ADR enumerates every route on each
side, because a short list gets read as the whole one. It constrains agents, not
people: a human with an editor keeps every power they had, and `ank check`
remains what notices.

A task here declares its verifiers when it is written, and a plain `ank new
task` already carries `cargo-test` and `fmt-check`, which `.ank/config.yml`
marks default ([Proof and verifiers](proof.md#the-default-list)).

## Changing the format

The format is the specification, and `ank-core` is its reference
implementation. Every format change happens in this order, and it is ratified
(ADR-63b59c5c26f7):

1. **the specification**: the `spec` document that states the rule, which for a
   format change is *The data model* ([the specification](specification.md)).
   No field exists in the code without existing there first.
2. **the goldens**: `crates/ank-core/tests/golden/`. `valid/` must round-trip
   byte for byte once normalised, `invalid/` must be rejected with the expected
   error.
3. **the code**.

The round-trip is byte-identical on canonical form; valid but non-canonical
input is read correctly and normalised on first rewrite. CRLF is read, never
written, and one golden is in CRLF on purpose and must come back in LF.

## Documentation

Documentation a person reads lives in `docs/`, changes by pull request, and is
published as this site (ADR-33970fcdb6e8). A block presented as what ank prints
is replayed against the binary by `crates/ank-cli/tests/doc_replay.rs`, and a
reference table is generated from the source table it describes
(ADR-2b62b9a1fe67): the exit codes, the entity fields and the `config.yml` keys
each carry the command that regenerates them in their first lines. A normative
rule is linked, never restated.

## Testing

**A criterion that talks about the binary is tested through the binary.** When a
`done_criteria` says "the binary does X", the test invokes the binary, not only
the function meant to produce X. Two real defects shipped past green unit tests
that way. The same rule applies to platforms: OS-dependent behaviour is not
verified until it has run on all three.

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
