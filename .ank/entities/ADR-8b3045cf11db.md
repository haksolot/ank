---
id: ADR-8b3045cf11db
type: adr
slug: the-skill-has-one-source-and-every-channel-is-se
title: The skill has one source, and every channel is served from this repository
created: 2026-08-19T16:19:52Z
author: claude-code/5
status: accepted
scope:
  - npm/**
  - skill/**
  - docs/**
  - .claude-plugin/**
  - package.json
  - .github/workflows/**
  - install.*
constraint: |
  Every distribution channel ank offers is carried by this repository. No satellite repository holds a skill, a plugin manifest, a marketplace catalogue or a package manifest. skill/SKILL.md is the single source: a channel either points at it by path, or receives a copy produced at release time by an assembly script and excluded from git. A second copy of the skill committed to the tree is a finding. A registry a release is pushed into is not a satellite: npm is an address, and what it holds is derived from a tag by the pipeline rather than kept in step by hand. What stays forbidden is the second repository somebody maintains.
supersedes: ADR-782a3556cf2d
ratified: 8579b23df9eb
schema: 3
version: 2
---

## Context

ADR-782a3556cf2d carried two things: the no-satellite rule inherited from
ADR-e3cb36646d77, and a licensing argument for the channels whose registry is
a git repository, written to buy exactly one thing, the AUR. The AUR was
abandoned when its registry froze publisher access, and ADR-221aa5da440a
drops every package-manager channel: the tap, the bucket, the apt tree and
the winget manifests all leave this repository. The scope entries and the
worked examples of the old wording now name directories that no longer exist,
so this supersession keeps the rule and sheds the dead illustration.

## What survives, unchanged

The rule and its test. A copy nobody derives drifts, nothing turns red, and
one day an agent is taught the opposite of a ratified decision. The test is
where the source of truth lives: a copy the release derives is bound to the
commit that produced it. npm remains the worked example: a registry that
receives a package this repository derives at release time, and holds no
authority over it.

## What is shed

The tap-and-bucket paragraph and the AUR licence. They decided whether brew,
scoop and Arch earned a satellite; ADR-221aa5da440a decides those channels do
not exist, which answers the question further upstream. Should a future
channel return, it re-enters through a supersession of that ADR, and this
rule then decides where its files may live.
