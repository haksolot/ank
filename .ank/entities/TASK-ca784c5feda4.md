---
id: TASK-ca784c5feda4
type: task
slug: the-help-surface-says-what-the-verb-does-done-s
title: "The help surface says what the verb does: done's hint, its verifier wording, and check's writes"
created: 2026-08-13T16:24:22Z
author: claude-code/2.1.229
status: in_progress
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  The refusal of done when no proof is given names commit:<sha>, a proof the caller already holds, rather than a run id it must wait for. ank help done says a verifier comes from the task's verify list, not from config.yml alone, so a reader cannot conclude the declared verifiers will run. ank help check says the verb prunes refs, since a verb named check reads as read-only and this one writes.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs: the text of the refusal, and the two help pages.
criteria_by: creator
schema: 3
version: 2
---

Three small wrongs on the surface agents read first, and each one measurably
misled a session.

**The hint steers toward the practice a task in this corpus was written to end.**
`done.rs` refuses with `--proof test:<ci-run-ref>`, hard-coded. TASK-2dff950e5d51
replaced that workflow: the pipeline attests the run itself and the agent closes
on `commit:<sha>`, a proof it already holds and one Ank actually validates
against git. The old rhythm cost one session two full CI waits per task, and it
is exactly what an agent reconstructs from habit when the tool suggests it.

**The verifier wording made an agent write the wrong thing into CLAUDE.md.**
`ank help done` says it refuses when there is no proof and no verifier declared
to produce one, which reads as though `config.yml`'s verifiers would run.
`check-repo` and `cargo-test` are declared there and no task in this corpus names
them in a `verify:` list, so `done` always demands `--proof`. An agent believed
the help text, wrote it into the project guide, and found out only by running the
command. Verifiers come from the task, and the sentence has to say so.

**`check` writes.** It printed `pruned refs/ank/claims/...` while an agent was
reading its output, having run it freely in loops on the assumption that a verb
called `check` reads. The behaviour is specified and correct -- `check` is the
only command that prunes -- and the help page is where a caller finds out before
scripting around it.

None of this changes behaviour. It is the self-correcting-error rule applied to
the places where the tool describes itself, and that rule is load-bearing here:
one session reused a refusal's own wording verbatim as the specification of a
feature, which only works because the wording is trusted to be exact.
