---
id: TASK-af4a6db95aab
type: task
slug: a-document-states-the-integration-contract-for-s
title: A document states the integration contract for someone outside this repository
created: 2026-08-17T05:15:07Z
author: claude-code/2.1.233+integration-contract
status: open
scope:
  - docs/integrating.md
  - README.md
blocked_by: [TASK-155e98c184ed]
done_criteria: |
  A document in docs/ states the contract for a reader who has never seen this tree: ank help --json as the entry point, the exit code table, the rule that a task's state is the file together with its claim ref, its proof ref and the log entities naming it, the warning that check writes and prunes refs so a poller must not call it, and the golden fixtures offered as the conformance suite. The README links it. Every command it shows is run and its output pasted from the run, not typed.
criteria_by: creator
schema: 3
version: 3
---

`docs/getting-started.md` already says the integration surface is two things, an
exit code and `--json`, and it says it well for a CI pipeline. What it does not
say is what a tool needs that is not a pipeline.

Three things in particular, all of which cost a reader real time to discover:

- **A task's state is not in its file.** It is the file together with
  `refs/ank/claims/<id>`, `refs/ank/proof/<id>` and the log entities whose
  `about` names it. A tool that reads `.ank/` alone under-reports, and it
  under-reports silently. `ank show --json` is already the verb that answers
  whole.
- **`ank check` writes.** It prunes claim and proof refs. Anything polling on a
  timer must not call it.
- **The goldens are for them too.** `crates/ank-core/tests/golden/` already
  says so in its own header, and nothing outside that header repeats it.
