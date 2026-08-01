---
id: TASK-9e9f815e7ef6
type: task
slug: the-specification-names-the-surface-as-eight
title: The specification names the surface as eight
created: 2026-08-01T01:58:13Z
status: done
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/human.rs
blocked_by: [TASK-85557f7ac5e7]
done_criteria: |
  The specification describes the agent surface as eight verbs with show among them, and no longer tells an agent to read the entity file because cat is already its show. The listing that puts show on the human surface moves it to the agent side. The module header of human.rs stops describing itself as the half of the CLI that does not run in the agent loop, since show lives there and now does. No ADR is edited: ADR-3859eb46bdc3 is the record of the decision, and this only makes the prose agree with it.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/adee250fea56@7429cdd
    tree: scope/77c936b41586
    criteria: b5fd48261ee7
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@7429cdd
    tree: scope/77c936b41586
    criteria: b5fd48261ee7
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/01a34aa2be7f@7429cdd
    tree: scope/77c936b41586
    criteria: b5fd48261ee7
    verifier: check-repo@5734e9cf9d3d
schema: 1
version: 4
---

ADR-3859eb46bdc3 moved show onto the agent surface. The specification still says the opposite, in three places, and the specification is the source of truth -- prose that contradicts a ratified corpus is worse than prose that is merely out of date, because an agent reading it is reading an instruction.

Section 9 heads the surface 'seven verbs, frozen'. The paragraph under it tells an agent that wants an entity's full body to read the file, on the grounds that the format is the specification and cat is already the agents' show. That sentence is the one TASK-3109a736c255 exists to contradict, and it survives in the document the whole project defers to. The command listing further down puts show under the human surface.

None of it was in scope for TASK-85557f7ac5e7, whose perimeter is the ADR, cli.rs, tests/skill.rs and SKILL.md. Widening it would have been the move the working loop forbids, so this is the new task with the blocked_by instead.

The human.rs header rides along for the same reason: TASK-24b9456d8ec7 and TASK-195538e3e15c both touched that file with a criterion about check and accept, and neither could repair a sentence about the audience split. show's implementation staying in human.rs is correct -- the ADR governs the surface, not the file layout -- but the header claims the file is the half that does not run in the agent loop, and one of its functions now does.

Not a format change, so ADR-63b59c5c26f7's order does not apply: nothing here touches a field, a golden or the round trip.

## Log
- 2026-08-01T02:28:50Z seanl@sean-laptop — Section 4 now heads eight verbs with show in the loop line, and the paragraph that told an agent to read the file carries the argument instead: show sat on the human side as the only unbounded reader, section 5 answers size with a budget and a truncation notice, and what that never answered was the body -- context serves the criterion and the constraints, never the prose that justifies them. Kept as prose rather than deleted, because it is the argument any ninth verb has to beat. show removed from the human listing and moved into the command block after claim; the v1 scope line reads 8 verbs, show included, and its human list gains close, which was missing there all along.

One edit beyond the letter of the criterion, inside scope and worth naming: section 4 ended with 'Everything else (editing fields, reordering, deleting) goes through editing the file directly'. No subject. It sits immediately under the human surface listing, so in context it means a human -- and ADR-01b6dd05f0db explicitly leaves a human every power they had -- but written without the actor it reads as a general permission and hands back exactly what that ADR withdrew from an agent. Made explicit rather than left to context.

The occurrences left in .ank/ are historical anchors and stay: task bodies already written, and ADR-2f8a61c04b7d itself, whose whole content is the position that was superseded. Rewriting either would falsify the record, which is the exception ADR-85e6bbb195b8 names.

README.md:139 carries the same sentence -- reading .ank/ directly is still a first-class use, cat is an agent's show -- and is in no scope here. Filed as TASK-5325eab5fce2 rather than reached for. Its second half is still correct after ADR-01b6 and should survive the rewrite, which is why it is a task and not a deletion.
- 2026-08-01T02:29:13Z seanl@sean-laptop — done, proof test:local/adee250fea56@7429cdd test:local/e3b0c44298fc@7429cdd test:local/01a34aa2be7f@7429cdd
