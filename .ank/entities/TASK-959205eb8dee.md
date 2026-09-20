---
id: TASK-959205eb8dee
type: task
slug: getting-started-walks-to-a-green-check-and-gives
title: getting-started walks to a green check and gives the whole CI recipe
created: 2026-09-19T18:26:18Z
author: claude-code/opus-5+docs-audit
status: done
scope:
  - docs/getting-started.md
blocked_by: []
done_criteria: |
  Following docs/getting-started.md command by command in a fresh repository, with an origin and without one, reproduces every output it shows up to the shown version and ids, and the walk's final ank check exits 0. Its CI section names fetch-depth: 0 and why, and its attest recipe says how ids are selected, fetches refs/ank/proof/*, and states the permission and the default-branch gate ci.yml uses. Its exit-code table lists every code in crates/ank-contract/src/exit.rs. ANK_AGENT examples use a typed actor. default: true verifiers and --no-verify are taught. The number of install routes it names agrees with docs/agents.md.
criteria_by: creator
proof:
  - type: commit
    ref: 4fda1d5d0de9eedd23939bd52cd7ce7038700c74
    criteria: b375d9da9ab0
    via: submitted
schema: 4
version: 3
---

Findings from the 2026-09-19 audit: init output shows .ank/tasks and .ank/adr; the walk ends at exit 8 on a dead scope and error[9] on a missing verifier script; a shallow checkout gives a green check that verifies nothing; versions 0.1.3 and 0.7.0 and stale skill revisions; code 1 missing; 'Five routes' and 'the four routes' in one page; the context example shows the constraint where the binary prints the title before a claim. No declared verifier reads prose, so verify: is left empty on purpose and the close is a commit proof; the future doc-output test (ADR-2b62b9a1fe67) is what will hold these pages afterwards.
