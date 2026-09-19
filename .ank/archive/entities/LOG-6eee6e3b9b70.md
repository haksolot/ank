---
id: LOG-6eee6e3b9b70
type: log
title: Two refinements after the first green pass, both from asking what a narrow window actually costs.
created: 2026-08-28T20:34:57Z
author: claude-code/opus-5+one-row-composer
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/text.rs
about: TASK-d58efe37eb7b
seq: 2
schema: 4
version: 1
---

 (1) The field a row drops is the rightmost droppable one and not simply the last: the entities' row number stands to the left of the identifier, and stopping at the first kept field would have protected the least useful field on the row behind the most useful one. (2) The last field is elastic and every field in front of it is a column. A row is a grid because the rows under it line up with it, and the field on the end has nothing to line up with -- so it takes the room that is left and is clamped, where a fixed column that cannot be afforded is dropped whole. Without that the config pane at forty columns showed the key and nothing else, where it used to show the head of the value; with it the old output comes back and the key is kept whole instead of cut. The config key is the one kept field that is not an identifier, for the reason the pane's own prose already gave: a key nobody can type back at a shell is what the pane exists to avoid. The last field is also given min(its column, the room left) so the clamp never puts a ~ on the end of a run of padding.
