---
id: LOG-79a7e6c55ef8
type: log
title: "drift audit, measured not read. The constraint enumerates the routes: 'Reading is ank show, ank"
created: 2026-08-31T03:10:52Z
author: claude-code/opus-5+drift
scope:
  - .ank/**
about: ADR-01b6dd05f0db
seq: 0
schema: 4
version: 1
---

 find and ank context; writing is ank new, claim, log, done, release, attest, close and accept.' Measured on ank 0.7.0 (50f4b39) in a throwaway corpus, hashing every file under .ank/ except index.db before and after each verb: ank read, ank amend --scope, ank edit --title and ank config context_budget each changed those bytes and each exited 0. None of the four is in the writing list. On the reading side the binary dispatches 26 verbs, and scope, graph, status, review, check and the read form of log all answer out of the corpus without appearing in the reading list. The rule this ADR states still holds -- the CLI is the route, the human keeps the editor -- and the two lists no longer name the routes.
