---
id: TASK-47beb64fd204
type: task
slug: every-licence-declaration-in-the-tree-says-apach
title: Every licence declaration in the tree says Apache-2.0
created: 2026-08-17T19:39:22Z
author: claude-code/2.1.233+exposition
status: open
scope:
  - LICENSE
  - README.md
  - CLAUDE.md
  - crates/**
  - npm/**
  - Formula/**
  - bucket/**
  - packaging/**
  - package.json
blocked_by: []
done_criteria: |
  No file in the tree declares GPL, and grep is the check: the root LICENSE carries the Apache-2.0 text, all three crate manifests declare Apache-2.0 and each crate carries that text beside it, and every channel that declares a licence says Apache-2.0 -- the four npm package.json files and the root one, the Homebrew formula, the Scoop manifest, the winget locale. README.md, CLAUDE.md and npm/ank/README.md state one licence with no second answer, and the README badge agrees. The tree says somewhere a reader will find it that the change is prospective and that a release already made under GPL-3.0 stays available under it. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
schema: 3
version: 1
---

ADR-9f03438f5422 requires it. This is the mechanical half; the decision and what
it gives up are in the ADR, and it must be accepted on the default branch before
this is claimed.

The inventory, measured on 2026-08-17 -- fifteen declarations in eleven files,
plus the licence text at the root:

    LICENSE                                       the GPL-3.0 text itself
    crates/ank-cli/Cargo.toml                     license = "GPL-3.0-only"
    crates/ank-core/Cargo.toml                    license = "GPL-3.0-only"
    crates/ank-contract/Cargo.toml                already Apache-2.0, no text beside it
    package.json                                  "license": "GPL-3.0-only"
    npm/ank/package.json                          "license": "GPL-3.0-only"
    npm/ank-win32-x64/package.json                "license": "GPL-3.0-only"
    npm/ank-linux-x64-musl/package.json           "license": "GPL-3.0-only"
    npm/ank-darwin-arm64/package.json             "license": "GPL-3.0-only"
    Formula/ank.rb                                license "GPL-3.0-only"
    bucket/ank.json                               "license": "GPL-3.0-only"
    packaging/winget/Haksolot.Ank.locale.en-US.yaml   License: GPL-3.0-only
    README.md                                     badge, and the Licence section
    npm/ank/README.md                             the same sentence, again
    CLAUDE.md                                     "Ank is a CLI (Rust, GPL-3.0)"

Two things to get right rather than fast.

**The Apache text is copied, never typed.** A licence retyped is a licence
altered. `bitflags` 2.13.1 and `bstr` 1.12.1 ship byte-identical copies, 10847
bytes, and agreeing with each other is the check that neither was edited.

**The prospective clause is not decoration.** Whoever received a GPL-3.0 release
keeps it under GPL-3.0, and the tree must not read as though that were withdrawn.
Say it once, where a reader looking for the licence will find it.

`crates/ank-contract/**` is in the perimeter for a defect this repository shipped
on the day the crate was created: the manifest declares Apache-2.0 and no licence
text sits beside it, so it asserts terms a reader cannot read.
