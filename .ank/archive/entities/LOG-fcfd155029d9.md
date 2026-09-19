---
id: LOG-fcfd155029d9
type: log
title: "decision, what scope a migrated entry carries: the scope of the entity it is about, copied at the"
created: 2026-08-15T09:51:28Z
author: claude-code/log-entities
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
about: TASK-df9c6d46e8ef
seq: 1
schema: 3
version: 1
---

 moment the entry is written, which is what section 3 already states for the kind and what the migration therefore applies to every entry it moves. An entry appears wherever its subject appears, which is the only placement that makes a trace findable by perimeter. The cost, stated rather than discovered: the copy does not track. When a task's scope is amended, or when the code under it moves, every entry already written keeps the perimeter as it stood, so the entries of one task can name several perimeters and an old entry can name a dead one. That is accepted on the same ground the specification gives, that a trace of work is a statement about a moment and not a live pointer, and on one more: tracking would mean rewriting an entity that is written once, which is the property the kind exists for. The dead-scope signal of ADR-97beaf55e73a is what makes a stale perimeter visible, and it needs no entry-specific rule.
