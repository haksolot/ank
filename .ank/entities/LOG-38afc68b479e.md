---
id: LOG-38afc68b479e
type: log
title: What moved and what did not. The counters in the_budget_still_governs_the_page_context_serves did
created: 2026-08-24T04:46:34Z
author: claude-code/opus-5-context-budget
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/golden-json/context.json
about: TASK-ecf0f37f68c9
seq: 7
schema: 4
version: 1
---

 not move: +12 not shown and +8 more tasks still hold, because that corpus is genuinely over budget and those numbers were pinning correct behaviour rather than the defect. tests/golden-json/context.json returned to its original bytes, so the bless is a no-op and is out of the diff; the conformance walker is green again and context.proposed and context.specs are exercised as before. The prose-identifier signal is unchanged by me: it reports 33 for the corpus, and every identifier my log entries and the new task body name resolves, so I add none. Worked under the existing claim because without the fix my own criterion cannot be green.
