---
id: ADR-2fc9b88cc0bd
type: adr
slug: ank-herdr-orchestrates-from-a-herdr-core-termina
title: ank-herdr orchestrates from a herdr-core terminal, through hc
created: 2026-11-02T10:36:29Z
author: claude-code/opus-5.5+plan
status: accepted
scope:
  - skill/**
  - crates/ank-cli/tests/skill.rs
  - .claude-plugin/**
  - docs/**
constraint: |
  The sixth sibling, skill/herdr (ank-herdr), teaches one activity: orchestrating several ank agents from inside a herdr-core terminal. Its description triggers only there, and it stops when HC_TERMINAL_ID is unset. It teaches the ank layer alone and defers every command's syntax to herdr-core's own manual (hc --skill). A worker starts with the environment of the herdr-core server, not the orchestrator's, so its identity (ANK_AGENT), TMPDIR and PATH are set in the command that starts it. The orchestrator claims nothing and records no load: it fans out ready tasks whose scopes are disjoint, one worktree, branch and ANK_AGENT per worker, and lands the finished branches one at a time. ank-plan never designates it as a task's method.
supersedes: ADR-55449b7f4f8f
ratified: f3056240c1ab
anchor: bd006ae645154c5fe35d12bd05805d8f48173956
verified:
  - by: haksolot@vmi3223161
    at: 2026-11-02T12:41:07Z
schema: 4
version: 2
---

ADR-55449b7f4f8f made orchestration a sibling skill and named herdr, the
runtime this project measured on. The runtime has since changed under it: the
maintainer's sessions now run in herdr-core, whose CLI is `hc`. Three facts
make the old constraint wrong rather than merely dated, all measured on
2026-11-02 with hc 0.1.14:

- **Detection.** A herdr-core terminal exports `HC_TERMINAL_ID` and
  `HC_TERMINAL_SOCKET`; `HERDR_ENV` is unset. A skill that stops when
  `HERDR_ENV` is not 1 stops in every session where it is now needed.
- **Syntax.** `hc --skill` prints herdr-core's manual. `herdr --skill` prints
  the older multiplexer's, whose panes, workspaces and `agent start` do not
  exist in hc: there are only terminals, started with `hc run`, talked to with
  `hc send`, waited on with `hc wait`.
- **Environment.** A terminal started with `hc run` inherits the herdr-core
  server's environment, not the orchestrator's. Measured: PATH was
  `/usr/local/bin:/usr/bin:/bin:/usr/games`, with no cargo, and neither
  `TMPDIR` nor `ANK_AGENT` was set. "Export in the pane before the agent
  starts" has no pane to export into; the variables belong in the command that
  starts the worker, where `hc` runs a template's `command` as a shell string.

**What does not change.** Everything ADR-55449b7f4f8f decided about the ank
layer stands: one sibling per activity, the ank layer only, syntax deferred to
the runtime's own manual, the orchestrator claiming nothing and landing one
branch at a time, ank-plan never naming it as a method. The directory and the
skill name stay `skill/herdr` and `ank-herdr`: herdr-core is herdr's core, and
renaming would move every channel that carries the skill for no reader's gain.

**Why hc alone, and not both runtimes.** A skill teaches only what has been
run. The older multiplexer is no longer what this project runs on, and a skill
that branches on two runtimes pays for both in every session's description.
