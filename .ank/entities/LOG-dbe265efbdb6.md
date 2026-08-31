---
id: LOG-dbe265efbdb6
type: log
title: "drift audit 2026-08-31, re-measured and holds. Structure identical to every reader: status, review,"
created: 2026-08-31T07:53:21Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/src/style.rs
  - crates/ank-contract/**
about: ADR-1f70ce2c3eac
seq: 0
schema: 4
version: 1
---

 context, show and check were each run into a pipe and again through a pseudo-terminal, and with the escape sequences stripped the two are byte-identical for all five. The paint is real and not absent: the pty run of status carried 18 SGR sequences, the pipe run 0, NO_COLOR=1 at the pty 0, and 'status --json' at the pty 0. Bundling and abbreviation hold as declared: 'ank find -st adr' exits 1 with "'-st' bundles short flags" and names the two flags to type as 'ank find -s <v> -t <v>'; 'ank status -jq' the same; 'ank stat' exits 1 with 'unknown command' and lists the 26 verbs. One letter per long flag across the whole table, no collisions: -b -c -j -l -p -q -r -s -t -u -v.
