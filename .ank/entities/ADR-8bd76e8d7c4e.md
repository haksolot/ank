---
id: ADR-8bd76e8d7c4e
type: adr
slug: a-terminal-reader-is-the-cli-s-own-presentation
title: A terminal reader is the CLI's own presentation, and it lives here
created: 2026-08-24T22:00:38Z
author: claude-code/opus-5+planning
status: accepted
scope:
  - crates/ank-cli/**
  - docs/**
constraint: |
  What ADR-894defc26f3d withdrew stays withdrawn: no browser reader in this repository, nothing under a viewer/ directory, no HTML page, and no task that would produce one. The contract such a reader rests on -- ank help --json, the exit codes, the contract version, the corpus identity and the golden suites -- stays public, documented in docs/integrating.md and permissively licensed. One case changes: a terminal reader ships here, as the verb ank tui. It reaches the corpus only by running the CLI with --json and never by linking ank-core, never by reading .ank/ and never by touching refs/ank/* itself, so the refusals it shows are the refusals the CLI gave and there is no second dispatch path to keep in step. It writes nothing the person at the keyboard did not ask for in that session, it renews no claim on its own, and accept stays a signed human act it may drive and never perform unattended.
supersedes: ADR-894defc26f3d
ratified: 2939cc5c27e2
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-24T22:26:38Z
schema: 4
version: 3
---

ADR-894defc26f3d expelled the reader from this repository, and gave three
reasons. Two of them are still true and this decision keeps them. The third does
not reach a terminal reader, and that is the whole of what changes.

## The reasons, weighed one at a time

**"A reader is a tool, and the tool this repository builds is the CLI."** True,
and it is the argument *for* `ank tui` rather than against it. A terminal reader
is not a second tool; it is the CLI drawing what it already prints, for the one
audience the CLI serves worst. The thing ADR-894defc26f3d refused to host was a
product with its own audience, its own release cadence and its own dependency
tree. A verb has none of those.

**"What it needed did not exist, and now does."** Still true, still the reason a
reader may live outside. It is not a reason every reader must. The contract was
built so that an outside reader is supported rather than a fork; nothing in it
says the inside is now forbidden ground.

**"An attempt at it had to reimplement packed-ref lookup and packfile delta
chains in JavaScript, and then reconcile the result against the CLI to know it
was right."** This is the load-bearing one, and it is the one that does not
survive contact with a terminal reader. That cost was not paid because the reader
was in-tree. It was paid because the reader could not run the CLI -- a browser
page cannot spawn a process. A TUI can, and does: it asks `ank` and renders the
answer. There is nothing to reimplement, so there is nothing to reconcile.

That is why this supersedes rather than contradicts. The browser reader is still
outside, for exactly the reason ADR-894defc26f3d gave. The terminal reader was
never the case that reasoning was measured on.

## Why a verb and not a sibling binary

ADR-372b82af1ec7 chose a sibling binary for the protocol surface, and it named its
reason: the surface is for a client with no shell, and a verb would have cost a
supersession that bought nothing. Here the reason inverts. The audience is a
human at a terminal, and a human types `ank tui`. A separate executable is
invisible to precisely the people it exists for, and would have to be
distributed, documented and discovered as a third thing.

The price is known and accepted: `crates/ank-cli/tests/skill.rs` holds every
dispatched verb to the Commands block of the specification, so SPEC-93531977642f
gains a line and passes through a signed acceptance before `tui` may dispatch.
That is one signature for a decision that will outlive it.

`skill/SKILL.md` does not move, and this is not an oversight. The skill teaches
agents, and ADR-91b77f036884 anchors what it teaches by revision hash. A TUI is
for a human; an agent has `context`, `find` and `show`, which are better suited to
it than any screen. Adding `tui` to the skill would spend the anchor to teach
agents something they should not use.

## The one dispatch path, and what it forbids

Spawning the CLI is slower than linking `ank-core`, and that is the trade this
decision takes deliberately. Every refusal in this project is a refusal on state:
a claim already held, a criterion whose hash moved, a proof missing. A reader
that linked the core would have to reproduce each of them, and the first one it
got subtly wrong would be a TUI showing a task as claimable that `claim` then
refuses. Running the binary makes that class of bug unreachable rather than
unlikely.

It also settles the writes without a second rule. `ank tui` takes a claim by
running `ank claim`, so ADR-052accd6e3b2 names an intersecting claim exactly as it
would in a shell, and ADR-0bb7ea8991bc holds: a claim is renewed by working, and a
screen left open all night renews nothing.
