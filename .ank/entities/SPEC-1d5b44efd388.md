---
id: SPEC-1d5b44efd388
type: spec
slug: intent-principles-and-what-v1-leaves-out
title: Intent, principles, and what v1 leaves out
created: 2026-08-15T17:45:36Z
author: claude-code/opus-5
status: accepted
scope:
  - "**"
ratified: 9fec073faafe
schema: 3
version: 2
---

This is one of the ten documents that carry the Ank specification, and it is the
one to read first: what Ank is for, the principles the other nine are measured
against, and the register of what v1 deliberately leaves out.

## The decomposition, and the test that produced it

The specification was one file, `docs/ank-spec-v1.1.md`, 1634 lines and a quarter
of a megabyte. It is now ten entities of kind `spec`, each ratified and anchored
on its own, with the coherence between them verified rather than assumed
(ADR-5a690829388d). That path is not empty: it holds an index naming the ten and
carrying no rule of its own.

**The boundaries are not the section numbers.** Thirteen sections would have made
thirteen entities, and that is the fragmentation §10 refused, wearing a ratified
decision as a disguise. The test applied to every candidate boundary is the one
the decision states: **a document that cannot be read without holding another one
open is not a document.** Applied honestly it merges more than it splits — four
merges against two splits, and the thirteen sections became ten documents:

- §1, §2, §10 and the revision record are **one** document, this one.
- §3 is one document, undivided.
- §4 is three: the surface, the presentation grammar, and the proof half.
- §4's proof half joins **§8**, four sections away, because they are one argument.
- §5 joins **§11**, because a measurement and its only sink are one subject.
- §6, §7 and §9 are one document each.
- §12 joins **§13**.

Each of the ten argues its own boundary in its own body, which is worth more than
the split: it is what stops the next reader from moving a rule across a boundary
because it looked tidy.

**What crosses a boundary is a declared reference and never a copy.** Where a
document defers a rule to another, it says so in `references`, and `check`
reports a reference that names an entity the corpus does not hold, one not yet
accepted, or one superseded without the citing document following the chain. A
reference is a dependency and not a credit: the ADRs that produced a rule are
named in the prose where they produced it, and declaring them here would record
the argument's history rather than this document's dependencies.

**Cross-references keep their section numbers.** A rule that read `(§7)` still
reads `(§7)`, because rewriting two hundred of them into identifiers would have
been two hundred chances to lose one, and because the numbers are now the names
of the parts rather than positions in a file. The map from a number to a document
is in `docs/ank-spec-v1.1.md`, which is an index and nothing else.

## Why these four sections are one document

They are the same act at four scales. §1 says what Ank is for and, in *Non-goals*,
what it refuses to be. §2 says how it decides. §10 says what it does not build
yet, and under what trigger each row would fire. The revision record says what
each revision argued. A reader asking *does this belong in Ank* reads exactly
these four and needs no mechanism at all — which is the test, satisfied in the
strongest form available: this document rests on nothing.

**It therefore declares no references, and that is a decision.** The register's
rows name mechanisms freely — the federation row enumerates seven locks that live
in §6 and §7 — but it names them for detail and defers to none of them for
authority. A citation from here would invert the reading order, making the
document that judges the others depend on them.

## The revision record, and why it closes here

The dated paragraphs at the end are the record of what each revision of the
monolith argued, kept entire because they are the only place several decisions
state their reasoning. **The register closes with this revision.** A revision is
now a supersession of one document, recorded in that document's own chain and
anchored in its ratification commit, so a fourteenth dated paragraph would be a
second history competing with the one the corpus keeps mechanically.

The document these ten replace was titled *Ank — Specification v1.1*, and the two
lines of its header are the first thing the record below carries.

---

## Who this document is for

This is the normative document and the source of truth: every question the
other two defer to it is settled here, and where any of them disagrees with
this one, this one is right.

It is not a tutorial, and it no longer stands in for one. A reader who wants to
install Ank and finish a first task reads [getting-started.md](getting-started.md);
a reader implementing a tool that reads or writes `.ank/` reads
[format.md](format.md), which carries the mechanical half — field order,
emission rules, conformance suite — and points back here for every rule it
depends on. Both are written so that this document does not have to teach.

What is here instead is the argument. Sections state a decision, the
alternatives rejected, and what the decision costs, because a specification
that records only its conclusions cannot be revised by anyone who was not in
the room.

## 1. Intent

Ank makes a repository's **organisational layer readable and actionable by agents**.

Tasks, architecture decisions, trade-offs: everything that today lives in a tracker, a wiki or a thread, and is therefore never available to an agent that spawns on the code. Ank puts that information in the repository, attached to the code it concerns, in a format an agent consumes without effort and without excessive token cost.

### Non-goals

These exclusions are the specification, not omissions.

- **Not a Linear competitor.** No cycles, estimates, velocity, roadmap or burndown. Ank can export to a tracker for human visibility; it does not replace one.
- **Not a wiki.** Only what is *actionable or binding for an agent* belongs in Ank. A decision that constrains code: yes. Meeting notes: no.
- **Not a security boundary.** Permissions protect against an agent going off the rails, not against a malicious actor.

### Success criteria

1. An agent that spawns on a path gets everything that constrains it in a single call, under 2000 tokens.
2. An agent cannot declare itself "done" without proof.
3. An agent cannot weaken a constraint to unblock itself — and when it sets its own bar, that is visible.
4. The tool works solo and local, with no configuration and no service.


## 2. Design principles

| Principle | Concrete consequence |
|---|---|
| The format is the specification | The CLI is a reference implementation, not a gatekeeper. Any tool can read and write. |
| Immutability is verifiable, not defended | The CLI cannot prevent a direct edit; every freeze is therefore anchored by a hash in an artifact the editor does not control, and `check` compares. |
| Shell rather than MCP | The common denominator across agents. No protocol serialisation per call. |
| Anchor, do not trust | Every state transition requires externally verifiable proof. |
| Immutable by default | You do not modify a decision, you replace it. Weakening becomes visible. |
| Terse by default | `git status`-style output. JSON is strictly opt-in. |
| Degrade, do not fail | With no remote and no daemon, Ank still works, in reduced mode. Degradation covers services and the network, not the substrate — git is a hard dependency (§7). |


## 10. v1 scope

### In

File format · one command surface, refusing on state and never on identity, with the loop SKILL.md teaches frozen at eight verbs (`show` included) · HEAD · `release` (reason mandatory) · IDs and prefix resolution · mandatory declared scope · `blocked_by` and derived blocking · named verifiers as a list, timeout, local execution of proofs · freeze by hash (`done_criteria` at claim, `constraint`/`scope` at ratification) · exit codes · two-phase `context` · sync levels 0 and 1, claims on per-task refs, completion refs and their pruning by `check` · `default_branch` and `accept`'s branch precondition · advisory roles, authority anchored on a signed commit, versioned `allowed_signers` · `status`, `edit`, `graph`, `scope`, the interactive form of `new` and the read form of `log` · `attest`, the one write allowed after `done` · bootstrap skill · `check` (full scope in §4).

### Out of v1, in order of expected value

| Deferred | Reason |
|---|---|
| `--since` (differential context) | Large token saving on long loops, but requires per-agent "seen" state. First candidate for v1.1. **The trigger has been observed, and the row stays deferred.** Two of the three parallel sessions of 2026-08-13 asked for it, both under the name "what moved since I last looked", so the pressure is measured rather than anticipated. What answered them is not this row: the corpus-drift signal of §4 (ADR-47e2ac102f58) says how far this checkout is from the default branch, which is the cheap half of the question — a fact about two revisions, needing nothing remembered — and it is the half whose absence caused a failure rather than friction. The expensive half is what `--since` still means: per-agent seen-state, one cursor per reader, persisted somewhere no revision holds. It is recorded here rather than acted on, because a deferral whose trigger has fired and is not written down is how a document starts disagreeing with the people using it. |
| A CI calling `attest` on its own | The verb itself is in v1 (above). What is deferred is a pipeline appending its run reference without being asked — an integration per provider, not a command. **Still deferred, and `--detached` does not lift it** (§7, ADR-493471d64ba0): that flag removes the obstacle a pipeline used to hit, which was having to produce a commit to record anything, and removing an obstacle is not calling the verb. A repository that wires `ank attest --detached` into the end of its workflow has written an integration; the binary calls nothing on its own, and that is the part the deferral was always protecting. |
| `.ank/` merge driver | The resolution rules are fixed (§7); automating them can wait for the first real conflicts. |
| `touched` inferred from commits | Scope-drift detection. A git dependency, not blocking to get started. **Still deferred**, and rename detection on a dead scope (§11, ADR-97be6d2bd4f2) does not lift it: that walks the history of a path already matching nothing and reports where it went, on a dead scope and on nothing else. Inferring `touched` means reading every commit against every live scope, continuously, to detect drift *before* death. Strictly narrower is not a first instalment. |
| `enforced_by` (mechanisation) | The underlying mechanism against context inflation (see §11). Useless while the ADR corpus is small. |
| `ank serve` (level 2) | Level 1 ships and is enough up to three concurrent agents. |
| `ank review --coherence` (ADR corpus analysis) | Detecting contradictions and duplicates. No value on a small corpus. The ratification queue itself is in v1. |
| Multi-repository federation | A root directory spanning several linked repositories has no answer today, and the corpus recorded that absence nowhere — which reads as "not thought about" when the truth is closer to *locked shut in seven places*: resolution returns the first `.ank/` walking up and `Repo` carries one root, with no collection type and no primary (§6); exactly one repository is resolved before dispatch, and a verb resolving its own is a written design rule broken; `--repo` is single-valued, and §4 fixes the global flag count at three while arguing each one is a memorisation cost; `config.yml` rejects unknown fields, so a `repos:` key cannot even be added speculatively; ids carry no namespace and the displayed short prefix is computed per corpus, so `TASK-8ebd` can mean two things in two repositories; `blocked_by` must resolve in the local store, and a scope naming an absolute path or climbing above the root has no answer to give; and `index.db` and `refs/ank/*` are per repository, so a cross-repository claim has no home. The shape retained, if it is ever pursued: **one `.ank/` per repository stays authoritative** — a corpus belongs to the code it constrains, and a single corpus hoisted to the root breaks that — with aggregation happening *above* the repositories and never instead of them. Reading across corpora is the useful part; writing across them is not. The remedy while the row stood cost no code: run the verb per repository, `ank --repo ./repoA context`, and accept that there is no aggregated view. **The trigger has fired, and on a case this row does not name** (ADR-a1de673043b4). It was written for dependencies — repeated real work where a task in one repository is blocked by a task in another and the pair is tracked by hand — and what pressed instead is the **shared constraint**, a decision binding two repositories at once, which has no legal expression at all: the scope cannot reach a sibling and the copy is already forbidden, so there is no wrong way to write it either. §7 states the shape, and it is this row's own: one corpus per repository stays authoritative, peers are declared and read, writing never crosses, claims never cross. The sorting the decision rests on is worth keeping beside the list above, because it is what made the shape affordable: of the seven locks, three are help rather than obstacle — the flat directory where the file name is the id, ids generated without coordination, an index believed over nothing — three are surface that a revision moves, and exactly one is semantic, `refs/ank/*` being per repository, which is why claims are the thing that stays local. What remains deferred under this row is what it always was above the read: an aggregated view, and any form of writing across corpora. |
| Read-only web view | To reopen only if non-developers must read the board. |
| Linear/Jira export | Management visibility. Never writing back into Ank. |
| Additional entity types | The common base makes extension trivial later. **Do not anticipate**, and the two kinds §3 now declares are not an exception to that — they are what the rule asks for, a kind added because a case was measured rather than foreseen. The kind registry changes what a kind *costs*, a table entry rather than a revision, so that "should this be a kind?" stops being decided by how expensive the answer is; it never makes an unmeasured kind a good idea. `spec` is the row below, fired. `log` is the trace the corpus already held, which was an entity in everything but name — an instant, an author, a message and something it is about — and stored where no query could reach it (§3). What stays refused is a kind proposed because the base would carry it. |
| Spec sections as routable entities | The pressure is real: during v1 development the agents kept re-reading this document whole — a thousand lines with no routing, so working on exit codes also loads synchronisation and permissions — and the tempting fix is a `spec` kind served by `context` the way constraints are. No, for three reasons. **A specification's authority comes from being one coherent document**, and its sections are not independent the way ADRs are: §4 refuses on state because §8 makes roles advisory, §7 means nothing without §3. Fragmenting it into scoped entities creates drift between the fragments, which is the exact failure one document prevents. **Ank already has the sink for what grows**: §11's model turns a rule born in prose into either a scoped ADR that `context` serves or a mechanised check, once it binds daily work — what stays here is narrative and rationale, read when designing rather than when executing, and narrative does not need routing. **And the row above already refuses to anticipate a kind**; this is that refusal applied to the one kind anybody has actually proposed. The remedy this row offered instead, with no code: when the same section keeps being reopened for one perimeter, distil it into a scoped ADR that summarises it and points back here — the specification stays the authority, the ADR becomes the context vehicle. **That remedy is what fired** (ADR-ce550b0dfa39), and the three reasons above are untouched: what was refused, fragmenting one document into scoped entities, stays refused in the same words. The remedy has **zero instances** across the thirty-five ADRs of the corpus that wrote it — the ten naming this document name it in `scope`, which is the opposite direction, decisions that change the document rather than distil it — and nobody tried because the vehicle cannot carry the load: the longest `constraint` ever written here is 1251 characters against a median near 420, the field is frozen at ratification, orientation serves no rule text at all, and the over-constrained ceiling of half a perimeter's budget is already exceeded by eight tasks. The channel was full before anything was put in it. So **the trigger as worded was unfalsifiable by construction**: "that remedy proving insufficient in practice" cannot be observed where the practice is impossible, and a trigger nobody can trip is a deferral that never expires. §11 does not cover the remainder either, every mechanism there being subtractive and presupposing a rule that could in principle be checked — nothing in it ever puts a description *in*. And the feature was already implemented by hand, in the one place that admits it: this repository's own agent guide, a hundred and twenty lines summarising the ADRs, unscoped, unhashed, loaded on every session whatever the perimeter, and confronted with the decisions it summarises by nothing at all. A shadow implementation of a refused feature is the strongest evidence the refusal was mislocated. What is admitted, in §3, is a **whole-document kind and never a section**: one entity per document, no `constraint` field, named in `context` and never quoted there. |



---

## The document this replaces, and its revision record

Status: working draft, arbitration revision
Last revised: 15 August 2026

## Decisions settled in this revision

Settled relative to v1: orientation and constraints reconciled (§5) · immutability anchored by hash, verifiable without making the CLI a gatekeeper (§3, §8) · nominal execution model, one worktree per agent (§7) · claims on git refs from level 0 onward, one ref per task (§7) · Ank never commits, except `accept` (§12) · return after TTL expiry, and a TTL ceiling (§3) · `verify` becomes a list (§3) · `proof` becomes an append-only list, with `attest` as the one write allowed after `done` (§3, §10) · log format fixed, one timestamped line per entry and no part of the task file (§3) · index lifecycle fixed (§6) · verifier timeout fixed (§4) · `--reason` mandatory on `release` (§4) · `check` signals extended (§4) · identity, default roles and signature verification specified (§8).

Every open point of v1 is settled: GPL-3.0 licence, native Windows in v1, merge driver specified but implemented in v1.1 (§13). `attest` was deferred alongside it and ships in v1: the command was written before a CI ever called it, so what remains deferred is the integration and not the verb (§10).

Additions of revision r: **two deferred rows have fired, and a third record gains a sentence** (§3, §7, §10 — ADR-ce550b0dfa39, ADR-a1de673043b4, ADR-25f977377fa0). *Spec sections as routable entities* fires, and not against what it refused: fragmenting one document into scoped entities that drift apart stays refused, in the same words. What fires is the remedy the row offered instead. Distilling a section into a scoped ADR has **zero instances** across the thirty-five ADRs of the corpus that wrote the refusal, and the vehicle it names cannot carry the load — the longest constraint ever written is 1251 characters, the field is frozen at ratification, it is invisible during orientation, and the over-constrained ceiling is already exceeded by eight tasks. So the trigger as worded, that remedy proving insufficient in practice, was unfalsifiable by construction: nobody can wear out a channel nothing fits through. What is admitted in its place is a **whole-document kind and never a section** · *Multi-repository federation* fires on a case its trigger does not describe. That trigger is written for dependencies — a task in one repository blocked by a task in another — and what pressed is the **shared constraint**, a decision binding a backend and a frontend at once, which today has no legal expression at all: a scope cannot reach a sibling, since an absolute path and a climb above the root are both refused and a glob that named one would be confronted with a path that exists on one laptop and in no CI, which is the label scope exists to avoid; and the copy is already forbidden (ADR-e3cb36646d77). Neither scope nor copy is the hole. The shape the row retained is kept entire and stated as a decision in §7 — one corpus per repository stays authoritative, aggregation above them and never instead · *The log*: ADR-ff294eff4d1a is **not superseded and was not wrong**. It decided between a section of the entity body and a file of its own, and never considered an entity per entry; the record therefore gains that sentence rather than a reversal · **the registry declares the kinds `spec` and `log`** (§3), each with its field table, and neither earns a schema bump, for the reason `via` did not · **four corrections that are pure defect**: three passages still described the log as an append-only section of the entity file — the settled list above, §12's recovery argument, and §13's merge-driver rules, which still listed a rule §7 removed — and one sentence claimed git's own union resolves an appended file with no driver, which is false, a three-way merge conflicting on two appends unless a repository configures `merge=union` itself. The property that has actually been protecting the corpus is **one file per entity**, which comes from the addressing and holds whatever git does (§7). The dated paragraphs below are left as the record of what each revision argued, the premise revision p recorded among them.

Additions of revision q: **a claim is renewed by working, not by reporting** (§3, §4, §7 — ADR-0bb7ea8991bc) — §3 renewed the lease on `log` alone, and `log` is reporting rather than working, so the lease lapsed precisely during the stretch with nothing worth logging; renewal now follows **the holder's verbs against the task it holds**, stated as a rule rather than a list because a list goes stale when a verb is added, and the guard against holding by reading is the `check` signal §4 already carries. **`claim_ttl_default` joins the closed key set** beside `claim_ttl_max`, and not instead of it: the cap stops an agent granting itself a day, the default states what this repository's work looks like, and thirty minutes is shorter than the CI run of the repository that shipped it.

Additions of revision p, **schema 3** (§3, §6, §7, §10 — ADR-c9f9d0d6f05d, ADR-ff294eff4d1a, ADR-3877fef1d662): **entities live in one flat directory**, `.ank/entities/<ID>.md`, since the kind is already in the id prefix which is already in the file name, and a third copy of a fact can only disagree with the first two — the previous per-kind layout is read for one window and never written, and a corpus still in it is a `check` signal naming the command that moves it · **kinds are a registry** declaring name, id prefix, required and optional fields and canonical field order, so a kind costs a table entry rather than a second serializer, a second parser branch and a second directory — strictness does not move, an unknown field inside a known kind is still rejected and an unknown kind is rejected by name · **the log is a file of its own**, `.ank/log/<ID>.md`, append-only, line grammar unchanged: the file carrying the frozen criterion was the file that churned most, git already unions two appends, and a second party now has somewhere to write — a task file changes only on a real transition, and §7 loses one of its two merge rules · **actors are typed** and an entity may carry `verified`, a list of readings, optional everywhere and required nowhere; a malformed actor is a `check` finding and never a parse error, because the corpus is not migrated by a rule it predates · the reader range becomes 1 to 3 and its **lower bound does not move**; the log leaving the body is what earns the bump, since a reader that does not know shows an empty history for a task that has one, silently.

Additions of revision o: **the TTL is a property of the claim, and a renewal reuses it** (§3, §7) — renewal recomputed the default and never read the granted lease, so `--ttl` held for one acquisition and collapsed to thirty minutes at the first `log`, failing at exactly the case the flag exists for; the granted duration joins the claim record, every renewal recomputes from it and re-caps by `claim_ttl_max`, and §7's description of the record is corrected — it omitted the claim timestamp too, which `check` has read all along.

Additions of revision n: **the flat listing says what each verb does** (§4, §9) — it carried the verb and its flag names, which is the shape that says what none of them does, and the description takes their place rather than joining them; where a refusal is what distinguishes a verb the description states it, because verbs that refuse on state make a purpose-only line a fourth surface able to misinform, and no description may name a flag the verb does not offer — the rule *a flag that is always refused is not a flag*, applied to the surface a caller reads fastest and checks least. No heading and no grouping: a description says what a verb does, a heading says who it is for, and only the second is the claim ADR-c656cbcc33a9 withdrew.

Additions of revision m: **`--repo` names a repository that exists, and `init` refuses it by name** (§4, §9) — `init` consulted the flag nowhere and initialised the current directory instead, appending the pointer paragraph to an `AGENTS.md` nobody was editing, reporting success, and leaving the named directory empty; the target is positional, `ank init <path>`, and the flag keeps one meaning on every verb rather than a second one on the verb that creates what it requires.

Additions of revision l: **a criterion that turns out unmeasurable has a route back** (§4) — `amend --criteria` is refused only while a live claim freezes the criterion, which is the state test `edit` already applied without either the specification or `amend` saying so, and it leaves `criteria_by` alone, an amend being no claim · in exchange `claim --criteria` sets an absent criterion and never replaces one (§4), closing the door that recorded a creator's correction as the claimer's — the door `amend` bolted shut was standing open on `claim`, and open to the one party the freeze constrains.

Additions of revision k: **`ank help` is one flat listing** (§4, §9, ADR-c656cbcc33a9, superseding ADR-9ede1ffd04e2) — revision i dissolved the split between an agent surface and a human one and left a layered `help` behind it, whose headings were still named after callers; layering is grouping, and a grouping printed by the binary is a claim about who a verb is for. The order of §4 carries the same information without asserting a category, and the loop stays where it is enforced, in the frozen content of SKILL.md.

Additions of revision j: **the ratification signature is verified and not merely read** (§8, §4) — `check` was comparing the anchor in the ratification commit against the file without asking who signed the commit, so an ordinary unsigned commit whose subject read `ratify <id>` was accepted as a ratification; the four outcomes are specified, an unchecked signature is a signal and never a success, and the layout of `allowed_signers` is documented along with the fact that git enforces it only under `gpg.format = ssh`, OpenPGP leaving the match to `check` itself.

Additions of revision i: **one command surface instead of two** (§4, §8, ADR-9ede1ffd04e2, superseding ADR-3859eb46bdc3) — every verb is available to every caller, the CLI refuses on state and never on identity, and the eight-verb freeze is restated as a freeze on the content of SKILL.md, which is where the token budget it actually protected is spent · the refusals are tabulated against their exit codes, and the signed ratification commit is named as the single hard line of authority, a proof requirement rather than a role check (§4) · `status`, `edit`, `graph` and `scope` enter the surface, along with the interactive form of `new` and the read form of `log` (§4) · roles in `config.yml` are named what they always were, advisory, and policy is placed where it holds: SKILL.md, harness hooks, then roles (§8).

Additions of revision h: **the ratification freeze becomes verifiable rather than declared** (§3) — `check` reaches the ratification commit by walking the ADR's own file history, since a commit cannot contain its own identifier and no field could ever name it; `ratified` is documented as what the code always wrote, the hash of `constraint`+`scope`, and an unreachable commit is reported as *cannot verify* and never as a divergence · **`accept` ratifies an ADR accepted by hand** (§3) — an `accepted` ADR carrying no anchor at all can be anchored in place, which is the only way a bootstrap corpus ever acquires the signed commits the whole authority model rests on; one that already carries an anchor stays refused, and that refusal is the half doing the work.

Additions of revision g: **`author` on the common base** (§3) — the identity that ran `new`, optional and immutable, without which two of §4's signals could not be computed at all · **`schema` names a range of versions a tool reads, not a single one** (§3): the frontmatter rejects unknown fields, so a reader limited to its own version would report a newer file as *unknown field* while the file plainly declares the schema that explains it · the agent surface becomes **eight verbs with `show`** (§4, ADR-3859eb46bdc3), and the invitation to read `.ank/` directly is withdrawn (ADR-01b6dd05f0db) · the human `amend` changes `blocked_by` and `scope` on an entity that already exists (§4), the two fields a plan revises without revising a decision.

Additions of revision f: **English is the only language of the project** (ADR-d3a8dcf38817) — specification, CLI output, identifiers and entity bodies alike · the `criteria` field of a proof entry is documented in §3, where it was previously present in the code and in the golden fixtures without appearing in the specification, which the ordering rule of ADR-63b59c5c26f7 forbids.

Additions of revision e: **the project is called `ank`** — the binary, the `.ank/` directory, the `refs/ank/*` ref namespace and the `$ANK_AGENT` identity variable (§6, §7, §8) · **the claim ref is no longer deleted at `done`, it becomes a completion ref** with no TTL, pruned only once the default branch has caught up with durable state: this is what closes the window during which a task finished on an unmerged branch looks free everywhere else (§7) · `claim` refuses a task carrying a completion ref, naming the commit and the branch (§4) · **`accept` requires the default branch**, with no way around it, because a constraint ratified on a feature branch would be a constraint of variable geometry (§4, §12) · `default_branch` enters `config.yml`, with fallback detection through `refs/remotes/origin/HEAD` (§7) · git plumbing extended with `merge-base`, `for-each-ref` and `symbolic-ref`, still plumbing only (§12).

Additions of revision d: **git is a hard dependency**, the file-lock fallback is removed and "degrade, do not fail" now covers only services and the network (§2, §3, §7) · plumbing through the git binary, plumbing only, minimum version 2.34 for SSH signing (§12, §13) · argument parsing written by hand, `gix` and `clap` removed from the justification for Rust (§12) · **canonical form and round-trip made explicit**: byte-for-byte identity guaranteed on canonical form, non-canonical input normalised on first rewrite, CRLF read but never written and reported by `check` (§3) · **a bug found in a `done` task yields a new task**, never a reopening, which would dissolve the proof (§3).

Additions of revision c: exit code 9 (verification environment broken, distinct from a task failure) · context budget given a number, and a mechanical threshold for over-constrained scopes (§5) · hash of applicable constraints recorded in the claim, `done` warns if context changed mid-task (§7) · one proof entry per verifier, an empty `verify` equals an absent one (§4) · `claim` refuses a `closed` blocker, `close` revokes the active claim (§3) · `created` in UTC plus a plausibility signal (§3, §4) · re-acquisition mechanics made explicit (§3) · hoarding signal from empty logs (§4) · chained log hash considered and rejected, with the reason (§3).

Additions of revision b, from external review: `created` field added to the common base (deterministic ordering, and a burst-creation signal) · `closed` status and the human `close --reason` command for ratified abandonment (§3, §4) · hash of the verifier definition anchored in the proof (§4) · task-flooding risk explicitly accepted and argued (§3) · `review` filters by live scopes (§4).
