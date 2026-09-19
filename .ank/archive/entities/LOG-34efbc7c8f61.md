---
id: LOG-34efbc7c8f61
type: log
title: "Taught from the binary built at 056f8cb, not the ADR body: ank help context says --since names"
created: 2026-09-18T12:14:31Z
author: claude-code/opus-5+init-origin
scope:
  - skill/loop/SKILL.md
  - skill/SKILL.md
about: TASK-1ce4695da1b6
seq: 1
schema: 4
version: 1
---

 entities whose file changed and claims/completions recorded at or after the held claim's expires minus ttl, by id, renews the claim, refused at 6 without one; verbs.rs declares context Renews::Held. Measured in a scratch repo with that build: context --since with no claim -> error[6] naming ank claim <id>; after claim, a new task written 2s later, then --since twice: first SINCE 12:14:13Z (the claim) listed 6 changed + 1 claimed, second SINCE 12:14:15Z (the first call) listed 2 changed -- the read moved the cursor, as the loop skill now says. skill/loop/SKILL.md: context --since added to One pass after show, and a section 'Every turn under a claim' saying what it names, that it renews, the refusal, and that it does not replace the full ank context <path> before the claim. skill/SKILL.md: the context line lists --since. Revisions 0d916cc3d9a5 and 9f00f607cdb8; tests/skill.rs 36 passed; skills 134 lines/1137 words and 113/885, under 180/1500. Review against the criterion: the three clauses map to the One pass line plus section, the section's last paragraph, and the context line; nothing else changed.
