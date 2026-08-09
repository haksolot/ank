---
id: ADR-e64dfaafd578
type: adr
slug: config-yml-is-written-through-the-cli-and-the-wr
title: config.yml is written through the CLI, and the writer touches nothing else
created: 2026-08-09T06:37:07Z
author: seanl@sean-laptop
status: proposed
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/cli.rs
  - docs/**
constraint: |
  .ank/config.yml is read and written through the ank CLI. Reading is
  `ank config <key>`; writing is `ank config <key> <value>` and
  `ank config --unset <key>`, with nested values addressed by dotted path
  (`verifiers.<name>.run`). The key set stays closed: a key the parser does not
  know is refused by name, never written.
  
  The writer changes no byte it was not asked to change. Comments, blank lines,
  key order and quoting style survive a write untouched, and every key other than
  the one named is byte-identical afterwards. The `run` of a verifier is never
  rewritten by a write that did not name it: that string is hashed into every
  proof that verifier has ever produced.
  
  `config` runs without a parsed configuration, as `init` and `help` do. The
  caller who most needs it is the one whose config.yml does not load.
  
  This constrains the agent, not the human: a human with an editor keeps every
  power they had, and the file stays reviewable text in the repository.
schema: 2
version: 1
---

## Context

ADR-01b6dd05f0db closed `.ank/` to agents: entities are read and written through
the CLI, never by opening the file. Its body enumerates the writing verbs -- new,
claim, log, done, release, attest, close, accept -- and says nothing about
`config.yml`, because `config.yml` is not an entity. The gap was not argued; it
was simply out of frame.

Ank's own error messages are where it shows. Six sites tell the caller to open
the file:

- `or add "default_branch: <name>" to .ank/config.yml` -- git.rs, status.rs,
  context.rs, human.rs;
- `add it under verifiers: in .ank/config.yml` -- done.rs, on a task declaring a
  verifier the configuration does not have;
- `declare it under verifiers: in .ank/config.yml` -- commands.rs, on `--verify`
  at `new` and at `amend`.

These are the only errors in the CLI that name a file to edit instead of a
command to run, and section 4 calls that shape out by name: a self-correcting
error carries the exact command, never generic help. They are not wrong today --
there is no command to name.

One writer exists: `init` writes the default file if it is absent, and never
touches it again. Everything after that is a hand edit.

## Decision

`config.yml` stops being a hand-edited file for the same reason entities did.
The tool knows things the file does not -- which keys exist, what a duration
means, that an unknown key is refused rather than ignored -- and a caller
guessing at YAML is a caller who finds out at the next command, in the form of
every command failing at once.

The shape is `git config`'s, and deliberately: a flat key and a value, with
dotted paths for what is nested. It is the shape the parser's own error messages
already use -- `verifiers.<name>.timeout` appears verbatim in config.rs today --
and it needs no second vocabulary.

## Why the byte guarantee is in the constraint and not left to taste

There is no serializer. `Config` derives `Deserialize` and nothing else, and the
only configuration text this program has ever produced is the literal `init`
writes. A writer built the obvious way -- parse, mutate, re-serialize -- would
drop every comment and blank line, and would reorder `verifiers`, `roles` and
`identities`, which are `BTreeMap`s and would come back alphabetical.

That is the visible damage. The invisible damage is worse. `definition_hash` is
`sha256(run.trim() + NUL + timeout_in_seconds)`, and it is stored in every proof
that verifier ever produced. Re-quoting or reflowing a `run` scalar changes the
hash, and `check` then reports "verifier X changed since the proof" on every
historical task in the corpus -- a corpus-wide false signal produced by a
formatting change nobody asked for. A useful corollary: `timeout: 600s` and
`timeout: 10m` hash identically, so a timeout's spelling is free where a `run`'s
is not.

So the guarantee is not a nicety about respecting the user's comments. It is
what keeps the coordination plane honest, and it belongs where it binds.

## Why `config` runs without a configuration

`startup` loads `config.yml` for every verb but `init` and `help`, so a file
that does not parse makes every command exit 1 -- including `check`, and
including the command that would fix it. The exemption already has its reasoning
written in the dispatcher, about `help`: the caller most in need of it is the
one whose environment is wrong. A configuration verb that a broken configuration
disables is a verb that works only when it is not needed.

## Rejected

**Extending `init`** with flags. `init` is a one-shot bootstrap with a perimeter
fixed by section 9, and it is idempotent by assertion: it will not redo anything,
including correcting a key. It cannot serve a read-and-write use case without
becoming a different verb under an old name.

**Extending `edit`** to the configuration. `edit` opens an entity and validates
frontmatter on the way back; `config.yml` has no id, no version, no frozen
fields and no CAS. It would be one verb doing two unrelated things.

**Leaving it to the human.** That is the status quo, and it is defensible for a
human -- who keeps the editor either way. It is not defensible for an agent,
which ADR-01b6dd05f0db already established has no route into `.ank/` but the
CLI. Today that route stops at the configuration, and the six hints above are
the CLI telling an agent to do what the CLI forbids it to do.
