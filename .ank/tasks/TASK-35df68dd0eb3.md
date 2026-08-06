---
id: TASK-35df68dd0eb3
type: task
slug: record-multi-repository-federation-as-deferred-i
title: Record multi-repository federation as deferred in the spec
created: 2026-08-06T23:25:19Z
author: seanl@sean-laptop
status: open
scope:
  - docs/**
blocked_by: []
done_criteria: |
  The deferred table in section 10 of the current spec revision contains an entry 'Multi-repository federation' whose Reason column names the retained shape - one .ank/ per repo stays authoritative, aggregation happens above it - and the explicit trigger for pursuing it.
criteria_by: creator
schema: 2
version: 1
---

Working from a root directory over several linked repositories has no answer in
ank today, and the corpus records the absence nowhere - no task, no ADR, no line
in the section 10 deferred table. That silence reads as "not thought about"
when the truth is closer to "thought about and locked shut by seven separate
decisions".

The shape retained, should it ever be pursued: one .ank/ per repository stays
authoritative - a corpus belongs to the code it constrains, and a single corpus
hoisted to the root would break that - with aggregation happening above the
repositories, never instead of them. Reading across corpora is the useful part;
writing across them is not.

The seven locks, measured, because that is what makes this entry worth anything
later:

1. Repo carries one root and discover returns the first .ank/ it finds walking up
   (repo.rs:15-48). There is no collection type and no notion of a primary.
2. cli.rs:894 resolves exactly one repo BEFORE dispatch, and the doc comment at
   cli.rs:907-918 states as an invariant that a verb never resolves the repo
   itself. Per-verb multi-root resolution violates a written design rule.
3. --repo is single-valued, and section 4 fixes the global flag count at three
   while arguing that every global flag is a memorisation cost. --repos, or a
   repeatable --repo, is a spec change before it is a code change.
4. config.rs uses deny_unknown_fields on all three structs, so a repos: key
   cannot even be added speculatively: the file would be rejected. Any additive
   key needs a schema bump.
5. IDs carry no namespace - twelve hex from a hash of timestamp, identity, title
   and entropy (id.rs:5,56-78) - and the short display prefix is computed per
   corpus (context.rs:219-225). TASK-8ebd can mean two different things in two
   repositories, and TASK-abbaab9007a0 already flags short-id ambiguity as a
   papercut within a single one.
6. blocked_by must resolve inside the local store or the write fails with code 2
   (commands.rs:111-116,447), and a scope that is absolute or climbs above the
   root is refused outright (scope.rs:17-29). A task cannot reference, or even
   point at, anything outside its repository.
7. index.db and refs/ank/* are per repository. A cross-repo claim has no home,
   which is the same arbiter problem as TASK-83d6eefdb36e one level up: claims do
   not even cross clones of ONE repository yet.

The remedy available today, with no code: run the verb per repository,
ank --repo ./repoA context, and accept that there is no aggregated view.

The trigger for reopening this: repeated real work where a task in one repository
is genuinely blocked by a task in another and the pair is being tracked by hand.
Convenience of a single dashboard is not the trigger - ADR-bcb18aecb7e1 already
places a read-only local viewer over one .ank/, and that is the cheaper answer to
"I want to see everything at once". Do not implement federation under this task;
if the trigger fires, that is a new task plus a spec revision.
