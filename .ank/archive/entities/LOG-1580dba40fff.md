---
id: LOG-1580dba40fff
type: log
title: packaging/** joins the scope now that packaging/winget/ and packaging/deb/ exist, and the timing is
created: 2026-08-16T18:04:04Z
author: claude-code/opus-5
scope:
  - npm/**
  - skill/**
  - docs/**
  - .claude-plugin/**
  - package.json
  - .github/workflows/**
  - install.sh
  - Formula/**
  - bucket/**
  - packaging/**
about: ADR-782a3556cf2d
seq: 1
schema: 3
version: 1
---

 the whole point. An accepted ADR's scope is hashed into its ratification commit and frozen there, so a path added after acceptance is a path the rule never reaches. Ratified as it stood, this decision would have covered neither of the two directories its own Consequences section names.

This is the second half of the repair recorded above. The scope was originally filed naming paths that did not exist, check reported four dead scopes at once, and they were dropped; install.sh, Formula/** and bucket/** came back when those channels landed. packaging/** is the last of the four, and with it the scope names every path the constraint actually describes.

The AUR is what remains, and it is the one channel this decision exists to buy. Ratification waits for it: TASK-cf8e must not be claimed before this ADR is accepted, and this ADR should not be accepted before the scope covers the directory that task will write into.
