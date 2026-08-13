---
id: ADR-9d4b7e2a5c13
type: adr
slug: format-is-the-spec
title: The format is the specification, and ank-core is its reference implementation
created: 2026-07-27T09:00:00Z
status: superseded
scope:
  - crates/ank-core/**
  - docs/**
constraint: |
  Every format change happens in this order: the specification first, then the
  golden files, then the code. The round-trip must stay byte-identical. No
  field exists in the code without existing in the specification.
see: crates/ank-core/tests/golden/
schema: 1
version: 3
---

Ank promises that any third-party tool can read and write `.ank/` files without
going through the CLI. That promise only holds if the format is described before
it is implemented, and if the reference implementation never lets it drift
silently.

The order specification, then goldens, then code is not ceremonial: writing the
golden before the code forces an explicit decision about the canonical form,
rather than letting it emerge from the serialiser's implementation.

These bootstrap ADRs are ratified by the repository's history rather than by a
signed commit: `allowed_signers` is empty while the project is solo, and `check`
must report that absence as an accepted limitation rather than hide it.
