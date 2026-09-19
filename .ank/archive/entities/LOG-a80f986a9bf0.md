---
id: LOG-a80f986a9bf0
type: log
title: "The meaning table lands in ank-contract as src/meaning.rs: Subject (Status/Kind/Severity) x name ->"
created: 2026-08-26T02:59:13Z
author: claude-code/opus-5+status-table
scope:
  - crates/ank-contract/**
  - crates/ank-cli/src/style.rs
about: TASK-174588603cd2
seq: 1
schema: 4
version: 1
---

 Role, one const MEANINGS and four lookups. Three findings worth keeping.

(1) Subject has to be part of the key, not a comment on it. The families are open -- a kind is whatever the registry declares -- so a lookup that took a bare name would resolve a future collision by declaration order, silently. role_of(Subject, name) makes 'what kind of entity is an open' answer None instead of guessing.

(2) The two parsing rules that used to sit inside ank-cli's state_sgr are meaning's, not a surface's: 'expired' wins over the status it expired from, and everything after a colon is addressing (claimed:who@host, finished:sha on main). Left in style.rs they would have been reimplemented by ank-tui, which is precisely the second opinion ADR-1f70ce2c3eac forbids -- the table would have been shared and the reading of it would not.

(3) The kind rows all land on Role::Identifier and that is the point, not filler. SPEC-89070ce7f3b8 fixes an identifier's rendering for every kind at once because find and scope list them side by side; one row per kind means a surface that wanted ADR- to read differently from TASK- has to delete rows rather than add an unreviewed call site. It also made the rows live: Style::id now paints Role::Identifier, same 33m, same bytes.

style.rs keeps the eight codes and gains one total Role -> code match, so a role added in the contract stops the CLI compiling instead of reaching a reader unpainted. state_sgr is gone; status() and landed() are now the shared lookup composed with that match and nothing else, asserted as an identity rather than as agreement -- comparing them against a second table written in style.rs would pass while two tables existed.

Not moved, and deliberately: human.rs paints check's severities with style.red/style.yellow at the call site. The rows are in the table (fault -> Fault, signal -> Attention) and Style::role renders them, but human.rs is outside this task's scope, so the call sites still name colours. That is the one place a second reading of a severity remains, and it is a one-line change for whoever owns human.rs next.

blocked is absent from the table on purpose: SPEC-89070ce7f3b8 struck it because it is derived from blocked_by at read time and no entity is stored carrying it. A row would have invented a status and coloured [blocked], which the existing style.rs test forbids.

The no-ANSI guard walks the whole crate's src rather than meaning.rs alone, and derives its needles from char::from(27u8) instead of writing them out -- so a plain grep for an escape sequence over ank-contract finds nothing at all, test included, which is what makes the criterion's own phrasing checkable by hand. Verified it fires by injecting a leak into renews.rs.
