---
id: TASK-0da5af5afd5f
type: task
slug: ci-runs-ank-check-and-the-agent-facing-prose-sto
title: CI runs ank check, and the agent-facing prose stops describing a CLI that does not exist
created: 2026-07-31T16:24:04Z
status: done
scope:
  - .github/workflows/ci.yml
  - README.md
  - CLAUDE.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  CI invokes the ank binary rather than the deleted check_repo example, and the workflow is green on ubuntu-latest, macos-latest and windows-latest. No file outside .ank/ names check_repo. README states that the CLI exists, and its instructions for agents working on this repository route through ank context and ank check rather than through reading and editing .ank/ by hand. cargo test, cargo fmt --check and ank check are green.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/9afaac97c843@eeec2a4
    tree: scope/4cff8d321f88
    criteria: bb655bcaf9d3
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@eeec2a4
    tree: scope/4cff8d321f88
    criteria: bb655bcaf9d3
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/cc95691ee8b1@eeec2a4
    tree: scope/4cff8d321f88
    criteria: bb655bcaf9d3
    verifier: check-repo@5734e9cf9d3d
  - type: test
    ref: ci://haksolot/ank/runs/30647629487
  - type: commit
    ref: 6d5fd21
schema: 1
version: 8
---

`fcd3934` retired `crates/ank-core/examples/check_repo.rs` in favour of the real
`ank check` and left `ci.yml` invoking the deleted example. The workflow has
been red on all three runners since that commit, and nothing caught it: the
three verifiers in `config.yml` run locally, and none of them executes `ci.yml`.
A verifier that cannot see the file it should be checking is the same blind spot
in a different place.

The prose half is the part that matters more. `README.md` still says the CLI
does not exist and tells agents to read and edit `.ank/` by hand — the file
written to orient an agent points it away from the tool. That is not a stale
comment, it is an instruction, and it is wrong.

The `.ank/**` occurrences of `check_repo` are left alone: log entries and
`done_criteria` of tasks already `done` are historical anchors, and rewriting
them would falsify what was true when they were written (ADR-85e6bbb195b8).
Hence the criterion says "no file outside `.ank/`".

Blocks TASK-b8c9d0e1f2a3: `release.yml` is written beside `ci.yml` and inherits
its shape, and there is no sense in adding a second workflow while the first one
is red.

Scope widened after claiming to `claim.rs` and `human.rs`: the criterion says no
file outside `.ank/` names `check_repo`, and two comments there still did. The
criterion is untouched — it is the scope that was drawn too narrow, because I
wrote it from the grep of markdown and workflow files and not from the grep of
everything. Two comments, no code.

**One clause of the criterion cannot be proved here.** "Green on the three
runners" needs a push, and nothing is being pushed. What is verifiable locally
is that the command CI runs is the command that works; the run itself is
evidence deferred to the first push, and the proof will say which clause it
covers rather than implying the whole.

## Log
- 2026-07-31T16:30:11Z seanl@sean-laptop — CI ran a deleted example since fcd3934 and was red on all three runners; it now runs the same line as the check-repo verifier. README told agents the CLI did not exist and to edit .ank/ by hand, which was the worse half: it is the file written to orient an agent, pointing it away from the tool. Scope widened after claiming to claim.rs and human.rs, two comments, because the criterion says no file outside .ank/ and I had drawn the scope from a grep of markdown and workflows only.
- 2026-07-31T16:30:45Z seanl@sean-laptop — done, proof test:local/9afaac97c843@eeec2a4 test:local/e3b0c44298fc@eeec2a4 test:local/cc95691ee8b1@eeec2a4
- 2026-07-31T16:34Z seanl@sean-laptop — the three proofs above cover every clause of the criterion but one: "green on ubuntu-latest, macos-latest and windows-latest" needs a push, and nothing has been pushed. What is proved locally is that the line CI now runs is the line that works, byte for byte the same as the check-repo verifier. The remaining clause is evidence deferred to the first push, and it should be checked there rather than assumed here. Also noted while diffing, and out of scope: .gitattributes pins only .ank/** and the goldens, so on a Windows clone with core.autocrlf=true every source file comes out CRLF, and the "line endings" step in ci.yml greps only those two paths. That belongs to TASK-aca0cb103980, which owns the CRLF question.
- 2026-07-31T16:35Z seanl@sean-laptop — identity here is seanl@sean-laptop and not claude-code@ank as in earlier entries: ANK_AGENT was unset in this shell. Recorded rather than corrected, because the claim ref was taken under this identity and rewriting the entries would falsify who did the work (ADR-85e6bbb195b8).
- 2026-07-31T16:40Z seanl@sean-laptop — the deferred clause is now proved: run 30647629487 green on ubuntu-latest, macos-latest and windows-latest, appended as ci:// because the environment is out of the agent's reach. One correction to the entry above and to the commit message, both already written and not rewritten: I said CI had been red since fcd3934, and it had not. The push reported 30b7038..6d5fd21, so fcd3934 and eeec2a4 had never left this machine and the last remote run (30604724417) was green on a tree that still contained check_repo. The break was latent, not observed: it would have fired on this push and did not, because the repair went out in the same push. The repair was needed either way, and this run is the first to exercise the tree fcd3934 left.
