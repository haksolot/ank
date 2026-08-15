---
id: LOG-a1579e7eb1fd
type: log
title: claim now warns when the claiming identity already holds a live claim on another task, naming it
created: 2026-08-05T04:58:13Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/identity.rs
  - crates/ank-cli/src/claim.rs
  - docs/**
about: TASK-d79dc424c63d
seq: 0
schema: 3
version: 1
---

 and its expiry, plus one line pointing at ANK_AGENT. It warns and never refuses: parallel agents each with their own identity are the design, one claim at a time is a convention. live_claims_of takes now as a parameter for the same reason is_expired does — the drift tolerance is two minutes, so an integration test waiting for a lapse would wait two minutes; the lapsed case is a module test instead. The warning survives --quiet, since what it reports is not the confirmation that flag silences, and in --json it goes into a warnings array rather than polluting the object. Section 8 of the specification and getting-started both document it, and the guide is checked against what the binary actually prints rather than against a hand-copied string.
