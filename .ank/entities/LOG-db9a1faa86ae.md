---
id: LOG-db9a1faa86ae
type: log
title: "Confirmed: an ADR scoped 'b*:**' (not a peer ref, since 'b*' fails is_peer_name) is listed by ank"
created: 2026-09-27T17:04:13Z
author: claude-code/opus-5.5+c666
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/tests/peer_scope.rs
about: TASK-c666eb306102
seq: 4
schema: 4
version: 1
---

 scope bb:x alongside '**' and 'bb:**': 3 ADRs. The match is string globbing over the qualified path, not peer resolution. Fix goes in context::in_perimeter: split the perimeter with repo::peer_ref; a peer perimeter meets only entries naming that same peer, matched glob-to-path under the peer; a local perimeter meets only local entries.
