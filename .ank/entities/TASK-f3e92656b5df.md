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
  - type: test
    ref: "30979320605"
    criteria: 9ed5d10705f8
schema: 3
version: 6
---

Execution of ADR-962c25797569, grammar half. The parser is hand-rolled
precisely for character-level control (cli.rs module header), so the change is
in one place: FlagSpec grows a short letter, parse() learns single-dash, and
the COMMANDS table stays the single source help renders from. One letter per
long flag where a letter is available; collisions within a verb are resolved
in the specification table, not improvised in code.
