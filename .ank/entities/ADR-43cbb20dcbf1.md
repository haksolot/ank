---
id: ADR-43cbb20dcbf1
type: adr
slug: ank-hooks-installs-at-user-level-the-binary-is-t
title: ank hooks installs at user level, the binary is the handler, and a corpus sets guards an orchestrator may only tighten
created: 2026-10-25T08:12:00Z
author: claude-code/plan
status: accepted
scope:
  - crates/ank-cli/src/config.rs
  - docs/integrating.md
  - crates/ank-core/src/config.rs
  - crates/ank-contract/src/env.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/hooks.rs
constraint: |
  ank hooks install <agent> writes into that agent's user-level configuration the entries that call ank hooks run <agent> <event>, and nothing beside it: no script file is written, the binary is the handler. A second install leaves the file byte-identical, and ank hooks remove <agent> takes out exactly the entries install wrote, leaving every other byte as it found it. Claude Code is the only agent of the first version; another agent is added by an amendment of this decision. ank hooks run is silent and exits 0 outside a corpus, and an error of its own lets the tool call through rather than blocking the harness. At session start it serves the agent ank status and the contract pointer. Before a tool call it applies two guards that .ank/config.yml declares: guard.corpus, deny or off, default deny, refuses a file tool aimed under the corpus directory; guard.edit, off, warn or claim, default warn, lets an edit through with a message naming the path that no live claim of the identity covers under warn, and refuses it under claim. ANK_GUARD_CORPUS and ANK_GUARD_EDIT, set by whoever launches the agent, may tighten the level the corpus declares and never loosen it: a looser value is ignored, and run says so.
ratified: 7710e1fd20c1
anchor: dbbec9db7aa7268487e15d57344d8903fc267dc0
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-25T12:09:02Z
schema: 4
version: 5
---

## Context

The maintainer retired the committed PreToolUse hook that denied `.ank/` to
Claude Code (TASK-10b8a29fd853): committed into the repository, it imposed
itself on every contributor's harness. herdr and moshi show another shape: the
tool installs itself into the user's own harness configuration, idempotently,
and stays inert where it has nothing to say.

## Decisions taken in the interview

- **User level, like herdr.** `install` writes `~/.claude/settings.json`, not a
  file in the repository. A repository without `.ank/` sees nothing.
- **The binary runs it.** The entries call `ank hooks run claude <event>`;
  ADR-1ea3 forbids a second file beside the executable, and a script would be
  one more thing to keep in step with the binary.
- **Two guards, read from the corpus.** Blocking direct access to `.ank/`
  enforces ADR-3c8d by default. Gating edits on a claim is strong, so the
  corpus chooses: `warn` by default guides without blocking, `claim` is an
  explicit choice of the repository.
- **The orchestrator tightens, never loosens.** An orchestrator such as herdr
  sets `ANK_GUARD_EDIT=claim` for its workers while a human beside them stays
  at `warn`. The committed level is a floor: the corpus is the authority, and
  a launch environment does not overrule it.
- **Fail open.** A hook that crashes and blocks every tool call bricks the
  agent's session; `check` remains what notices a rule broken.
- **Claude Code first.** One adapter, with the `<agent>` argument already in
  the surface so the next one arrives by amendment.

## Out of scope

A shell command that reads `.ank/` (`cat .ank/...` through Bash) is not
guarded in the first version: parsing a shell line is a heuristic, and a
heuristic that blocks is worse than none.
