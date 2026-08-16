---
id: LOG-fb9de148c745
type: log
title: "discrepancy: the criterion requires \"a superseding ADR carries the change and arrives proposed; the"
created: 2026-08-16T20:13:55Z
author: claude-code/opus-5
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
about: TASK-182a7f58cdd6
seq: 0
schema: 3
version: 1
---

 skill is not edited without it", and that clause rests on a premise this corpus contradicts.

What I assumed when writing it: that any edit to SKILL.md grows what it teaches, and ADR-5dd7b4a9c875 makes growth cost a superseding ADR and a signature.

What I measured: the freeze is narrower than that, and crates/ank-cli/tests/skill.rs says so in as many words. The doc comment on the_skill_states_the_execution_model_it_assumes reads "Not a superseding ADR, and the reason is measured rather than assumed. What ADR-e17e1bbd93ff freezes is which *verbs* the file teaches, which the_skill_teaches_nothing_beyond_what_is_frozen enforces, plus the ceiling below. This adds neither a verb nor a flag." TASK-e3f4b6295b23 added the entire execution-model paragraph on exactly those terms, and TASK-21031b516bb2 added the styling guarantee the same way. The recipe the file records is: ceiling held, revision regenerated, a test to keep it.

This change is that same register. It names status, claim and graph, all three already taught, and adds no flag. The ceiling holds at 173 lines and 1489 words against 180 and 1500 -- written to fit, because the ceiling test says a ceiling raised to accommodate whatever was just written is not a ceiling. metadata.revision is regenerated to 89529c87fb02, and the_skill_says_how_to_choose_work_beside_another_agent keeps the teaching against tokens no other line supplies.

So the ADR is not written, and the rest of the criterion is met whole. Writing one anyway would supersede a ratified decision to change nothing in it, which is worse than the gap it would close: it would put a signature on a document whose only content is that the previous one still holds.

Measured against 1510 words at first, not 1482: wc -w and the test disagree by 28 on this file, and the test's count is the one that decides. I trimmed against the wrong number once before noticing.
