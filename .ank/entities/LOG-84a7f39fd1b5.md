---
id: LOG-84a7f39fd1b5
type: log
title: "Measured before answering, as the criterion asks. Corpus: 18 accepted constraints, 11 open tasks,"
created: 2026-08-13T06:12:16Z
author: claude-agent-b
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/**
about: TASK-1ead0e19fb73
seq: 0
schema: 3
version: 1
---

 context_budget 8000 characters. Cold orientation at the root spends 7357 characters on constraints and 157 on tasks -- 96 percent against 2 percent. Seven constraints are rendered in full and eleven are not shown at all; one task line is printed and eleven are cut. The PROPOSED section is cut to a single 'not shown' line as well. With a perimeter given (crates/ank-cli/src) the allocation is unchanged: 7373 against 181, seven constraints in full, still one task shown. That is the finding the task body only hypothesised. Narrowing the perimeter narrows the candidate set -- eleven tasks cut becomes five -- but frees no budget for them, because the constraints are charged first and at full length either way. So a perimeter does not fix it and cannot: the allocation is the defect, not the breadth. The sharpest way to state it is the last line of the output. Orientation ends with 'ank claim TASK-10b8 to start', recommending the one task it had room to print. The mode whose whole purpose is choosing presents exactly one candidate out of twelve, and an agent reading it has no way to know eleven others exist without running a second command. Execution mode is untouched by any of this and must stay so: there the perimeter is settled and a truncated constraint would hide a rule the agent is about to break.
