---
id: LOG-7cb1db83504f
type: log
title: drift audit 2026-08-31, re-measured and holds for find. 'ank find' human output ends '+1681 more,
created: 2026-08-31T07:54:06Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/**
about: ADR-3e6ce108edcd
seq: 0
schema: 4
version: 1
---

 narrow with --scope <path> or --type task|adr|spec|log' -- the cut and its notice. 'ank find --json' answers total 1721, shown 1721, hidden 0 and carries 1721 rows: the budget cut the human and not the program. shown and total agree under every filter measured -- no filter 1721/1721/0, --type adr 84/84/0, --scope crates/ank-core 177/177/0, --status open 0/0/0 -- and hidden stayed 0 throughout. context --json keeps its budget as the constraint requires. One asymmetry is worth a reader's eye rather than a claim of violation: 'ank review' prints 76 dead-scope identifiers to a human, where 'review --json' carries "dead": 76 and no rows, the count being what crates/ank-contract/src/verbs.rs:944 declares (Type::Num) beside proposed and live, which are arrays. Whether that is a listing this constraint reaches or a summary counter like faults and signals is a judgement for the human; it is recorded here rather than written as a task, and closing it would retype a field within a contract version, which ADR-6fd69efb629c forbids without a version bump.
