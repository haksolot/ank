---
id: TASK-e70d28a5fba8
type: task
slug: skill-md-teaches-planning-and-the-freeze-moves-t
title: SKILL.md teaches planning, and the freeze moves to the new content
created: 2026-08-05T04:05:45Z
author: seanl@sean-laptop
status: done
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - docs/ank-spec-v1.1.md
blocked_by: []
done_criteria: |
  The specification section 4 is rewritten first: the freeze and its content as
  ADR-e17e1bbd93ff defines them. SKILL.md then grows the planning section — ank
  new adr, ank amend, ank review, ank graph, ank check, ank find --status open,
  and accept described as human, signed, default branch only, never as a verb
  the skill invites an agent to run. The skill.rs tests move to the new ceiling
  (at most 140 lines, 1200 words) and the new frozen content, including the
  teaches-nothing-beyond exclusion list. metadata.revision is regenerated and
  matches the skill hash ank --version reports.
criteria_by: creator
proof:
  - type: test
    ref: "31295730272"
    criteria: ef2a2ab1897e
schema: 3
version: 6
---

Execution of ADR-e17e1bbd93ff, in the order the project imposes: specification
first, then the taught file, then the tests that freeze it. The exclusion test
(the_skill_teaches_nothing_beyond_the_loop) does not disappear — it moves:
accept, close and attest stay excluded as invitations, while review, graph,
check, amend and new adr become taught. The wording for accept must thread the
needle the ADR describes: an agent knows what accept is and where its own
authority ends.

## Notes carried in the log

Rewrote the accept description test after the falsification exposed it as brittle. It had asserted the exact string with its backticks, so it failed on the rewording too -- for the wrong reason, and with a message claiming the file never names accept when it plainly did. It now asserts the three things the description has to say: accept, signed, default branch. A test that cannot tell a rewrite from a removal is a test nobody trusts twice.

Landed at 105 lines and 803 words against a ceiling of 140 and 1200. Left short deliberately: the ceiling exists to notice drift, and filling it now would leave nothing to notice.

Also corrected the citations. Section 4 and section 9 still credited ADR-c656cbcc33a9 for the one-surface clause and the flat help listing, and that ADR is superseded -- ADR-e17e1bbd93ff carries both forward unchanged. Citing a superseded decision as the authority is how a reader ends up at the wrong constraint.

Two things worth their own tasks, neither in this perimeter. commit.gpgsign is global here, so the throwaway git repos the integration fixtures build sign their commits: eight tests died in Repo::new when the local gpg agent's pinentry timed out, none reaching an assertion. Signing a fixture commit proves nothing and makes the suite depend on a developer's agent being unlocked, which is why CI is green -- it has no key. And this claim lapsed while that was being sorted out, which is the TTL working as designed; re-claimed against an unchanged criterion, so the freeze hash matched.
