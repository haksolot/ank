---
id: ADR-221aa5da440a
type: adr
slug: distribution-is-three-commands-npm-curl-sh-and-p
title: "Distribution is three commands: npm, curl | sh, and PowerShell"
created: 2026-08-19T16:19:26Z
author: claude-code/5
status: accepted
scope:
  - install.*
  - npm/**
  - .github/workflows/**
  - docs/**
constraint: |
  ank installs by exactly three official routes: npm install -g @haksolot/ank on any OS, curl | sh from install.sh on Linux and macOS, and a PowerShell one-liner from install.ps1 on Windows. No package-manager channel ships: no Homebrew tap, no Scoop bucket, no apt repository, no winget manifest, no AUR package. A workflow, manifest, formula or directory whose purpose is to feed a package manager is a finding. A new channel is a supersession of this decision, never an addition beside it.
ratified: 3c2d4e664c18
schema: 3
version: 2
---

## Context

Five channels shipped and two more were attempted. Measured over the life of
each: the AUR froze publisher access after a security incident and the channel
was abandoned (LOG-58e67); the apt repository needed a GPG key, a Pages
deployment and a rebuild script, and was declared no longer of interest by the
owner (LOG-5e4fa); winget submits to microsoft/winget-pkgs, a registry with a
human moderation queue this project does not control, and its first submission
(PR 418653) sat unmerged; brew and scoop cost no third party, this repository
being its own tap and bucket, but each still carried a workflow, a committed
manifest, a derivation assertion and a per-release bump. None of the four ever
published without a human republishing the release (TASK-2078ab116f63): the
whole downstream edifice also forced a PAT into the release workflow
(ADR-768374fe6076) just to make the release event real.

## The decision

Three routes, each a command served from this repository and derived from a
published release, none owing anything to a registry with its own gatekeeper:

- npm first, because it is OS-agnostic and crosses the firewall the channel
  exists for; the binaries travel inside the packages.
- curl | sh for Linux and macOS, from install.sh at the repository root.
- A PowerShell one-liner for Windows, from install.ps1 at the repository root,
  the one route that did not exist when this was decided.

## Rejected

Keeping the tap and the bucket because they are self-hosted. True, and not the
point: the criterion is a small surface, not zero gatekeepers. Each retained
channel is a workflow to keep green, a manifest to bump and an assertion to
keep honest, and the two shell installers already cover every machine brew and
scoop covered.

Keeping winget alone. It is the one channel with an external moderation queue,
which makes it the worst survivor, not the best.
