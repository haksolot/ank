---
id: LOG-f6db7c101b01
type: log
title: Three places, and a fourth I had to add so the document does not contradict itself. Section 4's
created: 2026-08-13T07:19:18Z
author: claude-agent-b
scope:
  - docs/ank-spec-v1.1.md
about: TASK-ae77a9ee2964
seq: 0
schema: 3
version: 1
---

 dispatch block now shows the --detached form. Section 7 names external attestation as a third category beside durable state and ephemeral coordination, with the argument the ADR makes: a green pipeline is durable and authored by an actor with no branch, so it must survive and has no tree to travel in. The union, the absence of a TTL and the pruning predicate are stated there. Section 10 keeps its deferred row and says why the flag does not lift it -- --detached removes the obstacle a pipeline used to hit, which was having to produce a commit, and removing an obstacle is not calling the verb. The fourth: section 7 already argues that claim and completed share one address because two namespaces would let a stale claim and a completion coexist. Read against a separate proof namespace that reads as a contradiction, so the ref-lifecycle paragraph now says why it is not one -- claim and completed are mutually exclusive answers to who holds this task, an attestation answers what anchors this work, and a task legitimately carries both at once. Verified that the guard is live rather than assuming it: misspelling the verb in the dispatch block turns three tests in skill.rs red, because section_4_order parses that block and help is held to it. No source file changed. Suite green, ank check exits 0.
