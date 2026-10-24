---
id: LOG-e4e8118bb0fc
type: log
title: Clauses and readings taken. (1) check reads refs/ank/ratify/* in one for-each-ref and one cat-file
created: 2026-10-20T12:26:53Z
author: claude-code/6e3f
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-6e3f8f0049e9
seq: 1
schema: 4
version: 1
---

 --batch, measured by GIT_TRACE; (2) a tag's signature is judged by git verify-tag with the generated allowed-signers file, verdicts cached in the index by tag object name, and classified by classify_signature so the four outcomes of section 8 (and retired) hold for a tag as for a commit; (3) the freeze reads the ref first and the ratify <id> commit walk only when no ref carries the anchor; (4) show, find and review say ref or commit; (5) second clone: verified ref + file proposed -> signal naming the file, and context and scope serve it as binding; (6) file anchor names a held object no ref points at -> signal naming git update-ref, and ank update restores it; (7) ref with no file -> orphan signal; (8) file superseded that no ref's supersedes names -> signal. Readings: 'verified' means anchor equal to the file's constraint+scope and a signature verdict that is Trusted, or not judged because signers: declares no key (the advisory regime of ADR-964be4d940b2); Unchecked or any fault does not bind. Clause 8 keeps the commit walk as the legacy fallback: a successor ratified by a ratify commit names its supersession there, so no signal. Clause 6: ank update is the release installer; the maintainer chose that update restores the missing ratify refs of the current corpus first (after checking the held object is a tag whose subject is ratify <id>), reports it, and then does its release work as before.
