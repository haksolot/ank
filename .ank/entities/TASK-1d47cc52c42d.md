---
id: TASK-1d47cc52c42d
type: task
slug: the-specification-records-that-three-deferred-ro
title: The specification records that three deferred rows have fired
created: 2026-08-15T06:56:12Z
author: claude-code/opus-5
status: in_progress
scope:
  - docs/**
blocked_by: []
done_criteria: |
  A new revision of docs/ank-spec-v1.1.md records the trigger as fired for the rows 'Spec sections as routable entities' and 'Multi-repository federation', and records that ADR-ff294eff4d1a decided between a body section and a file and never considered an entity per entry. Section 3 declares the kinds spec and log in the registry, each with its field table. Section 7 states the federation shape ADR-a1de673043b4 fixes. The three passages that still describe the log as an append-only section of the entity file are corrected, as is the sentence claiming git unions two appends with no driver. No source file path and no line number enters the document. ank check stays green.
criteria_by: creator
schema: 3
version: 3
---

The specification is the source of truth and it moves first: format changes go
through this document, then the goldens, then the code. Every other task in this
group is blocked on this one for that reason, and not because the prose is
prettier written early.

Three rows change, and each for a different reason. Do not flatten them into one
sentence about pressure.

**Spec sections as routable entities.** The refusal stands as written for what it
refused — fragmenting one document into scoped entities that drift apart. What
fires is not that. It is that the remedy the row offers instead, distilling a
section into a scoped ADR, has **zero instances** across thirty-five ADRs in the
corpus that wrote the refusal, and that the vehicle it names cannot carry the
load: the longest constraint ever written is 1251 characters, the field is frozen
at ratification, it is invisible during orientation, and the over-constrained
ceiling is already exceeded by eight tasks. Record that the trigger as worded was
unfalsifiable by construction, and that what is admitted is a whole-document kind,
never a section.

**Multi-repository federation.** The trigger there is written for dependencies. It
does not anticipate the shared constraint, which today has no legal expression at
all: a scope cannot reach a sibling, because an absolute path and a climb above
the root are both refused and a glob that named one would become the label scope
exists to avoid; and the copy is already forbidden by ADR-e3cb36646d77. Neither
scope nor copy is the hole. Record it as such, and keep the shape the row already
retained: one corpus per repository stays authoritative, aggregation above and
never instead.

**The log.** ADR-ff294eff4d1a is not superseded and nothing here says it was
wrong. It decided between a body section and a file of its own, and never
considered an entity per entry, so the record gains a sentence saying that rather
than a reversal.

**Four corrections that are pure defect.** Three passages still describe the log
as an append-only section of the entity file: the settled list in the revision
summary, and two in section 12, one of which still lists `log = timestamped
union` among the merge driver rules that section 7 removed. And section 7 claims
git's own union resolves an appended file with no driver, which is false —
nothing in the repository configures `merge=union`, and git's three-way merge
conflicts on two appends. The property that actually protects is one file per
entity, which comes from the addressing.

**No source file path and no line number goes into this document.** They are what
rots; TASK-35df68dd0eb3 learned it the hard way and its log says so.
