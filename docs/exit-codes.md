<!-- Generated from crates/ank-contract/src/exit.rs; do not edit.
     Regenerate: cargo run -q -p ank-contract --bin exit-codes > docs/exit-codes.md -->

# Exit codes

Every ank verb exits with one of these codes, and the code carries the meaning so a script can route without parsing output. They are stable. `ank help --json` publishes which verb returns which, under each verb's `refuses`.

A refusal writes `error[<code>]: <message>` on stderr, followed by the exact command to run next; under `--json` it leaves stdout empty.

| Code | Name | Meaning |
|---|---|---|
| 0 | `Ok` | the verb answered |
| 1 | `Generic` | generic error: a call the parser refuses, a file the tool cannot make sense of |
| 2 | `NotFound` | no such entity, or a prefix matching more than one |
| 3 | `Conflict` | version conflict: the entity moved under the caller, redo `context` |
| 4 | `Unavailable` | the task is unavailable: held by another agent, or finished on another branch; take something else |
| 5 | `Proof` | a proof is missing, malformed, or of a type this act does not accept |
| 6 | `Transition` | the act is illegal from the state the entity is in: a frozen field diverged, a transition the state machine does not allow, or a write without the claim it needs |
| 7 | `Prerequisite` | a prerequisite is missing: the task is blocked, it has no `done_criteria`, a mandatory flag was not given, `accept` ran off the default branch, or the caller already holds a live claim |
| 8 | `Findings` | `check` or `review` found a fault; a signal alone leaves the code at 0 |
| 9 | `Environment` | the environment, not the work: `sh` or `git` absent, git older than 2.34, `$EDITOR` unset, a default branch that cannot be determined, a detached proof that never reached the remote, a directory that refuses the lock |

Two of them are the ones an agentic loop must handle: **3** means somebody moved, read again, and **4** means take something else. **6 and 7 are two codes on purpose**: in 6 the state forbids what was asked; in 7 the thing asked for is legal and something it depends on is absent. **9 is not a failure of the work**, and a pipeline that collapses it into "the command failed" sends somebody to fix sound code.
