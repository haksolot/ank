---
id: ADR-534c7a3e6cf8
type: adr
slug: ank-is-apache-2-0-whole-declared-by-the-channels
title: Ank is Apache-2.0 whole, declared by the channels that still ship
created: 2026-08-31T04:05:44Z
author: claude-code/opus-5+corpus
status: accepted
scope:
  - LICENSE
  - README.md
  - CLAUDE.md
  - crates/**
  - npm/**
  - package.json
constraint: |
  Ank is Apache-2.0, whole. Every crate in the workspace, the binary as
  distributed, and the licence text at the root of the repository: no part of it is
  copyleft, and there is no format/tool split any more. Every channel that declares
  a licence declares that one -- the channels are the ones ADR-221aa5da440a fixes,
  which today means the npm packages and the release metadata -- and the
  documentation states it in one place, with no second answer left anywhere else to
  contradict it. A declaration that disagrees with this one is a defect, and grep is
  the check.
  
  Relicensing is prospective and says so: a release already made under GPL-3.0
  stays available under GPL-3.0 to whoever received it, and nothing in the tree
  claims to withdraw that.
supersedes: ADR-9f03438f5422
ratified: 11f332dddc55
verified:
  - by: claude-code/opus-5+ratify
    at: 2026-08-31T08:12:42Z
schema: 4
version: 2
---

The licence is ADR-9f03438f5422's and it does not move: Apache-2.0, whole,
prospective, and that paragraph is carried across word for word. What moves is
the list of channels the rule addresses, and the scope that followed from it.

## What went wrong, measured

ADR-9f03438f5422 requires a licence declaration from "the Homebrew formula, the
Scoop manifest, the winget locale". ADR-221aa5da440a, accepted two days later,
says: "No package-manager channel ships: no Homebrew tap, no Scoop bucket, no apt
repository, no winget manifest, no AUR package."

So one accepted decision demands a declaration from three artefacts another
accepted decision forbids anyone to create. On this tree, 2026-08-31, ank 0.7.0
(50f4b39):

- `ls -d Formula bucket packaging` fails on all three, and `git ls-files` matches
  zero tracked files under each;
- all three are declared scopes of ADR-9f03438f5422;
- `ank check` names them: `dead scope 'Formula/**'`, `dead scope 'bucket/**'`,
  `dead scope 'packaging/**'`;
- `npm/**` matches 8 tracked files, and `LICENSE`, `README.md`, `CLAUDE.md` and
  `package.json` all exist.

This is not a contradiction that ever printed a wrong answer, which is exactly
why it survived: a rule about a file nobody can create is invisible until
something walks the globs. `ank check` is that walk, and it has been saying so
one signal at a time.

## What the successor does

**The three channels leave the constraint, and the enumeration stops being a
list this document maintains.** It names the set by pointing at
ADR-221aa5da440a, which is the decision that owns which channels exist, and adds
what that means today. When a channel is added or removed it is a supersession of
*that* decision -- ADR-221aa5da440a says so in its own words -- and this rule
follows without a second edit. The two-channels-today clause is there because a
constraint an agent meets for the first time in `ank context` has to be readable
on its own.

**The three dead globs leave the scope**, so `ank check` reports no dead scope
against this document. Nothing else moves: `LICENSE`, `README.md`, `CLAUDE.md`,
`crates/**`, `npm/**` and `package.json` are the predecessor's remaining scopes,
carried as they are. In particular the perimeter is not widened -- `crates/**`
already charges 733 characters against a `crates/ank-cli/**` budget `ank check`
reports at 18762 against a limit of 4000, and the successor's constraint is
written to stay near that weight rather than above it.

**`NOTICE` is left out, deliberately**, though it cites this decision. It is a
declaration at the root, not a channel, and adding a path to a scope under cover
of a repair is the move that produced the dead globs in the first place. If it
belongs in the perimeter that is a decision of its own.

## What is not reconsidered

Everything ADR-9f03438f5422 argued: the format/tool split abandoned, the copyleft
line that moved outward within a day of being drawn, the paved road a
reimplementer already has, the proprietary fork that becomes permitted and is not
minimised, Apache-2.0 over MIT for the patent grant, and the accounting that
showed there was nothing to negotiate. None of it is touched here and none of it
is restated at length; `ank show ADR-9f03438f5422` carries it whole and stays
readable after this is signed.

## What stands between this and a signature

ADR-3b6ba766a42e: `accept` refuses a supersession while any tracked file outside
`.ank/` still cites what it retires. Twelve sites cite ADR-9f03438f5422 today, in
seven files: `NOTICE`, `crates/ank-cli/tests/skill.rs`, and the `Cargo.toml` of
each of `ank-cli`, `ank-contract`, `ank-daemon`, `ank-mcp` and `ank-tui`.

Every one must name this document, or be dropped, before a ratification is
possible. The sweep is not performed here: the task that produced this document
is scoped to `.ank/entities/**`, and those files belong to other perimeters.

**Nothing is accepted by writing this.** It lands proposed and binds nobody, and
until a human signs it on the default branch the corpus goes on holding a licence
rule that addresses three channels it also forbids.
