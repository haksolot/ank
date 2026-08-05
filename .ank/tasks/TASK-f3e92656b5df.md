---
id: TASK-f3e92656b5df
type: task
slug: every-long-flag-gains-a-declared-short-form
title: Every long flag gains a declared short form
created: 2026-08-05T04:06:02Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/cli.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  The short-form table lives in the specification section 4 before the parser
  moves. The parser accepts -s open and -s=open wherever --status is legal, and
  refuses bundling (-st) with a self-correcting error naming the exact flags to
  type separately. ank help <verb> shows both forms. Behaviour is tested through
  the binary.
criteria_by: creator
proof:
  - type: commit
    ref: 81c0501
    criteria: 9ed5d10705f8
schema: 2
version: 5
---

Execution of ADR-962c25797569, grammar half. The parser is hand-rolled
precisely for character-level control (cli.rs module header), so the change is
in one place: FlagSpec grows a short letter, parse() learns single-dash, and
the COMMANDS table stays the single source help renders from. One letter per
long flag where a letter is available; collisions within a verb are resolved
in the specification table, not improvised in code.

## Log
- 2026-08-05T04:06:58Z seanl@sean-laptop — amended: -blocked_by ADR-962c25797569
- 2026-08-05T05:48:55Z seanl@sean-laptop — Short-form table written into specification section 4 before any parser change, ADR-962c25797569. The rule is mechanical rather than negotiated: the letter is the first letter of the long flag, without exception, and where several long flags share one, exactly one takes it and the others keep only their long form. Ten letters: -j -q -r globally, then -b -c -l -p -s -t -v. That is what leaves --scope, --title, --ttl, --reason, --constraint, --supersedes and --body long-only, and section 4 names each one with the letter that would have been its. A -s meaning --status under find and --scope under new would be a silent wrong answer, not a saving: ank find -s open would filter on a scope named open and return nothing. One table in cli.rs rather than a letter beside each declaration, since --scope and --criteria are each declared in three CommandSpecs. Two consequences handled: a short flag that names a real flag the verb does not take gets its own message rather than unknown flag, and a single-dash argument containing whitespace is refused as the positional it is, naming the -- escape, because ank log '-1 rebuilt' used to be a message and is now a flag. Tested through the binary, six tests, with the negative control run: the branch disabled, three of them fail.
- 2026-08-05T05:49:22Z seanl@sean-laptop — done, proof commit:81c0501
