---
id: LOG-b9a7d78c875f
type: log
title: drift audit, measured not read. The constraint says 'Every channel that declares a licence declares
created: 2026-08-31T03:11:06Z
author: claude-code/opus-5+drift
scope:
  - LICENSE
  - README.md
  - CLAUDE.md
  - crates/**
  - npm/**
  - Formula/**
  - bucket/**
  - packaging/**
  - package.json
about: ADR-9f03438f5422
seq: 0
schema: 4
version: 1
---

 that one -- the npm packages, the Homebrew formula, the Scoop manifest, the winget locale'. Measured on this tree at 50f4b39: ls -d Formula bucket packaging returns 'No such file or directory' for all three, which are also three of this ADR's nine declared scopes and three of the dead scopes ank check reports. ADR-221aa5da440a, accepted two days after this one, states 'No package-manager channel ships: no Homebrew tap, no Scoop bucket, no apt repository, no winget manifest, no AUR package', so this enumeration names three channels a later accepted decision abolished. The licence rule itself was re-measured and holds: no superseded identifier is cited by any tracked file outside .ank/, and ank check reports 0 faults.
