---
id: ADR-aaee5307b34c
type: adr
slug: a-push-the-remote-rejects-is-classified-by-the-r
title: A push the remote rejects is classified by the ref's state, and the remote's reason is reported verbatim
created: 2026-10-23T13:27:44Z
author: claude-code/opus-5.5+plan571-552
status: proposed
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/claim.rs
constraint: |
  git push --porcelain joins the invocations ADR-9307e5d214a7 admits, and of it only the stdout status line of the pushed ref is read: its flag, `rejected` or `remote rejected`, and the parenthesised reason; stderr and `remote:` text are never read. Classification stays by state: after a failed push one ls-remote of the ref decides, and no answer is an unreachable remote as before; a remote ref differing from the expected witness is a lost race; a `remote rejected` whose ref still equals the witness is a refusal by the remote. A refusal reports the reason verbatim, gives no reason a meaning, names nobody as holder and never becomes a local win: claim exits 9 and leaves no local claim; log, done, release and close keep their local write and exit 0 with a warning naming the reason; attest --detached and accept exit 9 as for an unreachable remote. The retry without a witness is taken only on a client-side `rejected (stale info)` against an absent ref. A push whose stdout carries no status line for the ref keeps the ls-remote reading alone.
amends: [ADR-9307e5d214a7, ADR-af533e7a3e03]
schema: 4
version: 1
---

Raised by issue #571, and narrowed by the drift audit of 2026-10-23, which measured the present behaviour against a bare remote whose `update` hook refuses `refs/ank/*` (git 2.47.3, ank 0.9.2, push path unchanged at 9035be27).

## What ank does today, measured

`push_ref` (`crates/ank-cli/src/git.rs`) never reads the push's output: one `ls-remote` of the ref decides, an answer meaning a lost compare-and-swap and no answer an unreachable remote (SPEC-1a344e05197d, level 1: "told apart by asking, not by reading stderr"). Against a remote that refuses for a reason of its own, the remote is reachable, so every refusal reads as a lost race:

- `ank claim` exits 4 with `held by <caller>`: the caller is named as the holder that blocks it, the local ref keeps the caller's record, and `status` then shows a claim the remote refused.
- `ank log` on a claim pushed earlier warns `taken over while logging`, when the remote still holds the caller's own record.
- `ank release` prints `released` with no warning; the remote ref survives and the next fetch puts it back.

The issue's other fear, a refusal turned into a local win, does not occur: that path is taken only when `ls-remote` fails too.

## The decision

**The remote's reason is reported, never interpreted.** Ank stays usable alone and against any remote: GitHub's rulesets, a Gitea hook, a hand-written `pre-receive` and a coordination hub all refuse the same way, through report-status, and ank reads the one field git documents as machine-readable (`git push --porcelain`: "tab-separated and sent to stdout", one `<flag>\t<from>:<to>\t<summary> (<reason>)` line per ref). It assigns no vocabulary to the reason. A stock hook says `hook declined`; a hub may say `identity`; ank prints either verbatim and behaves identically.

**Classification stays by state, which is ADR-9307e5d214a7's real objection.** The plumbing rule forbids the human sentences, not the machine status line; `--porcelain` here is a flag of `push`, which ADR-9307e5d214a7 already admits, not one of git's porcelain commands. But the line only says *that* the remote rejected and *what it said*; *whether* the swap was lost is still asked of the remote with `ls-remote`. That keeps a server-side lost race correct: a ref moved between advertisement and update comes back as `[remote rejected] (failed to update ref)` or `(cannot lock ref ...)`, and the state, not the summary, makes it a lost race.

**A lost lease is client-side.** Measured: a lease with a witness against a ref the remote does not have is `!\t...\t[rejected] (stale info)`, exit 1, no `remote rejected`. That is the only case TASK-2d779142ca70's retry without a witness exists for; against a refusing remote the retry only pushes again into the same refusal.

**Exit codes, verb by verb (ADR-af533e7a3e03 stays whole).** `claim` fails with 9, not 4: 4 tells a loop the task is taken and to pick another, and a remote that refuses this identity would then refuse every task in turn; 9 is an environment to repair, which a refused credential or a refused namespace is. The local ref is unwound, so the clone never believes in a claim the remote refused. `log`, `done`, `release` and `close` wrote something to disk that stands, so they degrade, exactly as for an unreachable remote, but the warning names the remote's reason instead of "taken over" or nothing. `attest --detached` and `accept` already exit 9 when their ref does not reach the remote.

**What it does not change.** No verb, no exit code, no configuration. A push with no status line for the ref (an older transport, a remote that says nothing) keeps today's reading. `status` keeps no memory of a past refusal: the verb that met it reports it, in text and in `--json`, when it happens.

Left to the maintainer: ratification. SPEC-1a344e05197d's sentence about stderr and SPEC-a81795811fa5's exit-code table are revised by successors proposed beside this ADR.
