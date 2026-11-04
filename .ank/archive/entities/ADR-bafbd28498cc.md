---
id: ADR-bafbd28498cc
type: adr
slug: the-agent-bench-lives-in-bench-in-stdlib-python
title: The agent bench lives in bench/, in stdlib Python, outside CI, and every claim it makes carries a control
created: 2026-10-26T12:59:57Z
author: haksolot@vmi3223161
status: superseded
scope:
  - bench/**
constraint: |
  bench/ holds a harness in Python using only the standard library, outside the cargo workspace and outside every CI job; a published result names the model, harness, ank and fixture-seed versions, runs at least five repetitions per cell, and compares ank against both no record and the same decisions written as plain Markdown.
ratified: 385f1d354e38
anchor: 30f6868724f1e0432d2829181d55900ecfc8eb5d
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-26T13:03:46Z
schema: 4
version: 4
---

## Context

On 2026-10-26 a throwaway prototype (scratchpad, not kept) ran virgin agents
against generated fixture corpora and scored each run automatically from its
transcript and from a hidden behavioural probe of the code it left. It answered
a real question the reading of the code could not (see ADR-5eb0f65c75da), and
it is the shape a repeatable comparison across models and versions of ank
needs.

What it could not say is whether the gain belongs to ank or to having written
decisions down at all: its only control was "no bootstrap".

## Decision

- **Home.** `bench/` in this repository, so fixtures follow the format of the
  ank that generates them and a pull request touching `init` or the surface can
  be measured against the same scenarios.
- **Language.** Python 3, standard library only. The bench is not shipped and
  is not the product; it is not a workspace crate (ADR-24efa7543a16 is not touched).
- **Not in CI.** A run spends the quota of whoever launches it and needs their
  credentials. CI runs the harness's own unit tests only, which spend nothing.
- **Agent isolation.** A run uses a configuration directory holding nothing but
  credentials: no hook, no skill, no memory, no MCP server, so what is measured
  is what the repository teaches.
- **Controls.** Every scenario runs under three conditions: no record, the same
  decisions as plain Markdown (`docs/adr/*.md` plus a `CLAUDE.md` pointing at
  them), and ank.
- **Reproducibility.** Fixtures are generated from a seed; a result line names
  model id, harness version, ank version and seed; at least five repetitions
  per (scenario, model, condition) before a number is published.
- **Results** are JSONL committed under `bench/results/`, one line per run.

## Consequences

The first family is adoption, the one already prototyped. Comprehension, long
tasks and orchestration are families this decision admits but does not design;
each comes as its own task once its criterion can be written.
