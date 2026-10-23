---
id: ADR-b6fe84504ff2
type: adr
slug: a-remote-named-ank-carries-refs-ank-claims-and-r
title: A remote named ank carries refs/ank/claims/*, and ratifications, proofs and code state stay on origin
created: 2026-10-23T13:27:45Z
author: claude-code/opus-5.5+plan571-552
status: accepted
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-daemon/**
constraint: |
  When the repository has a remote named `ank`, every read and write of refs/ank/claims/* goes to it, and otherwise to origin; no configuration key chooses it. refs/ank/ratify/*, refs/ank/proof/* and every other refs/ank/* namespace are read and written on origin only, and the default branch, merge-base, completion judging and drift are read from origin only; with `ank` and no origin, claims are arbitrated through `ank` and every other ref stays local, as at level 0. init and the refspec repair of ADR-1b030fe70442 give `ank` a fetch refspec for refs/ank/claims/* and arrange origin's so that a fetch or prune from one remote never writes or deletes the namespace the other carries; status and check name the remote whose refspec is missing. The watcher mirrors claims from the remote that carries them. status names the coordination remote, in text and in --json, when it is `ank`.
amends: [ADR-1b030fe70442]
ratified: "682948602470"
anchor: 6388dff8f86bdcfa0ba7c3b41563961b1ce760d3
verified:
  - by: haksolot@vmi3223161
    at: 2026-10-23T14:06:13Z
schema: 4
version: 2
---

Raised by issue #552, and narrowed by the drift audit of 2026-10-23. Ank lives alone; a coordination remote is an addition any git server can be, AnkHub or another, and losing it must cost claims only.

## Why claims only, and not every refs/ank/*

The issue proposed routing every `refs/ank/*` to the remote named `ank`. Measured in a throwaway clone: an ADR accepted with its `refs/ank/ratify/<id>` on the coordination remote and not on origin, then a non-local clone fetching `+refs/ank/*` from origin as `ci.yml` does, and `ank check` reports "ratified, but no ratification commit is reachable: the freeze cannot be verified" as a signal at exit 0 -- the green-and-unverified state `ci.yml` warns against. The reverse breaks too: CI's `attest --detached` pushes `refs/ank/proof/*` to origin, the only remote a CI clone has, so a clone reading proofs from the hub misses them (ADR-1b030fe70442 measured that shape at 214 "done with no test proof" signals).

Ratifications and proofs anchor durable state; they travel with the code they anchor, on origin, where CI and every fork read them. Claims are ephemeral by construction (a lease of minutes), so a coordination remote that disappears loses nothing but leases. That is the line.

## Why a name, and not a key

SPEC-1a344e05197d makes the level a convention: "whether the repository has a remote named `origin`", and `crates/ank-cli/src/git.rs` gives the reason for not choosing among remotes: "picking one out of several would be a rule nobody declared". This ADR is that declaration. `git remote add ank <url>` is the whole setup, there is still no key to set, and `status` names the coordination remote whenever it is not origin, so a remote called `ank` for another reason is seen at once.

## What the audit found in place

- Every coordination round trip goes through five functions of `git.rs` (`remote`, `push_ref`, `ls_remote`, `ls_remote_refs`, `fetch_ref`); the change is a resolver keyed on the ref's namespace, not a rewrite.
- Completion records live in `refs/ank/claims/<id>` and move with the claims; pruning reads the default branch locally, so it is unaffected.
- The watcher already names its source in `refs/ank/watch/<remote>/claims/*` (ADR-4b45f344344f); only the constant `origin` in `crates/ank-daemon/src/fetch.rs` changes.
- ADR-af533e7a3e03, ADR-4e7c25b1f639 and ADR-9307e5d214a7 say "the remote", never origin, and stay whole. Peers (ADR-a1de673043b4) are other corpora read by path; one coordination remote serves exactly one corpus.

## The refspec, stated as a property

origin's `+refs/ank/*:refs/ank/*` (ADR-1b030fe70442) would fetch, and with `fetch.prune` delete, `refs/ank/claims/*` against a remote that no longer carries them, under the live claims of this clone. The constraint states the property -- a fetch or prune from one remote never writes or deletes the other's namespace -- and the mechanism (a negative refspec `^refs/ank/claims/*`, available since git 2.29, below the 2.34 ank requires) is measured by the task that writes it.

Left to the maintainer: ratification. SPEC-1a344e05197d (levels, the watcher's mirror) and SPEC-a81795811fa5 (`status --remote`) are revised by successors proposed beside this ADR.
