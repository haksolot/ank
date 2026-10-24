---
id: LOG-11d588bbdf90
type: log
title: "Review on two axes. Criterion: refused worded as the narrow frame words it on every width (title"
created: 2026-10-14T15:15:51Z
author: agent-afcd
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/graph.rs
about: TASK-afcd4f0f56ad
seq: 4
schema: 4
version: 1
---

 and body), the wide frame's resize with no graph owes one read run by read_asked and a second resize owes none (also wide->narrow->wide after a refusal), three wordings asserted in view.rs over nowhere(); graph.rs literal [N2,N3,N0,N1] and the property test now walk every pair of nodes by transitive reach. Constraints: ADR-559e (a resize spawns only the read of a panel it puts on the frame, a read and never a write; no modifier, no new key), ADR-8bd7 via 559e (reaches the corpus only through ank graph --json, renews no claim), ADR-d3a8 English. Gates: nextest --workspace 1553/1553 passed, cargo fmt --check clean, cargo check -p ank-tui --tests --target x86_64-unknown-linux-gnu clean, ank check exit 0. Out of declared scope: crates/ank-tui/src/lib.rs, 6 lines in the session loop calling read_asked.
