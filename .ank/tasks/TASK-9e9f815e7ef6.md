---
id: TASK-9e9f815e7ef6
type: task
slug: the-specification-names-the-surface-as-eight
title: The specification names the surface as eight
created: 2026-08-01T01:58:13Z
status: open
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
blocked_by: [TASK-85557f7ac5e7]
done_criteria: |
  The specification describes the agent surface as eight verbs with show among them, and no longer tells an agent to read the entity file because cat is already its show. The listing that puts show on the human surface moves it to the agent side. The module header of human.rs stops describing itself as the half of the CLI that does not run in the agent loop, since show lives there and now does. No ADR is edited: ADR-3859eb46bdc3 is the record of the decision, and this only makes the prose agree with it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
schema: 1
version: 1
---

ADR-3859eb46bdc3 moved show onto the agent surface. The specification still says the opposite, in three places, and the specification is the source of truth -- prose that contradicts a ratified corpus is worse than prose that is merely out of date, because an agent reading it is reading an instruction.

Section 9 heads the surface 'seven verbs, frozen'. The paragraph under it tells an agent that wants an entity's full body to read the file, on the grounds that the format is the specification and cat is already the agents' show. That sentence is the one TASK-3109a736c255 exists to contradict, and it survives in the document the whole project defers to. The command listing further down puts show under the human surface.

None of it was in scope for TASK-85557f7ac5e7, whose perimeter is the ADR, cli.rs, tests/skill.rs and SKILL.md. Widening it would have been the move the working loop forbids, so this is the new task with the blocked_by instead.

The human.rs header rides along for the same reason: TASK-24b9456d8ec7 and TASK-195538e3e15c both touched that file with a criterion about check and accept, and neither could repair a sentence about the audience split. show's implementation staying in human.rs is correct -- the ADR governs the surface, not the file layout -- but the header claims the file is the half that does not run in the agent loop, and one of its functions now does.

Not a format change, so ADR-63b59c5c26f7's order does not apply: nothing here touches a field, a golden or the round trip.
