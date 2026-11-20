---
id: ADR-8a84eee73a77
type: adr
slug: a-corpus-is-declared-to-the-watcher-where-it-is
title: "A corpus is declared to the watcher where it is obvious: by init, and by watch in an undeclared checkout"
created: 2026-11-20T14:30:59Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/cli.rs
constraint: |
  ank init declares the corpus it creates in the reader's watch.yml, keyed on its repository identity and never on a path alone, unless --no-watch is given, and a failure to declare is a line saying why and never a change of init's exit code; ank watch run inside an undeclared checkout declares that one checkout before it watches, and declares nothing from outside a checkout; a running watcher re-reads watch.yml when it changes, and a declaration that stops resolving is a line on stderr while the last good one stays in force; nothing walks a filesystem looking for a corpus.
amends: [ADR-24e21cb83793]
schema: 4
version: 1
---

## Context

Issue #806. Keeping a corpus warm takes three hand steps today: copy the
identity out of `ank status --json`, hand-edit `watch.yml` (`ank watch
--where`), restart the watcher, which reads `watch.yml` once in
`ank_daemon::run` and never again.

ADR-24e21cb83793 requires that the daemon watch only what is declared to it,
keyed on identity, and walk no filesystem. It does not say who writes the
declaration. `watch` refuses `--repo`/`--worktree` on the grounds that
addressing a corpus from the caller's side is the discovery that ADR refuses
(`crates/ank-contract/src/verbs.rs`). This decision narrows that reading: the
checkout you stand in when you run `init` or `watch` is not discovered, it is
named by the act, and what results is still a declaration in `watch.yml`.

## Decision

- `init` declares the corpus it creates, creating `watch.yml` with `schema: 1`
  when absent; a second worktree of the same identity joins the existing key.
  `--no-watch` declines it. Without a configuration directory, or with a
  malformed `watch.yml`, `init` succeeds as before and says why nothing was
  declared: a watcher's absence is never an error.
- `watch` inside a checkout that `watch.yml` does not list declares it, then
  watches. Outside any checkout with nothing declared, it keeps today's exit 9.
  `--repo`/`--worktree` stay refused: the declaration is of the checkout the
  verb runs in, not of one it is pointed at.
- The running watcher re-reads `watch.yml` when it changes. A declaration that
  stops resolving is one line on stderr, and the last good one stays in force,
  as a failed fetch already is.

## Consequences

`init` now writes outside the repository. Every test that runs `init` must
point the configuration directory at a temporary one, measured by running the
suite with an empty configuration directory and listing what it left behind.
