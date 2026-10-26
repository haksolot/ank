---
id: ADR-5eb0f65c75da
type: adr
slug: init-bootstraps-an-agent-a-teaching-block-in-age
title: "init bootstraps an agent: a teaching block in AGENTS.md, imported by CLAUDE.md"
created: 2026-10-26T12:59:41Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-cli/src/init.rs
  - .ank/entities/SPEC-3bccb8aee5b7.md
constraint: |
  On the run that creates .ank/, and only then, ank init appends to AGENTS.md a block that names Ank as a CLI, says to run ank context <path> before editing a path, forbids reading .ank/ by hand, names claim, done and help, and links the install page; and appends the line @AGENTS.md to CLAUDE.md, creating it when absent and leaving it untouched when a line of it already imports AGENTS.md.
amends: [ADR-1f5a45d43f82]
ratified: 4f7975b4f307
anchor: 1e164d74fa5d718a062a142ec4e9a99d51e625a1
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-26T13:03:46Z
schema: 4
version: 2
---

## Context

`ank init` writes one line into `AGENTS.md`: "This repo uses Ank: tasks and
decisions live in `.ank/`." Measured on 2026-10-26 with virgin agents (no
skill, no hook, no memory, `claude -p`) on a 1028-entity fixture whose one
accepted ADR caps a discount at 30% in words a grep for the method name does
not find, asked to "add a percentage discount method":

- With that line only: 3 runs out of 8 respected the cap, all of them Opus,
  and every Opus first ran `cat` over `.ank/` (1.9 MB, truncated). Haiku 0/3,
  Sonnet 0/2. `AGENTS.md` was opened once in seven runs; Claude Code does not
  load it, and where it was read the line pointed at the files, not the CLI.
- With a seven-line block in `AGENTS.md` and a `CLAUDE.md` holding
  `@AGENTS.md`: 6 out of 6 across Haiku, Sonnet and Opus respected the cap,
  every one through `ank context`, `claim` and `done`, none opened `.ank/`.

The block reached the agent because `CLAUDE.md` imports it into the system
prompt; its content worked once it was read.

## Decision

On the run that creates `.ank/` (ADR-1f5a45d43f82 keeps that boundary), `init`
appends the teaching block to `AGENTS.md` instead of the single line, and
appends `@AGENTS.md` to `CLAUDE.md`. `CLAUDE.md` is created when absent, and
left untouched when one of its lines already imports `AGENTS.md`. A re-run on an
existing corpus touches neither file. The block summarises no constraint, since
a summary drifts: it teaches the route to them. It links the install page
rather than naming one channel, so it stays true to SPEC-3bcc's three.

## Consequences

`init`'s perimeter in SPEC-3bcc grows by one file, `CLAUDE.md`, and the
specification says so. `--json` reports it among `added`. Repositories already
initialised keep what they have; their maintainers write the block if they want
it. Other agents' instruction files (Cursor, Gemini) are not written: AGENTS.md
is read natively by most of them, and a new file is added by amending this
decision once a measurement shows it is needed.
