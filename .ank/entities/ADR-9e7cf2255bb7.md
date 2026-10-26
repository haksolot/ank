---
id: ADR-9e7cf2255bb7
type: adr
slug: the-agent-bench-lives-in-bench-in-stdlib-python
title: The agent bench lives in bench/, in stdlib Python; no CI job launches an agent, and every claim it makes carries a control
created: 2026-10-26T16:29:59Z
author: haksolot@vmi3223161
status: proposed
scope:
  - bench/**
  - .github/workflows/ci.yml
constraint: |
  bench/ holds a harness in Python using only the standard library, outside the cargo workspace; no CI job launches an agent, and CI runs only the harness's own tests; a published result names the model, harness, ank and fixture-seed versions, runs at least five repetitions per cell, and compares ank against both no record and the same decisions written as plain Markdown.
supersedes: ADR-bafbd28498cc
schema: 4
version: 1
---

## Why this supersedes ADR-bafbd28498cc

Its constraint said bench/ stays "outside every CI job", while its own Decision
and the task that built the harness (TASK-9ae9b0959c9d) put the harness's unit
tests in CI. The worker followed the criterion and the body; the constraint was
the line that was wrong. This decision keeps everything else word for word and
states the rule the body meant: no CI job launches an agent.


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
- **No CI job launches an agent.** A run spends the quota of whoever launches
  it and needs their credentials, so it happens only on a person's machine, by
  their hand. CI runs the harness's own unit tests, which launch no agent and
  spend nothing, so a pull request that breaks the harness is red before anyone
  spends quota on it.
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
