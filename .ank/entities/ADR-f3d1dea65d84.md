---
id: ADR-f3d1dea65d84
type: adr
slug: a-verb-pays-for-the-answer-it-gives-and-status-d
title: A verb pays for the answer it gives, and status does not read the corpus to count what it reports
created: 2026-08-27T16:31:52Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/**
constraint: |
  A verb pays for the answer it gives. A verb whose answer is a handful of fields does not pay a cost that grows with the corpus to produce them, and a counter it reports is served from the index that already holds the parse rather than from a fresh read of every file on disk.
  
  status is where this is measured, because it is the verb a reader asks first and the one that costs the most. It reports faults and signals, and it obtains them without reading the corpus a second time: the verdict of the mechanical check is memoised in the index, keyed on the state the index already tracks for the files together with the tip of refs/ank/*. Both halves of that key are required and neither is sufficient, because the check inspects orphaned claims, which live in the refs and not in the files. On a key that does not match, the verb pays once and stores what it found.
  
  What the verb answers does not move to buy this. status --json keeps every field it has, under the same key and with the same type: the machine contract lets a document gain a field within a version and never lose, rename or retype one, so no counter becomes absent, optional or null in exchange for speed. Speed is bought from where an answer is computed and never from what the answer says.
  
  A cache whose key is wrong makes the verb lie, and a lying status is worse than a slow one. So the key is stated here rather than left to the implementation, and a verb that cannot establish its key pays the full cost rather than guessing.
schema: 4
version: 1
---

`ank status --json` costs 2750 to 2850 milliseconds on this corpus of 1471
entities. `ank find --json`, which returns every one of those entities and 216
kilobytes of JSON, costs 530 to 820. The verb that answers in fourteen fields is
four times more expensive than the verb that answers with the whole corpus.

## Where it goes

`crates/ank-cli/src/status.rs:256` calls `human::inspect(repo, cfg, None,
false)`. That walks both storage layouts, parses every entity file in the
corpus, and returns a `Report`. Three of that report's fields are used:
`report.drift`, `report.faults()` and `report.signals()`. The last two are
printed as two integers at `status.rs:404-408`.

So the corpus is read from disk, in full, to print `"faults":0,"signals":396`.

The `prune: false` argument is right and stays: a reader does not sanitise the
coordination plane underneath everyone else. It is also not where the cost is.
The cost is the read, and the read is not needed.

## Why a cache and not a flag

The obvious answer is to compute the counters only when asked. It is not
available, and the reason is written into `crates/ank-contract/src/lib.rs:40`:

> Within one version a document may *gain* a field, and may never lose, rename
> or *retype* one.

A `faults` that is a number when asked for and `null` otherwise has been
retyped. So has one that is absent without a flag. Either would force
`CONTRACT_VERSION` from 1 to 2, which is the most expensive change this
repository can make, and it would buy nothing a cache does not.

The `null` idiom ten lines further down in the same file -- `refs` is null
without `--remote`, because "null is the question never asked" -- is not a
counter-example. That field was born null. Retyping one that was born a number
is the thing the contract forbids.

## What this is not

It is not a claim that verbs must be fast. `ank check` reads the corpus because
reading the corpus is its answer, and it should go on costing what it costs.
The rule is narrower and harder: a verb does not pay for an answer it is not
giving. `status` gives a summary, and a summary is not a re-derivation.

Nor does it make the daemon load-bearing. ADR-a22cd3196529 says nothing depends
on `ank watch`, and nothing here does: the cache lives in the index every verb
already opens, and a cold index simply pays once.
