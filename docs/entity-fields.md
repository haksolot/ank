<!-- Generated from crates/ank-core/src/registry.rs and model.rs; do not edit.
     Regenerate: cargo run -q -p ank-core --bin entity-fields > docs/entity-fields.md -->

# Entity fields

Every kind, with its fields in canonical order: the kind registry the binary reads and writes with, printed. [The file format](format.md) says what the order and the emission forms mean; this page is the table it describes.

This build writes schema **4**, and reads schema **1** through **4**. A file declaring a newer schema is refused on its version, never on the first field it does not recognise.

## Task

`type: task`, ids `TASK-<12 hex>`.

| # | Field | Emission | Presence | Values | Notes |
|---|---|---|---|---|---|
| 1 | `id` | bare | always emitted | `TASK-<12 hex>` |  |
| 2 | `type` | bare | always emitted | always `task` |  |
| 3 | `slug` | scalar | omitted when absent |  | cosmetic, never resolved on |
| 4 | `title` | scalar | always emitted |  |  |
| 5 | `created` | scalar | always emitted |  | ISO 8601, always UTC with the `Z` suffix |
| 6 | `author` | scalar | omitted when absent |  | a typed actor; absent means the entity predates the field |
| 7 | `status` | bare | always emitted | `open` \| `in_progress` \| `done` \| `closed` |  |
| 8 | `scope` | block sequence | always emitted |  | globs, never empty |
| 9 | `blocked_by` | flow list | always emitted |  | task ids, `[]` when empty |
| 10 | `done_criteria` | literal block | omitted when absent |  | frozen by hash at claim |
| 11 | `criteria_by` | bare | omitted when absent | `creator` \| `claimer` | invalid without `done_criteria` |
| 12 | `verify` | flow list | omitted when absent |  | verifier names `config.yml` declares |
| 13 | `method` | scalar | omitted when absent |  | one sibling skill the binary carries |
| 14 | `proof` | block sequence of maps | omitted when absent |  | keys in order: `type`, `ref`, `tree`, `criteria`, `verifier`, `via` |
| 15 | `verified` | block sequence of maps | omitted when absent |  | readings: `by`, then `at`, both required in an entry |
| 16 | `schema` | integer | always emitted |  |  |
| 17 | `version` | integer | always emitted |  |  |

## ADR

`type: adr`, ids `ADR-<12 hex>`.

| # | Field | Emission | Presence | Values | Notes |
|---|---|---|---|---|---|
| 1 | `id` | bare | always emitted | `ADR-<12 hex>` |  |
| 2 | `type` | bare | always emitted | always `adr` |  |
| 3 | `slug` | scalar | omitted when absent |  | cosmetic, never resolved on |
| 4 | `title` | scalar | always emitted |  |  |
| 5 | `created` | scalar | always emitted |  | ISO 8601, always UTC with the `Z` suffix |
| 6 | `author` | scalar | omitted when absent |  | a typed actor; absent means the entity predates the field |
| 7 | `status` | bare | always emitted | `proposed` \| `accepted` \| `superseded` |  |
| 8 | `scope` | block sequence | always emitted |  | globs, never empty |
| 9 | `constraint` | literal block | always emitted |  | binding on every scope it covers once accepted |
| 10 | `see` | scalar | omitted when absent |  | reference code the constraint points at |
| 11 | `supersedes` | bare | omitted when absent |  | an entity id |
| 12 | `ratified` | scalar | omitted when absent |  | the signed commit `accept` wrote |
| 13 | `verified` | block sequence of maps | omitted when absent |  | readings: `by`, then `at`, both required in an entry |
| 14 | `schema` | integer | always emitted |  |  |
| 15 | `version` | integer | always emitted |  |  |

## Spec

`type: spec`, ids `SPEC-<12 hex>`.

| # | Field | Emission | Presence | Values | Notes |
|---|---|---|---|---|---|
| 1 | `id` | bare | always emitted | `SPEC-<12 hex>` |  |
| 2 | `type` | bare | always emitted | always `spec` |  |
| 3 | `slug` | scalar | omitted when absent |  | cosmetic, never resolved on |
| 4 | `title` | scalar | always emitted |  |  |
| 5 | `created` | scalar | always emitted |  | ISO 8601, always UTC with the `Z` suffix |
| 6 | `author` | scalar | omitted when absent |  | a typed actor; absent means the entity predates the field |
| 7 | `status` | bare | always emitted | `proposed` \| `accepted` \| `superseded` |  |
| 8 | `scope` | block sequence | always emitted |  | globs, never empty; what the document governs |
| 9 | `references` | flow list | omitted when absent |  | entity ids |
| 10 | `supersedes` | bare | omitted when absent |  | an entity id |
| 11 | `ratified` | scalar | omitted when absent |  | the signed commit `accept` wrote, over the body and `scope` |
| 12 | `verified` | block sequence of maps | omitted when absent |  | readings: `by`, then `at`, both required in an entry |
| 13 | `schema` | integer | always emitted |  |  |
| 14 | `version` | integer | always emitted |  |  |

## Log entry

`type: log`, ids `LOG-<12 hex>`.

| # | Field | Emission | Presence | Values | Notes |
|---|---|---|---|---|---|
| 1 | `id` | bare | always emitted | `LOG-<12 hex>` |  |
| 2 | `type` | bare | always emitted | always `log` |  |
| 3 | `slug` | scalar | omitted when absent |  | cosmetic, never resolved on |
| 4 | `title` | scalar | always emitted |  | the message, or its head |
| 5 | `created` | scalar | always emitted |  | ISO 8601, always UTC with the `Z` suffix; the instant of the entry |
| 6 | `author` | scalar | omitted when absent |  | a typed actor; who wrote the entry |
| 7 | `scope` | block sequence | always emitted |  | the subject's scope as it stood |
| 8 | `about` | bare | always emitted |  | an entity id of any kind |
| 9 | `seq` | integer | always emitted |  | rank among that entity's entries, from 0 |
| 10 | `records` | scalar | omitted when absent | `edit` \| `create` \| `method` | absent is work; a value unknown to the reader is read as machinery |
| 11 | `verified` | block sequence of maps | omitted when absent |  | readings: `by`, then `at`, both required in an entry |
| 12 | `schema` | integer | always emitted |  |  |
| 13 | `version` | integer | always emitted |  | above 1 means the entry was rewritten |

## `records`

What a log entry a verb wrote records. Absent, the entry is work; a value this build does not know is read as machinery and never refused.

| Value | Records |
|---|---|
| `edit` | a change of content outside a status transition: the fields, the versions, the hash replaced and the hash produced |
| `create` | the creation of its subject: version 0 to 1 and the hash produced |
| `method` | a sibling skill opened under a claim; the title is its name |

## Proof `type`

What a proof entry's `ref` points at. A weak type anchors nothing outside the agent's reach, and `check` marks it.

| Value | Trust | Meaning |
|---|---|---|
| `test` | strong | a test run, by a reference to it |
| `commit` | strong | a commit the work is in |
| `human-review` | weak | somebody read the work |
| `assertion` | weak | a statement, and nothing behind it |

## Proof `via`

The route by which a proof entry arrived. Absent means the entry was written before the field existed, never a fourth route; a `test` entry `submitted` by a caller anchors nothing outside the agent's reach.

| Value | Meaning |
|---|---|
| `verifier` | ank ran a verifier `config.yml` declares; its own statement |
| `attested` | reached the task on `refs/ank/proof/<id>`, written by whoever held the pipeline |
| `submitted` | a caller passed it to `done --proof` or `attest --proof`; recorded as given |
