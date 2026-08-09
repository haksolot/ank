# Ank — Specification v1.1

Status: working draft, arbitration revision
Last revised: 1 August 2026

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

## Decisions settled in this revision

Settled relative to v1: orientation and constraints reconciled (§5) · immutability anchored by hash, verifiable without making the CLI a gatekeeper (§3, §8) · nominal execution model, one worktree per agent (§7) · claims on git refs from level 0 onward, one ref per task (§7) · Ank never commits, except `accept` (§12) · return after TTL expiry, and a TTL ceiling (§3) · `verify` becomes a list (§3) · `proof` becomes an append-only list, with `attest` as the one write allowed after `done` (§3, §10) · log format fixed as an append-only section of the task file (§3) · index lifecycle fixed (§6) · verifier timeout fixed (§4) · `--reason` mandatory on `release` (§4) · `check` signals extended (§4) · identity, default roles and signature verification specified (§8).

Every open point of v1 is settled: GPL-3.0 licence, native Windows in v1, merge driver specified but implemented in v1.1 (§13). `attest` was deferred alongside it and ships in v1: the command was written before a CI ever called it, so what remains deferred is the integration and not the verb (§10).

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

---

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

---

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

---

## 3. Data model

### Two orthogonal planes, not a pyramid

Ank does not model a decision → epic → task chain. Constraints and work are **two independent planes, joined only by scope**.

This is the property that separates Ank from a tracker: an agent receives what constrains it without traversing any hierarchy, and a constraint applies to work that did not exist when it was written. A pyramid would force every task to hang off a parent decision — factually wrong, and expensive to traverse when building context.

### Where precision lives

Ank lets you plan precisely, but the precision is about **the specification of the work**, not its placement on a calendar. Four fields carry it: `scope` (where), `done_criteria` (what proves it is finished), `constraint` (under which rules), `blocked_by` (in which order).

Deliberately absent: estimates, points, declared priorities, cycles, deadlines. Those are instruments for coordinating human teams over time. They do not help an agent work correctly, and they are the slope that leads straight to reimplementing Linear.

### Grouping happens by scope

There is no epic, no milestone, no label. "Everything about the auth migration" is answered by `ank context src/auth/`.

Scope is a better grouping axis than a label for one precise reason: it is **verifiable**. A label is declarative, it drifts, and nobody cleans it up; a glob is confronted with the filesystem. That is also what guarantees a grouping does not silently go stale when the code moves.

### Common base

Every Ank object is a markdown file with YAML frontmatter. Shared fields:

| Field | Role |
|---|---|
| `id` | Canonical identifier, immutable, generated without coordination |
| `type` | `task` \| `adr` |
| `slug` | Cosmetic, never used for resolution |
| `created` | ISO 8601 timestamp of the act of creation, **always in UTC** (`Z` suffix): the ordering of §5 must not depend on a timezone. Immutable. This is what makes task ordering deterministic without depending on git, and what gives `check` a basis for the burst-creation and plausibility signals. |
| `author` | The identity that ran `new`, in the form `$ANK_AGENT` resolves to (§8). Optional, immutable, and written by `new` on every entity it creates. Together with `created` it is what makes two of §4's signals computable at all: an authorless corpus cannot say who created a burst, nor whether a blocker was written by the agent that would benefit from it. |
| `scope` | List of globs. Source of truth for context routing. **Mandatory**: without a scope an entity appears in no `context` and becomes invisible. `new` fails rather than create a silent orphan. |
| `status` | Lifecycle, typed per entity |
| `version` | Integer, incremented on every write. Intra-tree compare-and-swap (§7). |

### Canonical form and round-trip

The format has a **canonical form**: fixed field order, literal blocks for multi-line fields, flow lists for references, UTF-8 without BOM, LF line endings.

The guarantee is exactly this: **`serialize(parse(x))` is byte-for-byte identical to `x` when `x` is in canonical form**. Valid but non-canonical input — another acceptable YAML form, superfluous quotes, CRLF line endings — is read correctly and **normalised on first rewrite**. That is what lets a third-party tool or a human hand write a file without knowing the canonical form, without making that form an authoritative variant, and it is what guarantees that a command which reads then rewrites never produces a spurious diff (§12).

CRLF line endings are therefore **read, never written**. The parser accepts them; `check` reports the entity as non-canonical form — a finding, not a fatal error — with a diagnostic that **names the line endings** and gives the fix command. Never a syntax message that sends the reader to the wrong place: a `---\r\n` diagnosed as "missing frontmatter" costs the user an hour down the wrong path. `ank init` writes a `.gitattributes` (`.ank/** text eol=lf`): on Windows `core.autocrlf=true` is the default, and without this git would convert back on checkout whatever the tool has just normalised, on every clone.

### Identifier allocation

Borrowed directly from git, with one adaptation.

Git derives the ID from content, which presupposes immutability. A task mutates (status, log, title): an ID derived from content would change on every edit and break every reference already written. Ank therefore hashes **the act of creation** — timestamp, agent identity, initial title, randomness — which is immutable. The ID is stable for life and generated without coordination, which is indispensable offline-first.

- **12 hexadecimal characters** stored. Below 8, birthday collisions arrive within the first thousand entities.
- **Short prefixes accepted** on input and shown on output (`TASK-8f3a`).
- **Ambiguity is an error.** A prefix matching two entities fails with the list of candidates. The tool never guesses.

### Task

`.ank/tasks/TASK-8f3a91c2d4e7.md`

```yaml
---
id: TASK-8f3a91c2d4e7
type: task
slug: migrate-auth-sessions
title: Migrate auth to opaque sessions
created: 2026-07-25T09:14:00Z
author: claude-code@host-3   # the identity that ran `new`. Optional: a corpus predates it.
status: in_progress          # open | in_progress | done | closed   (blocked is derived)
scope:
  - src/auth/**
  - src/middleware/session.ts
blocked_by: [TASK-51c2a7f0]  # DAG. Empty = ready to be claimed.
done_criteria: |             # required to claim, frozen by hash afterwards
  Auth integration tests pass, and no reference to
  jwt.verify remains in src/auth/
criteria_by: creator         # creator | claimer — set by the tool, a signal for check
verify: [auth-tests, no-jwt] # list of verifiers from config.yml, all must pass
proof:                       # append-only list, required for done
  - type: test               # test | commit | human-review | assertion
    ref: local/9c1f4a@a3f9c21
    tree: scope/4be2d10c     # hash of the scope files' content at execution time
    criteria: 7d1e2a90b4c3   # hash of the frozen done_criteria, recorded by done
    verifier: auth-tests@1f2e3d4c   # hash of the definition that ran (§4)
  - type: test
    ref: local/e51b22@a3f9c21
    tree: scope/4be2d10c
    verifier: no-jwt@9ab0c1d2
schema: 2
version: 7
---

Free-form context, notes, links.

## Log
- 2026-07-26T14:02Z claude-code@host-3 — jwt.verify removed from session.ts
- 2026-07-26T14:31Z claude-code@host-3 — released: needs access to the staging Redis store
```

**The log format is fixed** (formerly open point 3): a `## Log` section at the end of the task file, append-only, one timestamped line per entry. Appending at the end produces a one-line git diff, which preserves the recovery property (§12). A separate file would have given equivalent diffs while doubling the number of objects to resolve; a single file keeps "the format is the specification" simple for third-party tools. `log` writes here and increments `version`. The log is a **work trace, not proof**: nothing authoritative is anchored in it — freezes and proofs have their own hash anchors — and a rewritten past entry is a git diff visible in review like any other falsification of history. That is why a chained log hash, considered, was rejected: it would weigh the format down to defend a surface that carries no authority.

**The claim is not in the file.** It lives exclusively in the ephemeral coordination plane (§7). Recording it here would produce a git diff on every task pickup, which is precisely what separating the two planes exists to avoid. A task that is `in_progress` with no active claim and no completion ref is simply a task whose TTL expired: it can be picked up again, and its log says where the previous holder stopped.

**`schema`** carries the format version, with explicit migration and never a silent break. It is the counterpart of the promise "the format is the specification": a third-party tool must be able to cleanly refuse a file it does not know how to read.

A tool therefore declares a **range of versions it reads**, not a single one, and refuses anything outside it naming the version rather than the symptom. The distinction is not academic, and adding `author` is what made it concrete. The frontmatter rejects unknown fields — that is what lets a typo like `priorty:` be an error instead of a silent loss — so a tool that read only its own version would report a file one version newer as *unknown field `author`*, while the file plainly declares the schema that explains it. The reader would go looking for a typo. Refusing on the version says the one true thing: this file is newer than this tool.

The rule is asymmetric on purpose. Reading **older** versions is a promise the format keeps: a corpus is never migrated by a tool that refuses to read it, so every field introduced after version 1 is optional at parse time, and its absence means "written before this existed" rather than "invalid". Reading **newer** versions is refused, because the fields a tool does not know about are exactly the ones it would silently drop on the next rewrite.

**Lifecycle.** `open` → `in_progress` (via `claim`) → `done` (via `done`, proof mandatory). There is no separate `claimed` status: a successful claim puts the task directly into work. **`claim` on an `in_progress` task with no active claim is a legal transition** — that is pickup after expiry, not an anomaly. The single exception is a task carrying a **completion ref**: it was finished on another branch, and `claim` refuses with code 4, naming the commit and the branch (§7). That is precisely the case the file's status cannot express, since a `done` lives in the durable state of the branch that produced it and exists nowhere else before the merge. After `done`, the only legal write is **appending** a proof to the `proof` list; any other modification is reported by `check`.

**A bug discovered in a `done` task yields a new task.** Never a reopening, never a re-edit of the code under cover of the finished task. This case is permanent, not exceptional: a task is finished when its criterion is proven, not when its code is perfect, and proving a criterion never meant nothing was left to fix. Reopening would dissolve the proof — a `proof` entry anchors a state of the tree at a point in time, and that content would change underneath it with nothing to signal the fact, which is exactly the falsification anchoring exists to make visible. The new task carries its own scope and its own criterion, and cites the one whose work it corrects. First example in this repository: `TASK-dc87e0ecfb6c` corrects the lock-retry strategy written by `TASK-244a842bc0cc`, which stays `done` with its proof intact.

**`closed` is ratified abandonment.** Terminal, reachable from `open` and `in_progress` through `ank close <id> --reason <r>`, with the reason mandatory and going into the log. The verb is outside the loop SKILL.md teaches (§4) rather than closed to agents: what makes the abandonment ratified is the reason left in the record, not the identity of whoever typed it. It is the answer to an active corpus ageing: the dead scopes and orphan tasks `check` detects must be closable explicitly, never automatically, and never by deleting the file (deleting would break other tasks' `blocked_by` references, whereas `closed` preserves them). **`closed` does not unblock**: a closed task was not done, so its dependents stay blocked, and `check` reports "blocked by a closed task" so a human decides — close down the chain, or rewrite the dependency. Two operational points: `claim` refuses a task one of whose `blocked_by` is `closed` (code 7, naming the closed blocker, as for any active blocker), and `close` on an `in_progress` task revokes the active claim in the same operation — the ref is deleted, and the holding agent learns this at its next `log` (code 6, the task is no longer in work).

**`blocked` is not a status, it is a derived property.** A task is blocked if and only if it has at least one unfinished `blocked_by`. Nothing is entered by hand, so nothing can go stale. `claim` refuses a blocked task and names the blocker.

**`blocked_by` is the only relation between tasks.** A DAG, not a tree: no parent/child, no rollup, no cascade. Three reasons, in order of importance.

*Rollup is completion without proof.* "The parent is finished when the children are finished" is structurally the same hole as `assertion:`, hidden in the topology instead of written in a field. With a DAG, when the blockers are finished the task is **unblocked, not finished**: it still goes through its own `done` with its own proof. The parent verifies the whole, not the sum of the parts — and that is exactly where integration regressions live.

*Decomposition is discovered, not planned.* A human breaks an epic down from the top. An agent discovers mid-course that its task requires another. That is ordering, not containment: forcing a tree would require deciding on parentage at the moment when only an order is known.

*The same work often serves two tasks.* "Add the Redis adapter" blocks both the auth migration and the session cleanup. A tree forbids that; a DAG represents it without duplication.

Cycles are refused on write and reported by `check`.

**Blocking defers the obligation, it does not release it.** An agent may create blockers after claiming, which is the legitimate case of discovered subtasks — but it is also a possible escape hatch for an agent in difficulty. The guardrail is not to forbid the act but to remove its payoff: the `done_criteria` stays frozen, the task stays to be finished, and creating a blocker costs the price of a real task (scope and verifiable criterion mandatory), so a fake blocker is visible to the naked eye. `check` reports the pattern "blockers created by the same agent after claiming" as a signal, not a fault. Faking must cost more than doing.

**`done_criteria` is required in order to `claim`, and its freeze is anchored by hash.** The freeze cannot happen later: a task created without a criterion would then become permanently uncriteriable once claimed. `claim` therefore fails if the field is empty, with the exact command to set it and claim in the same call.

The freeze mechanism accounts for the CLI not being a gatekeeper: any tool can rewrite the file. The freeze is therefore **verifiable, not defended**: `claim` records the hash of the `done_criteria` in the claim record (§7), `done` checks that the current criterion matches that hash before executing anything, and writes the hash into the proof. A criterion modified between claim and done makes `done` fail with code 6, and `check` reports the case. Editing the file unblocks nothing.

The freeze prevents weakening a criterion *after the fact*; it does not prevent setting a lenient criterion *at claim time*. That is why the `criteria_by` field records who set the criterion: a task created by a human carries its criterion from creation (`creator`), and `check` reports "criterion set by the claimer" as a signal — the same logic as self-created blockers, visible without being forbidden.

**Claim TTL.** Short, 30 minutes by default, **capped by `claim_ttl_max` in `config.yml`** (2 hours by default) — an agent cannot grant itself 24 hours and hoard. It is **renewed implicitly by `log`**: working is enough to keep the lock, there is no `heartbeat` verb to memorise.

**Return after expiry.** A 40-minute build with no `log` expires the claim; that is normal, not a fault. On expiry the task stays `in_progress` and becomes claimable again. When the original holder returns: if nobody took the task over, `log` and `done` **re-acquire silently** and carry on; if another agent took it over in the meantime, they fail with code 4 and the name of the new holder. Mechanically, "silently" means checking that no active claim exists for the task — the ref `refs/ank/claims/<id>` — then recreating it in the current agent's name, both steps resting on the atomic primitive of a ref update. No data is lost either way — the log says where each holder stopped.

### ADR

`.ank/adr/ADR-3c7e0b9142af.md`

```yaml
---
id: ADR-3c7e0b9142af
type: adr
slug: opaque-sessions
title: Opaque sessions rather than stateless JWT
created: 2026-07-18T11:02:00Z
author: marie@laptop         # the identity that ran `new`
status: accepted             # proposed | accepted | superseded
scope:
  - src/auth/**
constraint: |                # the only field injected into context
  Do not introduce self-contained JWTs for user auth.
  Every session goes through the Redis store.
see: src/auth/session_store.ts    # optional, for positive constraints
supersedes: ADR-9a12ff03b8e1
ratified: 4c1e9a20            # hash of constraint+scope at acceptance (set by accept)
version: 2
---

Decision, alternatives rejected, consequences.
```

**`constraint` is short and imperative.** It alone goes into the agent's context, never the body of the document. A three-page ADR therefore costs about thirty tokens at injection time.

**`see`** answers the fact that a negative constraint ("do not do X") is respected without context, whereas a positive one ("everything goes through adapter X") needs a pointer to the reference code.

**Immutability, anchored like that of tasks.** An `accepted` ADR has `constraint`, `scope` and `status` locked; the body stays editable. The lock is verifiable: the signed ratification commit (§8) records the hash of `constraint` and `scope` at acceptance time, and `check` compares the current state against that hash. An ADR whose constraint has diverged from the ratified hash is reported as **altered** — and its injection into `context` is suspended with an explicit warning, because injecting an altered constraint would amount to letting the editor rewrite the rule.

**How `check` reaches the commit.** `ratified` holds the hash, not the commit — a commit cannot contain its own identifier, so a field naming it could never be written by the single commit `accept` makes (§12). The pointer is the file's own history instead: walking back from `HEAD` over the ADR's path, through `rev-list`, the first commit whose message is the `ratify <id>` of §12 is the ratification, and `cat-file` reads the `constraint+scope` hash out of that message. Comparing against *that* hash is what makes the freeze verifiable at all: the copy in the file is written by whoever writes the file, whereas replacing the one in the commit means producing another signed commit, which is what a ratification key is for.

Three outcomes, and the third must not be confused with the second. The hashes agree, and the constraint is the ratified one. They disagree, and the ADR is **altered**. Or no ratification commit is reachable — a shallow clone, a rewritten history, a corpus moved between repositories — and the honest report is that the freeze **cannot be verified**, never that it was broken. A check that cries divergence over a shallow clone is a check people learn to ignore.

Modifying a decision means creating a new ADR that `supersedes` it. **The `accepted` → `superseded` transition is the only legal write on an accepted ADR**, and it is performed by `accept` of the new ADR, inside the same ratification commit — the replacement and its authorisation are inseparable.

**Ratifying what was accepted by hand.** A corpus holds ADRs that are `accepted` and carry no `ratified` anchor: written before the tool existed, or promoted by editing the file. `check` reports them as a signal and not a violation (§4), because condemning a bootstrap corpus wholesale would block every `done` behind it. `accept` therefore admits one exception to "`proposed` → `accepted` is the only promotion": an `accepted` ADR **carrying no anchor at all** is ratified in place, which writes the anchor and produces the signed commit without changing the status. An ADR that already carries one is refused, and that refusal is the half doing the work — re-anchoring is precisely how an edited constraint would be laundered, and changing a ratified decision stays a succession. Supplying a *first* anchor launders nothing: there was no anchor to diverge from.

The succession such an ADR declares follows the same reasoning. A `supersedes` whose target is already marked `superseded`, with no other accepted ADR claiming that target, is a succession already on record — bootstrap again, or an `accept` interrupted between its two writes — and ratification records it in the commit without rewriting the file. A target still `proposed` was never binding, and remains a refusal.

**Ratification.** An agent creates in `proposed`. Promotion to `accepted` goes through `accept` and through nothing else, and `accept` produces the signed ratification commit (§8, §12): the authority is carried by the signature, verifiable by anyone against `allowed_signers`, not by the identity string of the caller. A `proposed` ADR is visible in orientation mode, **never injected in execution mode**: non-binding means it must not consume the attention budget of an agent that is writing code.

The underlying principle, which explains the asymmetry with tasks: **ratification applies where an artifact commits others, not where it records work.** An ADR constrains every agent that comes after it; a task commits nobody. Hence `new task` without restriction and `new adr` in `proposed`.

The symmetric risk — an agent going off the rails and flooding the repository with tasks — is **accepted, without a quota**. A quota would be unenforceable in this design: the format is the specification, an agent writes files directly, and there is no central arbiter offline-first to do the counting. The defence is visibility, not restriction: every task costs a valid scope, `check` reports burst creation by a single identity (through `created`), and `review` presents creations by author. Flooding is a noisy diff in review, not a silent state.

---

## 4. CLI surface

**Ank exposes one surface** (ADR-e17e1bbd93ff, superseding ADR-c656cbcc33a9). Every verb is available to every caller, and the CLI refuses on state, never on identity. Git has more than a hundred commands, every one of them reachable by anybody who types it, and stays learnable because of what a newcomer is taught first — not because the porcelain sorts callers into classes. Ank borrows that shape.

The memorisation budget is real; it is simply not the binary's job. It is spent on documentation, and it is enforced where it operates: what an agent is taught is the loop and the planning that fills it, and the content of what teaches it is frozen. What a human may type is everything.

### What the skill teaches, and the freeze on SKILL.md

```
Loop:        context → claim → show → log → done
Off-loop:    new, find, release
Planning:    new adr, amend, review, graph, check, find --status open
```

**This is the entire content of SKILL.md, and that content is frozen** (ADR-e17e1bbd93ff). SKILL.md is loaded permanently (§9), so what it teaches is what costs tokens on every call, for every agent, including the ones that only loop; growing it costs an ADR superseding ADR-e17e1bbd93ff, exactly as growing a verb list once did. The freeze constrains the documentation, not the dispatch table: an agent that runs `ank scope` gets the answer; it was simply never told the verb existed, and being untold is not being refused.

**Two modes, because an agent that can only execute is half an agent.** The loop is how work gets done; planning is how the work that gets done comes to exist — deciding which tasks should exist, in what order, under which constraints. ADR-c656cbcc33a9 taught the loop alone, and an agent taught by it could not propose a decision, correct a graph, or notice that the corpus had gone incoherent: `ank new adr` was never mentioned, so an agent seeing an architectural problem had no documented path from the observation to a recorded ADR. Planning is the highest-leverage activity in the corpus, and a badly shaped backlog wastes more downstream than the teaching costs upstream. The ceiling moves with the content — **at most 140 lines and 1200 words**, up from 80 and 700 — and stays a ceiling to notice drift, not a target to fill.

**`accept` is described and never invited.** The skill says what it is — a human act, signed, on the default branch only — so that a planning agent knows where its own authority ends, and it never shows the command as something to run. That is the one hard authority line in the system (§8, §12), and describing it is what makes it legible rather than mysterious. `close` and `attest` stay outside for the ordinary reason: nothing has decided they are worth what every session pays for them.

That distinction is the whole revision. ADR-3859eb46bdc3 froze an *agent surface* at these same eight verbs and sent everything else to a human side, but the split it protected was never a boundary — the CLI told callers apart by `$ANK_AGENT`, a variable the caller sets itself (§8). A wall whose bricks are self-declared identity is a sign, not a wall. What the freeze actually protected was the token budget, and that protection moves here, where it holds without pretending to be a check.

`show` is in the loop by that route, and the reasoning is worth keeping because it is the argument any addition to SKILL.md has to beat. It sat outside at first, as the only unbounded reader in the system. That argument was about output size, and §5 answers size with a budget and a truncation notice that says what it cut. What it did not answer is the body: `context` serves the criterion and the constraints, never the prose that justifies them, and the prose is where the reasoning an agent is supposed to inherit actually lives. With `.ank/` closed to direct reads (ADR-01b6dd05f0db), withholding it entirely was the worse trade.

### The rest of the surface

These verbs are the rest of the surface. Four of them — `review`, `check`, `amend` and `graph` — are what the planning mode above teaches, and the others are outside what the skill teaches rather than withheld from anyone: none of the refusals below consults who is calling, and an agent that types one gets its answer.

```
review    ratification queue, pending proposals, corpus health
accept    promotes a proposed ADR to accepted (produces the signed commit, §8, §12;
          requires the default branch)
check     mechanical invariants, exit code usable in CI
close     closes a task that will never be done (--reason mandatory)
amend     changes `blocked_by` and `scope` on an entity that already exists
attest    appends a proof to a finished task: the one write §3 allows after `done`
status    where am I: branch, claim, perimeter, queue, findings
edit      opens an entity in the editor and validates what comes back
graph     the `blocked_by` DAG in readable text
scope     what covers a path
```

`review` and `accept` are not comfort features: **the entire authority model of ADRs depends on them**. Without them an agent creates in `proposed` and nothing ever becomes binding.

`review` filters by default on **live scopes**; entities with a dead scope are grouped into a cleanup section with `close` suggested. An ageing corpus therefore produces an explicit closure queue rather than diffuse noise.

**`accept` only runs on the default branch** (§7), and there is no way around it — no `--force`. An ADR ratified on a feature branch would be binding only on that branch: a constraint of variable geometry, and a ratification hash that depends on the reading context, which is exactly the ambiguity Ank eliminates everywhere else. The legitimate use case — introducing a constraint and the code that applies it in the same PR — is already covered by the model: the ADR is created in `proposed`, travels with the code, and is accepted only once it has reached the default branch. An unratified constraint binds nobody, so nothing is lost by waiting, and that is what makes the prohibition sustainable while strict. Off the default branch, `accept` exits with **code 7** — missing prerequisite, not illegal transition: the promotion is legal, the place is not.

```
$ ank accept 19d0
error[7]: accept requires the default branch (current: feat/opaque-sessions, default: main)
  -> git switch main && ank accept 19d0
```

**`amend` covers the two fields a plan actually changes on**, `blocked_by` and `scope`, and covers nothing else. A subtask discovered after a task was filed is a `blocked_by` added to something that already exists; a scope found to omit the files the work must touch is a scope corrected before the task can be claimed at all. Both are ordinary, both were done by hand until this verb existed, and an edit done by hand is indistinguishable in the resulting file from any other edit done by hand — which is the argument that put `attest` on this surface too.

It **adds and removes explicitly** and never takes a replacement list: a verb given the whole list silently drops whatever the caller forgot to repeat. It refuses a `done` or `closed` task, since §3 allows exactly one write after completion and that write is `attest`'s. It refuses the `scope` of an accepted ADR, whose `scope` is hashed into the ratification commit (§8) — changing a ratified decision is a succession, and succession has its own verb. And it refuses `done_criteria` by name rather than by omission, because the flag a caller reaches for deserves the command that actually applies: `release --reason`.

Amending the `scope` of a task under a live claim is **allowed and warned about**. The claim record anchors the hash of the constraints that scope selects (§7), so the change moves what binds the work in progress. Refusing would be wrong — a scope discovered false mid-task is exactly the situation the verb exists for — and allowing it silently would be worse.

Everything else (editing fields, reordering, deleting) goes through **`ank edit`**, which is the paved road rather than a gate: it opens the same file in the same editor and validates what comes back. Below it, the direct edit remains possible, since the format is the specification and the CLI is not a gatekeeper. The actor matters there and is not decoration: ADR-01b6dd05f0db closes `.ank/` to direct reads and writes *by an agent*, and leaves **a human** with an editor every power they had. Written without the subject, that sentence reads as a general permission and hands back what the ADR withdrew.

### Refusals are on state

The binary refuses, and what it refuses on is always a fact about the corpus or the repository, never a fact about the caller.

| Refusal | Code |
|---|---|
| frozen field diverged from its anchoring hash, or illegal transition | 6 |
| claim held by another agent, or task already finished on another branch | 4 |
| task blocked, no `done_criteria`, or `accept` off the default branch | 7 |
| proof missing or invalid | 5 |

That list is the guarantee. A state refusal applies to every caller equally and means the same thing to all of them, which is what makes it worth writing an exit code for; a refusal conditioned on identity would mean whatever the caller declared itself to be. `$ANK_AGENT` names who acted — it goes into the log, into the claim ref and into `check`'s signals — and it is never consulted to decide whether a verb runs.

**The one hard line of authority is the signed ratification commit** (§8, §12), and it holds precisely because it is not a role check: `accept` produces it, `check` verifies it against `allowed_signers`, and an anchor no key covers is reported as unverifiable. That is a proof requirement, the same shape as every other anchor in the system. Everything else that looks like permission is policy, and policy lives above the binary (§8).

### Four verbs and two forms, for the reader at the keyboard

With the surface no longer a boundary, verbs serving human ergonomics enter it without ceremony. None of them introduces state: each one composes what `context`, `find`, `review` and `check` already derive, or wraps a write that was being done by hand anyway.

**`status`** answers *where am I* in one call: the branch, the active claim and its expiry, the constraints on the current perimeter, the ratification queue, completion refs the default branch has not caught up with (§7), and the corpus findings `check` would report. It **degrades with a warning** rather than failing when there is no remote or no determinable default branch — the parts that need neither are still worth printing. Terse `git status` register, and it ends with the next command to run, like every other output here.

**`edit <id>`** opens the entity in the editor named by `$EDITOR`, validates the result on save, and writes it back in canonical form (§3). **A change to a frozen field is refused by naming the command that legally performs it**: `release --reason` for a `done_criteria` frozen at claim, `new adr --supersedes` for the `constraint` of a ratified ADR. An invalid result leaves the entity untouched and says why, so a mistyped frontmatter costs a re-edit and never a corrupt file. This is the argument that put `amend` and `attest` on this surface, generalised: an edit performed by hand is indistinguishable, in the resulting file, from any other edit performed by hand, and chaperoning it through the tool strengthens the invariants instead of relaxing them.

**`graph [<path>]`** prints the `blocked_by` DAG in readable text, restricted by an optional path the way `context` is, with `--json` for the raw edges. It **names the perimeter it drew** and says so explicitly when that perimeter holds no task. The ordering of §5 already walks these edges to count what a task unblocks, and this makes the same structure visible to a reader. `show` surfaces the narrow case from the same derivation: on a task it lists what that task **directly unblocks** alongside its blockers, one line each with status. Both are computed from the corpus at read time and stored nowhere — a stored reverse edge is a second copy of `blocked_by` that can disagree with the first.

**`scope <path>`** lists every entity whose declared scope matches the path, grouped by type, one line each with its status, and says so explicitly when nothing matches. The resolution is the **same glob matching `context` uses**, resolved deterministically against the filesystem, so the answer is the one that will actually bind. It is the `check-ignore` of Ank: a dead or over-broad scope is otherwise visible only after the fact, through `check`, and this makes glob resolution observable before an entity is written wrong.

**`new` without its mandatory flags opens a pre-filled template** in `$EDITOR` instead of failing, validates the result, and refuses to write an entity that would not pass validation. The `git commit` pattern: no `-m`, an editor. **The flag form is unchanged and remains the scripted path** — it is what SKILL.md teaches and what an agent uses, and nothing about the interactive form reaches it.

**`log <id>` with no message reads** the task's log section, newest first, and requires no claim. `log` with a message keeps writing and renewing the claim, unchanged (§3). The disambiguation is stated rather than inferred: an argument that resolves to an entity id is a read, anything else is a message, and a message that also resolves to an id is an error naming both readings rather than picking one. This closes the one place the git intuition was betrayed — `git log` reads — without renaming the verb.

**`$EDITOR` unset is an environment failure, not a task failure**: `edit` and the interactive form of `new` exit with **code 9** and name the flag form as the way through, consistent with how `sh` not found is treated below. Nothing falls back to a guessed editor.

### HEAD

Borrowed from git, and probably the highest-yield borrowing. `claim` sets a "current task" pointer per agent. Subsequent commands do without an ID:

```
$ ank claim 8f3a
claimed TASK-8f3a migrate-auth-sessions -> HEAD

$ ank log "jwt.verify removed from session.ts"
$ ank done
```

The agent cannot get the identifier wrong, and every iteration saves a context round trip.

**HEAD is not stored, it is derived**: it is the task on which the current agent holds an active claim. Nothing to synchronise, nothing to clean up, no state that can become inconsistent with the real claim.

This assumes and enforces **one active claim at a time per agent**. That is a useful constraint in itself: an agent must finish or release before moving on, which prevents task hoarding and keeps work in progress readable by others.

The optional ID on `log`, `done` and `release` is therefore always redundant: it exists only for explicitness in scripts, and **must match HEAD**, otherwise error 6. It is never a way to act on somebody else's task.

### Pickup and abandonment

`release` closes a real gap: an agent that finds it cannot do the task otherwise has only TTL expiry, which means thirty minutes of dead work for everyone else.

```
$ ank release --reason "needs access to the staging Redis store"
released TASK-8f3a -> open
```

**`--reason` is mandatory.** `release` is the delegation mechanism between agents: the reason goes into the log, and the next agent to claim the task receives the recent log in its `context`, so it resumes where the previous one stopped instead of starting from scratch. A silent release is exactly the gap this verb exists to close — the tool refuses it, with the full command as an example.

### Commands

```
ank context [<path>]        [--json] [--limit N]
ank claim <id>              [--criteria <c>] [--ttl 30m]
ank show <id>
ank log [<id>] [<message>]  (id alone reads; a message writes)
ank done [<id>]             [--proof <type>:<ref>]
ank release [<id>] --reason <r>
ank new task --title <t> --scope <glob>... [--criteria <c>] [--blocked-by <id>...]
ank new adr  --title <t> --scope <glob>... --constraint <c> [--supersedes <id>]
ank new task|adr            (no flags: pre-filled template in $EDITOR)
ank find <query>            [--type task|adr] [--status ...] [--scope <path>]
ank status
ank review [<path>]
ank accept <id>
ank close <id> --reason <r>
ank amend <id>              [--blocked-by <id>...] [--drop-blocked-by <id>...]
                            [--scope <glob>...] [--drop-scope <glob>...]
ank attest <id> --proof <type>:<ref>
ank edit <id>
ank graph [<path>]
ank scope <path>
ank check [<path>]
ank init [<path>]           (§9)
ank help [<verb>]           (§9)

ank --version               (the build itself: version, commit and skill revision, no verb, no repository)
```

**This block is the whole dispatch table**, and that is a property worth stating rather than assuming: `attest`, `init` and `help` were reachable in the binary while appearing nowhere here, so a reader comparing the two documents could not tell which one was wrong (TASK-5c868c20472f). `init` and `help` are specified in §9 and listed here only so the list is complete; `ank help` prints this order, minus whatever is not yet implemented, which is what ADR-c656cbcc33a9 requires of it.

**`ank context` with no argument** covers the whole repository. It is the first call an agent should make, before it even knows which path it works on — an agent launched on "fix the login bug" does not yet know its perimeter.

**`context` with an active claim is always in execution mode** (§5). A path argument is then ignored, with a warning line: `active claim on TASK-8f3a, execution context (release to explore elsewhere)`. Exploring another perimeter mid-task is precisely what the one-claim-per-agent rule discourages.

`--limit` applies **only to tasks**, never to constraints.

**`find` is subject to the same cap as `context`**, one line per result, and it announces what it cut. A search command without a budget is a context-explosion vector at least as effective as a badly bounded `context`. `--scope <path>` filters by scope match — it is the command the truncation counters point to (§5).

**Writing to `log` requires holding the claim**; reading it requires nothing. It is the task's anchoring register: if anyone can write to it, it stops being a reliable trace of what the holder did — that is a state condition on the claim, not a condition on who is calling, and it is why `log <id>` with no message is a read available to everybody. Someone who wants to annotate without holding the task edits the body of the entity, which is already the normal route for anything that is not a state transition.

Commands that take a perimeter take **paths**, uniformly: `review [<path>]`, `check [<path>]` and `graph [<path>]` share the same semantics as `context`, and `scope <path>` resolves against the same globs.

**A path is normalised before it is matched, and a directory has one meaning however it is typed.** Separators are unified to `/`, repeated and trailing ones collapse, `.` disappears and `..` is resolved lexically — so `docs`, `docs/`, `docs\`, `./docs`, `.\docs\` and `docs/../docs` are one perimeter. A path naming nothing inside the repository, because it is absolute or because it climbs above the root, is **refused with the command to run next**; it is never answered. Case is not folded, deliberately: `DOCS` matches nothing here and matches nothing on Linux, and folding it on Windows alone would give one corpus two meanings depending on the machine reading it.

The rule is not cosmetic, and it is written down because its absence was not obvious. Measured on this repository against `docs/`, which five accepted ADRs bind: `docs` answered five, `docs\` answered **four**, `./docs` answered zero (TASK-df4c39031583). The zeros announce themselves. The four does not — it is the form Windows tab-completion produces, it looks like an answer, and it withholds a binding rule from an agent that has no way to know one is missing. A perimeter resolved two ways is a constraint set that depends on typing.

Global flags, deliberately limited to three: `--json`, `--quiet`, `--repo <path>`. Every global flag is a memorisation cost. `--json` is available on every command without exception: full scriptability is an invariant, not an option.

### Short forms

Every long flag keeps its form. Short forms are an addition and never a replacement (ADR-962c25797569): single-dash, single-letter, and this table is the whole of them.

| Short | Long | Legal on |
|---|---|---|
| `-j` | `--json` | every verb |
| `-q` | `--quiet` | every verb |
| `-r` | `--repo` | every verb |
| `-b` | `--blocked-by` | `new`, `amend` |
| `-c` | `--criteria` | `claim`, `new`, `amend` |
| `-l` | `--limit` | `context` |
| `-p` | `--proof` | `done`, `attest` |
| `-s` | `--status` | `find` |
| `-t` | `--type` | `find` |
| `-v` | `--verify` | `new` |

**The letter is the first letter of the long flag, and one letter has one meaning in every verb.** Where several long flags begin with the same letter, exactly one takes it and the others keep only their long form. That is the whole rule, and it is worth the flags it costs: a `-s` meaning `--status` in `find` and `--scope` in `new` would not be a saving but a silent wrong answer, since `ank find -s open` would filter on a scope named `open` and return nothing at all. A short form is only useful if it can be typed without checking which verb it is being typed at.

The long flags left without one, with the letter that would have been theirs: `--body` and `--drop-blocked-by` (`b`), `--constraint` (`c`), `--reason` (`r`), `--scope`, `--supersedes` and `--drop-scope` (`s`), `--title` and `--ttl` (`t`). The three globals take their letter ahead of everything else, because they are legal on every verb and a letter they did not hold everywhere would be a letter nobody could rely on — that is what leaves `--reason` on its long form, `r` being `--repo`'s.

A short form takes its value both ways, exactly as the long form does: `ank find -s open` and `ank find -s=open`.

**Bundling is refused, and the refusal names the flags to type instead.** `-st` is not `-s -t`, because a parser that accepts bundling has to decide what `-sopen` means, and every answer to that is a guess about the caller's intent:

```
$ ank find bug -st task
error[1]: '-st' bundles short flags
  -> ank find -s <v> -t <v>
```

**A single dash is a flag, which is what makes `--` matter.** It was already the only way to write a positional beginning with a dash; short forms make it reachable rather than theoretical, since `ank log "-1 rebuilt the index"` now names a flag where it used to name a message. An argument that begins with a dash and contains whitespace is refused as what it is — no flag contains a space — and the refusal names the escape:

```
$ ank log "-1 rebuilt the index"
error[1]: '-1 rebuilt the index' is not a flag: it contains a space
  -> ank log -- "-1 rebuilt the index"
```

Values are untouched by any of this. A flag's value is taken verbatim, whichever form the flag was typed in, so `ank release --reason "-x"` and `ank find bug -s=-x` both pass their dash through and only a positional ever needs escaping.

**Verbs are never abbreviated.** `ank cl` is not `ank claim`, and it is `unknown command 'cl'` naming the verbs. A prefix that resolves today stops resolving the day a second verb shares it, which would make a working script break on a release that added a verb.

`ank help <verb>` shows both forms; `ank help` does not, and the flat listing is unchanged (ADR-c656cbcc33a9, §9). The overview buys its token economy by staying short, and the detail is one call away for the caller who wants it.

**`ank --version` is not a fourth global flag**, and the limit above is untouched. The three modify a verb; this one replaces it — there is no `ank check --version`, and asking for the build while also asking for something else is not a question anyone has. It prints the crate version, the commit the binary was built from and the revision of the `skill/SKILL.md` it was built alongside, on one line, and exits 0.

It answers **before the foundation**, like `help` and for a sharper reason: the caller who needs it is the one holding an artifact they cannot identify. A version that required a resolved repository, a git of 2.34 and a readable `config.yml` would fall silent in exactly the situation it exists for. Outside any git repository, in a directory with no `.ank/`, it still prints and still exits 0.

The commit is embedded at build time through `rev-parse`, which §8's plumbing rule already allows, and is `unknown` when the build had no checkout to ask — a source tarball, a vendored dependency. Naming the absence beats inventing a value, and it beats the silence that made TASK-1ea38a17d854 cost an investigation to conclude that the binary in hand predated the feature being measured for.

**What the stamp guarantees, and on which builds.** It names the commit the build script last ran at, and the script reruns whenever the commit moves — including a commit that changes no source file, which is the case that catches a binary quietly left behind. It does not depend on the source having changed. The files a commit moves are located through `rev-parse --git-path` rather than assumed at `.git/`, so a linked worktree and a packed ref are covered rather than hoped for; on a detached HEAD the sha lives in the HEAD file, which is watched on its own account. A release, built from a fresh checkout, is exact by construction. The stamp is not a claim about the working tree: uncommitted edits are invisible to it, and a binary built from a dirty tree names the commit it was based on, not the code it contains (TASK-0b26c8b5bfc5).

**The skill revision, and what it lets a reader do offline.** The third value is `skill <rev>`, where `<rev>` is the short form of the same freeze hash that anchors a frozen `done_criteria` (§3), taken over the body of `skill/SKILL.md` — the same value that file declares under `metadata.revision`, computed at build time from the file rather than typed. An agent has both halves in hand and nothing else: the skill is loaded into its context, the binary is on its `PATH`. Comparing the two strings tells it, with no repository and no network, whether the instructions it is following predate the tool it is holding. That is the check TASK-1ea38a17d854 needed and did not have, and the case that motivated it was measured (TASK-b495234f192c): an installed `SKILL.md` two commits behind a tree that had just withdrawn the invitation to read `.ank/` by hand. The hash covers the body and not the frontmatter, so the revision the frontmatter carries does not change its own input. It is `unknown` on a build with no `skill/` to read, for the same reason and with the same honesty as the commit.

### Presentation

Presentation splits in two, and only one half depends on who is reading (ADR-0c8ab846d262). **Color is a property of the reader**: an escape sequence means nothing to a parser, so it is emitted only when a human is demonstrably at the other end. **Structure is a property of the corpus**: a tree is what the `blocked_by` edges *are*, and it is drawn in the bytes, identically for everyone. `--json` carries neither — it is the machine surface, and it is the one place where both would be wrong.

#### Structure

The alphabet is bounded here, before any code uses it, for the reason the palette and the short-form table are: the grammar is the specification.

| Element | Characters |
|---|---|
| a child with siblings after it | `├── ` |
| the last child | `└── ` |
| the gutter under an expanded node | `│   ` |
| the gutter under a wrapped constraint | `│  ` |
| the task the caller holds, in a listing | `* ` |

`graph` and `show`'s `BLOCKED BY` / `UNBLOCKS` draw the relation; `context` gutters a constraint that wraps, so a multi-line rule reads as one rule; `find` and `scope` mark the row the caller is holding, the way `git branch` marks the current branch.

**The prefix of a row is derived from its parent's connector, never from a depth.** A node under `├── ` continues as `│   ` because its parent still has siblings below; a node under `└── ` continues as four spaces because nothing follows. A depth counter cannot tell the two apart, and the difference is exactly what makes a diamond legible.

**Two of these replace indentation instead of adding to it**, and that is load-bearing rather than tidy: the constraint gutter occupies the columns the alignment already spent, and the held marker takes the two leading spaces of a listing row. So the attention budget of §5 measures what it measured before, and truncation does not become a function of the drawing.

Ank already writes non-ASCII to standard output — the em-dash separating a log entry's author from its message — so the alphabet introduces no new encoding question on any platform.

#### Color

Color is emitted when **stdout is a terminal and `NO_COLOR` is unset**, and under no other condition. There is no `--color` flag: the globals stay at three, and a flag able to force color into a pipe is a flag that eventually puts an escape sequence into an agent's context window. Detection is not a preference to be configured, it is an observation about who is reading.

The guarantee bought is negative, and it is the whole point. **No escape sequence ever reaches captured output** — a pipe, a file, a `$(...)`, a CI log. `--json` is never colored even at a terminal: it is the one place both conditions can hold and styling still be wrong, so the rule is stated separately rather than left to follow.

`NO_COLOR` set to any non-empty value disables it, which is what a human at a terminal types to get the raw bytes; `TERM=dumb` disables it too.

The palette is small on purpose, and it is bounded here rather than in the code for the same reason the short-form table is: the grammar is the specification, and the implementation is only its application. Status markers, section headers, identifiers, the error line. Nothing that carries meaning is carried by color alone — every marker still reads as `[open]` or `[done]` in a pipe.

| Element | Style |
|---|---|
| section headers: `CONSTRAINTS`, `PROPOSED`, `TASKS`, `DONE_CRITERIA`, `LOG`, `BLOCKED BY`, `UNBLOCKS` | bold |
| identifiers: `TASK-8ebd`, `ADR-962c` | yellow |
| `[open]` | blue |
| `[in_progress]`, `[claimed:…]` | cyan |
| `[… expired:…]` | yellow |
| `[done]`, `[finished:… on …]`, `[accepted]` | green |
| `[closed]`, `[superseded]` | dim |
| `[proposed]` | magenta |
| a transition word that advanced the corpus: `created`, `claimed`, `logged`, `attested`, `amended`, `accepted` | green |
| a transition word that gave something up: `released`, `closed`, `superseded`, `pruned` | dim |
| the state a transition landed on: `-> done`, `-> open`, `-> closed` | as its marker above |
| `status`'s keys: `branch`, `claim`, `perimeter`, `queue`, `corpus` | dim |
| `error[N]:` and `check`'s `error:` tag | red |
| `warning:` and `check`'s `signal:` tag | yellow |
| a verifier's `ok` / `FAILED` | green / red |
| the trailing next-command line, `> ank …` | bold |
| the `---` fences and the frontmatter keys of the entity `show` prints | dim |
| the `id` and `supersedes` values of that frontmatter | yellow |
| its `status` value | as its marker above |
| a markdown heading in its body | bold |
| the `- <timestamp> <who>` prefix of a log entry, under `show` and under `log` | dim |

**`show` paints the entity and moves nothing.** ADR-01b6dd05f0db returns the
entity byte for byte, and that stays exactly true: an escape sequence occupies
no column, so stripping the escapes from what `show` writes yields the byte
sequence it wrote before, character for character, in the same order. Nothing is
added, removed, aligned or re-indented. The file is not re-laid-out for a human,
because ADR-0c8ab846d262 already refused to give one corpus two shapes — what
changes is which of those bytes are lit, and only for a reader who is at a
terminal. Every reader that parses gets what it always got.

The `status` value is painted from the same table its bracketed marker reads, so
`status: done` at the top of a file and `[done]` in a listing are one fact seen
twice, which is the rule the transition grammar below already states. A log
entry's prefix recedes for the reason `status`'s labels do: the timestamp and
the author are addressing, and the message is what the reader came for. The
prefix is painted the same way under `log`, because the two verbs print the same
line and a line printed twice must not have two shapes.

**Every status carries a color, and every base color is spent.** A task is
`open`, `in_progress`, `done` or `closed`; an ADR is `proposed`, `accepted` or
`superseded`; and the two sets read from one table, because `find` and `scope`
list them side by side and a reader should not have to know which kind a row is
before knowing what its color means. `in_progress` takes the color `[claimed:…]`
carries, because they are one state seen twice — from the index, and from
the ref that claimed it. Nothing is left at the terminal's default, and that is
what makes the table checkable rather than memorable: a status with no color is
now a defect, where before it was an omission somebody had to notice. `blocked`
is absent because it is not a status — it is derived from `blocked_by` at read
time (§3), and no entity is ever stored carrying it.

**A transition reads the same whichever verb produced it.** Every verb that changes state confirms it in one shape — the word for what happened, the identifier it happened to, and where the entity landed — and the color follows that shape rather than the verb. The word carries the direction, the identifier is yellow like every other identifier, and the landing state takes exactly the color its bracketed marker takes in a listing: `-> done` and `[done]` are the same fact seen twice, and a reader who has learned one has learned the other. This is why the rule is stated as a grammar and not as a list of verbs — `attested` is legible to someone who has only ever run `claim`.

`status` is the one output whose lines are label-and-value rather than sentences, so its labels recede and its values do not. The counters keep their own rule: a fault count is red when it is not zero, and the reader is meant to see the number before the word.

The error line follows the same rule applied to its own stream: it is colored when stderr is a terminal **and** the condition above already holds, a conjunction rather than a substitution, so that no arrangement of redirections produces an escape sequence stdout's rule forbids.

**On Windows the terminal must also announce itself**: one of `WT_SESSION`, `TERM`, `TERM_PROGRAM`, `ConEmuANSI` or `ANSICON` present in the environment. Legacy `conhost` does not interpret escape sequences unless the process turns them on, and turning them on means a console API call that this feature does not justify. A console announcing nothing is served plain text — the failure that costs a reader nothing, rather than the one that prints `←[1m` at them.

### Exit codes

The semantics are carried by the code so that the shell can route without parsing output.

| Code | Meaning |
|---|---|
| 0 | ok |
| 1 | generic error |
| 2 | entity not found, or ambiguous prefix |
| 3 | version conflict — re-read and retry |
| 4 | task unavailable — claim held by another agent, or task already finished on another branch (§7) |
| 5 | proof missing or invalid |
| 6 | illegal transition, or frozen field modified (hash diverged) |
| 7 | missing prerequisite — no criterion, task blocked, or `accept` off the default branch |
| 8 | `check`: invariants violated (reserved for `check`, for CI) |
| 9 | environment unavailable — not a task failure |

Codes 3 and 4 are the ones the agentic loop must know how to handle. 3 literally means "redo `context`, somebody moved". 4 means "take something else", and its two causes call for the same reaction, which is the reason for uniting them under a single code: in both cases the task is not to be taken, and the message says which one to take instead. `check` exits 0 when the corpus is healthy and 8 when it has findings — never 1, so that CI can distinguish a sick corpus from a broken tool.

9 covers the environment broadly and not only the verifiers': `sh` not found (§4), git absent or older than 2.34 (§12), default branch indeterminable (§7). What they have in common is that none of them is a failure of the agent's work — it is an environment to repair, and confusing it with a 1 or a 5 would send the agent to fix sound code.

### Self-correcting errors

Never generic help, always the exact command to run next. One well-designed error round trip costs less than three blind attempts.

```
$ ank done
error[5]: proof required to move TASK-8f3a to done
  done_criteria: "Auth integration tests pass, and no reference
                  to jwt.verify remains in src/auth/"
  -> ank done --proof test:<ci-run-ref>
```

```
$ ank claim 8f3a
error[4]: TASK-8f3a held by codex@host-9 (expires in 12m)
  -> ank claim 51c2   (another ready task in this scope)
```

```
$ ank claim 51c2
error[7]: TASK-51c2 has no done_criteria
  -> ank claim 51c2 --criteria "<verifiable criterion>"
```

### Proofs

**Ank runs the verification itself.** This is the central point: if the agent runs the tests and then reports the result through `--proof`, nothing is anchored — it can simply claim it passes. The task declares its verifiers, `ank done` runs them, captures the exit codes and a hash of the outputs. The agent never self-reports.

```
$ ank done
verifying done_criteria hash ... ok
running: auth-tests ... ok (2.4s)
running: no-jwt ... ok (0.1s)
proof recorded: auth-tests -> local/9c1f4a@a3f9c21
proof recorded: no-jwt -> local/e51b22@a3f9c21  (tree:scope/4be2d10c)
```

**Two modes, never ambiguous.** If the task declares a `verify`, `ank done` runs **all** the verifiers in the list — a composite `done_criteria` ("the tests pass *and* no more jwt.verify") is mechanised by several verifiers, not by one that covers only part of it — and `--proof` is refused: the agent cannot short-circuit. Every verifier that runs produces **its own proof entry**, with its output hash and its definition anchored. Without `verify` — field absent or empty list, the two forms are equivalent and the canonical form omits the empty field — `--proof` is mandatory and Ank validates what it can: `commit:` is checked with git, `human-review:` and `assertion:` are recorded as they are and marked unverified.

If a verifier fails or times out, the transition is refused. No dependency on any service: this is usable **inside** the loop, not only at the end.

### Named verifiers

A task's `verify` field references verifiers declared in `config.yml`, never an inline shell command.

```yaml
verifiers:
  auth-tests:
    run: pytest tests/auth/ -q
    timeout: 10m              # default: 10m, a timeout failure is code 5
  no-jwt:
    run: "! grep -rq jwt.verify src/auth/"
```

The reason: a task may arrive through a PR from a fork. An inline command would be arbitrary code execution triggered by `ank done`. Git has exactly this problem with hooks and solved it by never running them on clone. Here, verifiers live in a file controlled by the repository, so modifying one goes through code review like any other change.

**Execution: always `sh -c`, on all three operating systems.** Linux, macOS and Windows are supported natively in v1. On Windows, `sh` is resolved from Git for Windows, which ships it — Ank already requires git, so the dependency is free and verifiers are written once, in POSIX syntax, for the whole team. An `sh` that cannot be found is an explicit error with the installation link, never a silent fallback to `cmd`. **A broken environment is not a task failure**: `sh` not found, command absent (shell code 126/127), inability to spawn the process all exit with code 9 and the exact command that failed — the agent should report or repair the environment, not conclude that its code is wrong. Code 5 stays reserved for a verifier that ran and said no. The **timeout is fixed** (formerly open point 4): 10 minutes by default, overridable per verifier, exceeding it is a code 5 failure with the elapsed time in the message.

`config.yml` remains editable by an agent — replacing a verifier with `true` is the obvious workaround. Two complementary defences, neither resting on good faith. First, **the proof records the hash of the definition that ran** (`verifier: auth-tests@<hash>`, a normalised hash of the `run` + `timeout` entry): what actually ran is anchored in the proof, not in the current state of `config.yml`, so a verifier weakened before or after the `done` — in the same commit or another — is detectable by comparing the proof's hash with the definition at the corresponding commit. Second, `check` reports the patterns: **verifier modified inside the task's activity window** (between the first log entry and the `done`), and **proof hash diverging from the definition in force at the `done` commit**. Splitting the workaround across several commits no longer hides it; it stays a diff in review — faithful to the principle that faking must cost more than doing.

### Trust hierarchy

| Type | What is guaranteed |
|---|---|
| `assertion:"..."` | Nothing. The agent asserts. **Marked weak** in `check`. |
| `test:local/<hash>@<sha>` | Ank executed it, in an environment the agent controls |
| `commit:<sha>` | Verifiable by anyone with `git` |
| `test:ci://<ref>` | Third-party environment, out of the agent's reach |

The dividing line is not local versus hosted, it is **who controls the environment**. Locally, an agent can weaken a test to make it pass — the same class of problem as an ADR edited to unblock oneself.

**What local proof anchors.** An agent's nominal case is an uncommitted working tree: anchoring proof on the HEAD SHA alone would almost always point at a stale state. The proof therefore records three things: the HEAD SHA, a dirty-tree indicator, and **a hash of the scope files' content at execution time** (`tree:scope/<hash>`, git hash-object style). That last one is what actually captures what was tested. `check` additionally reports the case where the task itself modified the test files it invokes.

The levels stack: local proof at `done` time, a CI reference **appended** later to the `proof` list — appending proof is the only legal post-`done` write (§3).

**`ank attest` is in v1, and this settles it.** Earlier revisions deferred it with its shape frozen, on the reasoning that the data structure was ready and the command would come *when a CI called it*. The command was implemented before that happened, and has been used: this repository's own corpus carries `attest`ed CI references. A deferral whose condition has been overtaken is not a plan, it is a document disagreeing with its binary — the state ADR-63b59c5c26f7 orders the work to prevent, and the one a reader could not resolve from §4 and §10 alone (TASK-5c868c20472f).

What that deferral was actually protecting is worth keeping, because it is still deferred and it is not a command: **nothing calls `attest` automatically**. A CI provider appending its own run reference at the end of a pipeline is an integration, not a verb, and it is listed below as such. The verb exists and anyone — an agent, a human, a CI script — can call it today.

**What `check` does instead is notice the omission**, which is the same choice read from the other side. A pipeline that attests itself would assert "a green run existed on a tree containing this task"; the entry carries the frozen criteria hash, so that statement is mechanically true, and it is still not the statement a `test` proof makes. CI proves the tree passes its tests, not that *this* criterion was met — the link between the two exists only because somebody wrote a test encoding the criterion, and no pipeline can know whether they did. Collapsing the two is the rollup hole of §3 one level out. So the assertion stays human, and forgetting to make it is what gets reported (§4).

The `assertion` type exists because "refactor for readability" has no hash to attach. It is allowed but visible as weak, which avoids a `--force` becoming the default path within two weeks.

### Scope of `check`

Summary of the invariants and signals, all mechanical:

- expired claims, `blocked_by` cycles, broken supersede chains, dead scopes (no file matched), over-constrained scopes (§5);
- frozen fields diverging from their anchoring hash — `done_criteria` against the claim, `constraint`/`scope` against the ratification commit, and the signature on that commit against `allowed_signers` (§8): an anchor read from a commit nobody signed anchors nothing;
- weak proofs (`assertion`, unverified), `done` tasks modified beyond appending a proof;
- behavioural signals, reported without being faults: blockers created by the holder after claiming (`author` of the blocker is the current holder and its `created` is later than the claim), criterion set by the claimer, verifier modified inside the task's activity window or proof hash diverging from its definition, scope test files modified by the task that invokes them, burst creation by a single identity (**more than 10 entities by one `author` within an hour**, through `created`), implausible `created` (in the future, or well before the commit that introduces the file — the field is declarative, git is the anchor), repeated claim renewals with no modification to the scope files (possible hoarding; a best-effort signal, since another agent's tree is not observable), constraint accepted after the claim of a task in progress, tasks blocked by a `closed` task;
- a `done` task carrying no `test` proof, once the **default branch** carries it as `done`: the completion rests on a local run and nothing external anchors it. A signal and never a fault — the corpus is intact, the record is thin, and exiting 8 would redden CI on the very merge that introduces the task. The gate on the default branch is what makes it actionable: before the merge there is no run to cite, and reporting there would name work the reader cannot do. The window between the merge landing and its run going green is left to fire, since the statement is true when printed and clears when someone attests; buying that quiet would cost a grace constant, and the constants below are justified for flooding alone;
- entities predating `author`, **reported once for the corpus and never per file**: they are skipped by the two signals above, and saying so once is what keeps that fact visible. One line per file would add a line for every entity written before the field existed — the volume that teaches a reader to stop reading `check`;
- unresolved git conflict markers in `.ank/` files (§7);
- maintenance of the coordination plane (§7): pruning orphan refs, and completion refs whose task is `done` or `closed` on the default branch. `check` is the only command that prunes. A task carrying a completion ref for a long time without the default branch catching up is reported as a signal — that is a branch never merged, not a corpus anomaly, and the answer is human.

**The two signals that need `author`, and why they are signals.** A blocker written by the agent currently holding the task is the shape of an agent building itself an excuse — but it is also the shape of an agent doing exactly what §3 asks, since a discovered subtask *is* a new task with a `blocked_by`. Only a reader knows which, so it is reported and never refused. Burst creation is the same argument at the corpus scale: §3 accepts task flooding without a quota, on the grounds that the defence is visibility rather than restriction, and this is that visibility.

**The numbers are constants, not configuration.** More than 10 entities by one `author` within an hour: a threshold high enough that a session filing the four tasks of a plan passes in silence, low enough that a runaway loop is named within minutes. They live in the tool rather than in `config.yml` because a repository that can raise its own flooding threshold has a flooding threshold that will be raised the first time it fires — and the signal costs nothing to ignore, which is what makes it safe to leave unadjustable.

---

## 5. Attention budget

The single most decisive point for real use. A `context` that explodes on a large repository is a `context` the agent will end up ignoring.

### Two moments, two outputs

`context` serves two situations that used to be treated, wrongly, as one.

**Before claiming — orientation.** The agent does not yet know what to do. Breadth, not depth: the perimeter's active constraints in compact form (id + `constraint` text, never the body), non-binding proposals on one line, and open tasks on one line each. **No `done_criteria`, no log** — it is not writing code yet, and execution detail would be noise. Constraints, on the other hand, are present from orientation onward: choosing a task while knowing the perimeter's rules is exactly what orientation is for.

**After claiming — execution.** HEAD is set. Inversion: no other task at all, but the full `done_criteria`, the constraints matching the scope **of the task**, and the most recent log entries.

Same command, output driven by HEAD. Nothing more for the agent to memorise, and most of the useless context disappears.

```
$ ank context src/auth/

CONSTRAINTS (2 active)
  ADR-3c7e  Do not introduce self-contained JWTs for user auth.
            Every session goes through the Redis store.
  ADR-8b41  Rate limiting mandatory on every public endpoint.

PROPOSED (1, non-binding)
  ADR-19d0  [pi@host-2] Prefer idempotent migrations

TASKS (2)
  TASK-8f3a  [claimed:claude-code@host-3] Migrate auth to opaque sessions
  TASK-51c2  [open] Add secret rotation

> ank claim 51c2 to start
```

### Truncation priority

**In execution mode, a constraint is never truncated.** Cutting a binding constraint means an agent can violate a rule it never saw — a discreet `+12 more` would be the worst possible behaviour. The two-phase design makes the guarantee tenable: after claiming, the perimeter is that of the task alone, so few constraints match. The budget is concrete: `context_budget` in `config.yml`, measured in characters, 8000 by default (roughly 2000 tokens — the character is the only unit measurable without depending on a tokeniser). A scope is **over-constrained** when constraints alone consume more than half of it in execution mode: a mechanical threshold, implementable as stated, and `check` reports it as such — it is a corpus problem, not a display problem.

### Task ordering

An agent facing eight ready tasks must pick one without hesitating and without inventing a criterion. The ordering is therefore **deterministic and derived**, never declared:

1. Number of tasks this one directly unblocks, descending
2. On a tie, the `created` field ascending (deterministic without depending on git)

Tasks on the critical path rise naturally, with no `priority` field to maintain or derive. A human who wants to steer the work does so by creating or by claiming, not by reordering a list.

### End of loop

No ready task in the perimeter is a normal state, not an error. `context` says so explicitly and exits 0:

```
no ready tasks in scope (3 blocked, 1 in progress by codex@host-9)
```

An agent in a loop needs a clean stop signal. An empty output reads as a breakdown and triggers pointless retries.

In orientation mode, where the agent is not writing code yet, truncation is acceptable. The cutting order:

1. Tasks first, before any constraint
2. Constraints with the most **specific** scope are kept — a narrow glob beats `src/**`, it was written for that precise code
3. Constraints whose vocabulary overlaps the titles of the perimeter's tasks
4. The rest as a counter: `+12 broad constraints, ank find --type adr --scope <path>`

---

## 6. Storage and search

```
.ank/
  config.yml
  allowed_signers   # public keys allowed to ratify (§8), versioned
  tasks/TASK-<id>.md
  adr/ADR-<id>.md
  index.db          # derived, disposable, gitignored by init (§9)
```

**Flat tree.** Attachment happens through `scope`, not through location. One entity can constrain several modules; a tree mirroring the code would force a single parent and break at the first refactor.

**Atomic writes** (write-then-rename), under a file lock for the duration of the read-compare-write cycle — that is what makes the `version` compare-and-swap effective, since write-then-rename alone compares nothing.

**Derived SQLite index**, fully rebuildable from the files. It is never the source of truth.

**Index lifecycle, fixed** (formerly open point 1): the index stores a content hash per `.ank/` file. Every command compares the files in the perimeter it touches against those hashes and incrementally reindexes what diverged — the index is therefore always up to date *at read time*, with no daemon and no watcher. `check` reindexes fully. An index that is absent or of an unknown schema is rebuilt silently: deleting it is always a safe operation.

### Three levels of search

1. **Scope resolution** — glob matching, deterministic, zero ambiguity. Covers the bulk of cases; this is what `context` does.
2. **Lexical search** (FTS5) for `find`. Fast, local, explainable.
3. **Semantic** — explicitly out of scope. It would impose an embedding model (loss of agnosticism) and non-deterministic ranking, which reintroduces exactly the uncertainty the tool is trying to eliminate.

---

## 7. Synchronisation

A git-like model: fully functional locally, deployable progressively, never dependent on a service.

### Nominal execution model

The nominal case is **one working tree per agent** — clones or `git worktree` — each on its own branch. That is what local proofs already assume (a `verify` run in a tree other agents are modifying in parallel would prove nothing clean) and it is the effective practice of agent harnesses. Several agents sharing one tree works but is a degraded mode, not a design mode.

### Separation of the two planes

- **Durable state** (tasks, ADRs, log) — replicated, versioned, offline-first. Pure git.
- **Ephemeral coordination** (claims) — short TTL, never historised, disposable. The only thing requiring an arbiter.

Claims live in dedicated git refs, **one ref per task**: `refs/ank/claims/<task-id>`. A single ref for all claims would put unrelated claims in contention — two agents taking two different tasks would fight over the same non-fast-forward push. Per task, git's CAS arbitrates exactly the conflict that matters and no other. These refs are never merged into the working branch: no noise in diffs, no conflicts on files.

A claim record carries: identity, expiry timestamp, hash of the frozen `done_criteria` (§3), and a hash of the set of constraints applicable to the task's scope at claim time. That last one closes the long-work window: a constraint accepted while the agent works changes that hash, and `done` **warns** — never blocks, since a new constraint does not necessarily concern work already done — inviting a re-read of `ank context`; `check` reports the same case. The full lifecycle of the ref is described just below. `ank init` adds the fetch refspec `refs/ank/*` to the repository config: hosts do not fetch non-standard refs on their own.

Expiry is evaluated on the timestamp the claim carries, with a 2-minute clock-drift tolerance: at the scale of a 30-minute TTL, NTP is more than enough.

### Ref lifecycle: pickup, completion, pruning

A task's ref has **two states**, and the record it points to says which: `claim` (a holder, an expiry) or `completed` (a commit, a branch, an identity, the `done` timestamp). One ref per task in both cases, at the same address `refs/ank/claims/<task-id>`: two distinct namespaces would let a stale claim and a completion coexist for the same task — exactly the ambiguity the ref exists to settle — and would split git's compare-and-swap across two addresses where there is only one conflict.

- **`claim`** writes a `claim` record. The primitive is the atomic ref update: two concurrent agents, one winner, code 4 for the other.
- **`release`, `close`, pickup after expiry** replace or delete the `claim` record, unchanged.
- **`done` does not delete the ref, it transforms it** into a `completed` record, **with no TTL**.

That last point closes a real window. `status: done` lives in durable state, therefore on the agent's branch alone until the merge: between the end of the work and the integration of the PR — that is, most of a branch's life — the task would look free everywhere else, and another agent would run `context`, read `open`, and redo work already done. The fix stays inside the ephemeral coordination plane, touching nothing in durable state.

`claim` on a task carrying a `completed` record **refuses with code 4**, naming the commit and the branch:

```
$ ank claim 8f3a
error[4]: TASK-8f3a finished on another branch (commit abc1234, branch feat/opaque-sessions), not merged here yet
  -> ank claim 51c2   (another ready task in this scope)
```

**A blocker carrying a `completed` record gets the same answer, with code 7.** The refusal itself is unchanged — `blocked_by` names work whose result this branch does not have, and claiming on top of it is the risk the ref exists to make visible — but the bare message hides the one fact that decides what to do next. `claim` therefore consults the ref of every active blocker, and a blocker finished elsewhere is named ahead of the first one in the list, wherever it sits there: the order of `blocked_by` says nothing about which blocker matters, and being told about the merged-nowhere one is the point.

```
$ ank claim 51c2
error[7]: TASK-51c2 is blocked by TASK-8f3a, finished on feat/opaque-sessions (commit abc1234), not merged here yet
  -> ank context
```

The hint follows the same rule as everywhere else and never names a command that would refuse on the spot: the task offered as another thing to take is one whose ref is free, not one whose file reads `open` on this branch only because a completion has not landed yet.

**Pruning is conditioned on durable state, not on a TTL.** A completion ref is pruned once the task appears `done` or `closed` **on the default branch**: the information the ref carried is then present where everyone reads it, and the ref has no further use. That is what makes "ephemeral" accurate rather than decorative — the ref lives exactly as long as it is useful, three minutes or three weeks depending on how long review takes.

The predicate is about the task file as it appears on the default branch (`git cat-file -p <default_branch>:.ank/tasks/TASK-<id>.md`), **and not about the reachability of the recorded commit**. The difference is not theoretical: `done` writes only to the working tree (§12), so the commit it records is the current HEAD, which is frequently already an ancestor of the default branch — an agent that branches and then runs `done` before its first commit would see its ref pruned immediately, which would reopen the very window the mechanism exists to close. The recorded commit serves the diagnostic above, never the pruning decision.

`check` prunes, at the same time as orphan refs — "never historised" is a maintenance operation, not a free property. `claim` and `context` never prune: a reader does not sanitise the coordination plane underneath everyone else.

### Default branch

`default_branch` in `config.yml` names the branch that carries the reference durable state. Failing that, it is detected from `refs/remotes/origin/HEAD`. If neither answers, there is no answer to invent — assuming `main` would be exactly the guess the tool refuses everywhere else:

```
$ ank accept 19d0
error[9]: default branch indeterminable (default_branch absent from .ank/config.yml, refs/remotes/origin/HEAD absent)
  -> git remote set-head origin -a
  -> or add "default_branch: <name>" to .ank/config.yml
```

Two uses depend on it, and not in the same way. `accept` **fails** with code 9: without a reference branch its branch precondition (§4) cannot be evaluated, and running anyway would produce precisely the variable-geometry ratification it exists to forbid. `claim` and `context` **degrade**: they keep completion refs, display them, and warn once. That is "degrade, do not fail" (§2) to the letter — reading does not stop because maintenance is impossible, and `claim`'s refusal on a task finished elsewhere is still delivered, which is the safe behaviour.

### Why `version` coexists with git's CAS

The two cover disjoint ranges. Git's CAS protects **between clones**, at push time. The `version` field protects **inside a single working tree**. With one tree per agent that case becomes rare — but not nil: a human and an agent often share the same tree, and the field costs one integer. We keep it for the residual case, without presenting it as the main defence any more.

### Level 0 — local

No remote. Claims use the **same `refs/ank/claims/<id>` refs, locally**: a local git ref update is already atomic, and level 1 becomes literally "the same ref, pushed" — no migration, no state to convert. There is **no fallback without git**: git is a hard dependency, and an uninitialised repository exits with code 9 and the exact command. Functional without configuration, like a `git init` without a push. Default mode.

### Level 1 — git remote only

Any existing remote, GitHub included. Zero infrastructure.

The central insight: **a git ref update is already an atomic compare-and-swap**. A non-fast-forward push fails server-side, atomically, on every host. That is exactly the primitive claims need — the CAS is guaranteed by git, not by home-made code.

TTL renewal through `log` updates the local ref then pushes; at one log every few minutes and a push on the order of a second, the cost is marginal. The transition to a completion ref at `done` is pushed the same way, and that is what makes it useful: a completion ref that stayed local would tell other clones nothing, which is exactly the window it exists to close. **Offline at level 1**, the claim is taken locally and marked unsynchronised, with a warning: degrade, do not fail — the risk of a concurrent claim is displayed, not hidden.

The trade-off: latency on the order of a second, no notifications. Comfortable up to two or three agents, saturates beyond.

### Level 2 — `ank serve`

A single binary, one port, one SQLite. It stores **only claims**, and broadcasts changes over SSE. Durable state keeps going through git; the daemon never owns it.

Consequence: if the daemon goes down, automatic fallback to level 1, with no possible loss.

Moving from one level to another changes neither the format nor the commands.

### Merging durable state

Two branches that modified the same task meet like any other git conflict: v1 ships **no dedicated merge driver**, resolution is human, and `check` detects leftover conflict markers in `.ank/` (code 8) so that a sloppy merge does not pass CI. Two resolution rules guide the human and prepare a future driver: resolved `version` = max of the two + 1; `## Log` section = union ordered by timestamp (append-only, so the union is always correct). A merge driver automating those two rules is a v1.1 candidate (§13).

---

## 8. Permissions

There is one surface (§4) and the CLI refuses on state. What is left here is **policy applied over that surface**, and policy has to live somewhere it can actually hold. Three places, in decreasing order of how much they hold:

- **SKILL.md**, whose content is frozen (§4, §9). It is what an agent is taught, it is the only description of Ank most agents will ever read, and a verb it does not name is a verb that does not come up. This is the strongest of the three precisely because it does not pretend to be a check.
- **Harness hooks**, which is where enforcement is real. A `PreToolUse` hook that refuses a tool call cannot be talked out of it by an environment variable, because the process it guards does not get to set it. This repository's own hook, refusing direct reads of `.ank/` per ADR-01b6dd05f0db, is the working example.
- **Roles in `config.yml`**, which are **advisory** and are named as such. They catch honest drift and nothing else.

### Roles, and what they are worth

A declarative model in `.ank/config.yml`, identity through `$ANK_AGENT`.

```yaml
roles:
  agent:
    can: [context, find, claim, log, done, new:task, new:adr:proposed]
    cannot: [adr:accept, adr:edit-constraint, task:close, delete]
  human:
    can: ["*"]
identities:
  "marie@laptop": human      # any identifier absent from the table gets the agent role
```

**The default role is `agent`.** An unknown identity — including `$ANK_AGENT` being absent, in which case the fallback identity is `<user>@<hostname>` — gets least privilege. Declaring yourself human in the config confers no real authority anyway: the signature is what carries it.

They are kept, three lines and advisory, rather than dropped. They express intent where a reader can see it, an unknown identity defaulting to least privilege is a sane default, and `check` can say when the record and the behaviour disagree. What they never do is make the CLI refuse: a refusal derived from `$ANK_AGENT` would be a refusal the caller can lift by exporting a different string, and shipping that as a guarantee is worse than shipping no guarantee at all. Dropping them entirely was considered and rejected on the same grounds — the cost is three lines, and honest drift is worth catching.

**One identity per concurrent session, and the fallback cannot tell them apart.** With `$ANK_AGENT` unset, two terminals on one machine both resolve to `<user>@<hostname>`: they are one agent as far as the refs can see, so they share and renew each other's claims in silence. Binding the identity to the session instead — a PID, a TTY — is rejected for the reason above: identity is declared, never proved, and a session-bound identity would also lose a claim to a restarted terminal, which is the one case the 30-minute TTL exists to survive. What the CLI owes the user is therefore the observation, not a stricter identity: `claim` **warns** when the claiming identity already holds a live claim on another task, naming that task and its expiry, and names `$ANK_AGENT` as the way to tell two sessions apart. It warns and never refuses — parallel agents, each with its own identity, are the design, and one claim at a time is a convention rather than a lock (§3).

The main guardrail is not the permission but **the status**: an agent writes freely, in `proposed`. It captures information immediately without being able to constrain anyone. Human ratification is the only path to authority.

### Anchoring human identity

`$ANK_AGENT` is set by the agent itself: an agent going off the rails can declare itself `human`. No file-level check can prevent that, since it has filesystem access.

The only anchor that holds is therefore external: **ratification requires a signed commit**. `accept` produces the ratification commit itself (§12), recording the SHA and the hash of the accepted ADR's `constraint` + `scope`, and `check` verifies that signature against the keys in `.ank/allowed_signers`. The key file is versioned: adding a key to it is a diff in review.

**The file uses git's allowed-signers layout**, `principal [options] keytype key`, with the key type naming the format:

```
sean.lamet@dekrow.com gpg 739A603FB05F9F2F7D3C8D50624FCFCC1482554A
marie@laptop          ssh-ed25519 AAAAC3NzaC1lZDI1NTE5...
```

**Who enforces the allowlist depends on the signature format, and the difference is not cosmetic.** Under `gpg.format = ssh`, git reads the file itself and answers with the match: a signature whose principal the file covers verifies, one it does not comes back as good-but-unmatched. Under OpenPGP, **git never reads the file at all** — it resolves the signature through the keyring — so `check` parses the file and compares the signing key's fingerprint itself. A `gpg` entry may name the full fingerprint or the long key id, since the second is the tail of the first. Without this, the file would go on declaring something nothing enforced.

**Four outcomes, and collapsing any two of them loses the check.**

| Outcome | Reported as |
|---|---|
| signed by a declared key | nothing |
| no signature, or one git refuses | fault: the anchor proves nothing |
| good signature, key not declared | fault: not a ratification |
| signature present, no local public key | signal: **not verified, and not refused** |

The last row is the one that needs saying out loud. A clone without the public key is a correct repository on an incomplete machine: calling it a fault turns CI red on a sound corpus, and calling it verified is worse, because **a verification that degrades to success is not a verification**. It is reported once for the corpus rather than once per ADR, for the reason §4 gives about entities predating `author` — a line per file is the volume that teaches a reader to stop reading `check`.

With **no key declared at all**, `check` does not judge signatures. There is no allowlist to judge against, the advisory notice below already covers it, and the abstention is also what keeps the tool honest: under `gpg.format = ssh` with no allowed-signers file configured, git reports a perfectly signed commit as unsigned, so a corpus without the file would otherwise have every ratification called a forgery.

**What the signature proves — and does not.** It proves access to an authorised key, not human intent. An agent running on a developer's machine whose git signing is configured and unlocked can produce a valid signed commit. The defence against that case is operational, not cryptographic: a ratification key protected by passphrase or hardware (touch-to-sign), distinct from the everyday commit key if needed. This is consistent with the threat model (§1): we protect against drift, not against an adversary.

With no signing configured, **the hard line is gone and nothing replaces it**: roles were already advisory, and the ratification anchor becomes a hash in a commit message that anyone can write. Ank protects against accidental drift there, not against an agent actively trying to get around it. That is an accepted limitation, and `check` displays it rather than hiding it — a corpus whose ratifications cannot be verified says so, since a verification that degrades to success is not a verification.

---

## 9. Bootstrapping

Skill installation leans on the existing ecosystem rather than a home-made mechanism:

```
npx skills add <owner>/ank
```

Skills install from repositories rather than npm packages: the identifier is `owner/repo`, there is no bare name. The skill's repository is therefore called simply `ank`. Installation happens through a symlink, and a `skills-lock.json` versioned in the repository reproduces the same set of skills on every machine in the team — consistent with the rest of the design, where everything that matters is in the repository.

The `skills` CLI handles multi-agent detection (Claude Code, Codex, Cursor, OpenCode, and many others) and creates links from each agent to a canonical copy — exactly the desired design, already maintained by a third party.

**Token economy.** These files are loaded permanently, which is why the content of SKILL.md is frozen (§4): it carries the loop, the planning that fills it, and the mental model behind both — nothing else, and growing that costs a superseding ADR. Flag details and the rest of the surface stay in `ank help`, loaded on demand.

**`ank help` is one flat listing** (ADR-e17e1bbd93ff): every verb, in the order of §4, with no headings and no grouping. What SKILL.md teaches is not what the binary prints — a heading printed by the CLI is a claim about who a verb is for, and that is the claim §4 withdraws. The order carries what a heading would have said, since §4 puts the loop first, and it carries it without asserting a category. `ank help <verb>` answers about that verb alone: its usage, its flags with their value placeholders, and the globals. An unknown verb is a **code 2** and never a fallback to the general listing — an agent that asked about one verb and received all sixteen has to work out that its question went unanswered.

`ank init` keeps a narrow perimeter: create `.ank/`, write `config.yml`, add the `refs/ank/*` refspec (§7), place a pointer in `AGENTS.md`, write a `.gitattributes` (§2) and a `.gitignore` (§6).

Both git files are written at the root of the initialised directory, and both carry a single line added idempotently to whatever is already there. The `.gitignore` line is `.ank/index.db`: §6 calls the index derived, disposable and gitignored, and the third adjective is only true of a repository where something wrote the rule. It goes at the root rather than inside `.ank/` for the same reason `.gitattributes` does — one place to look for what `init` changed — and because ADR-01b6dd05f0db makes `.ank/` opaque to agents, so a rule hidden there could not be read by the agent asking why the index is tracked.

**Binary distribution**: the skill says *how* to use Ank, it does not install the CLI. Plan for `curl | sh` and Homebrew in addition to npm.

**npm is the first channel implemented**, and it is implemented for one reason that shapes it entirely: the workstation whose firewall blocks downloading a bare executable but lets the registry through. The binaries therefore travel **inside** the packages — one package per target, listed as `optionalDependencies` on a wrapper that carries `os` and `cpu`, so npm installs exactly the one that matches. **No `postinstall` download**, because a `postinstall` that fetched the binary would die behind the very firewall the channel exists to cross, and would do it after the install appeared to succeed. The wrapper resolves and executes; it forwards the child's exit code unchanged, since the codes above are an interface an agent branches on, and it reserves 9 — the environment code — for its own failures. The package name is scoped, `@haksolot/ank`: the bare name was taken on the registry before this project existed, and the scope is the closest thing to the name the project actually has (ADR-85e6664c67d8).

---

## 10. v1 scope

### In

File format · one command surface, refusing on state and never on identity, with the loop SKILL.md teaches frozen at eight verbs (`show` included) · HEAD · `release` (reason mandatory) · IDs and prefix resolution · mandatory declared scope · `blocked_by` and derived blocking · named verifiers as a list, timeout, local execution of proofs · freeze by hash (`done_criteria` at claim, `constraint`/`scope` at ratification) · exit codes · two-phase `context` · sync levels 0 and 1, claims on per-task refs, completion refs and their pruning by `check` · `default_branch` and `accept`'s branch precondition · advisory roles, authority anchored on a signed commit, versioned `allowed_signers` · `status`, `edit`, `graph`, `scope`, the interactive form of `new` and the read form of `log` · `attest`, the one write allowed after `done` · bootstrap skill · `check` (full scope in §4).

### Out of v1, in order of expected value

| Deferred | Reason |
|---|---|
| `--since` (differential context) | Large token saving on long loops, but requires per-agent "seen" state. First candidate for v1.1. |
| A CI calling `attest` on its own | The verb itself is in v1 (above). What is deferred is a pipeline appending its run reference without being asked — an integration per provider, not a command. |
| `.ank/` merge driver | The resolution rules are fixed (§7); automating them can wait for the first real conflicts. |
| `touched` inferred from commits | Scope-drift detection. A git dependency, not blocking to get started. |
| `enforced_by` (mechanisation) | The underlying mechanism against context inflation (see §11). Useless while the ADR corpus is small. |
| `ank serve` (level 2) | Level 1 is enough up to three concurrent agents. |
| `ank review --coherence` (ADR corpus analysis) | Detecting contradictions and duplicates. No value on a small corpus. The ratification queue itself is in v1. |
| Read-only web view | To reopen only if non-developers must read the board. |
| Linear/Jira export | Management visibility. Never writing back into Ank. |
| Additional entity types | The common base makes extension trivial later. Do not anticipate. |

---

## 11. Constraint lifecycle

Not implemented in v1, but the model must be laid down now because it determines the `enforced_by` field.

The problem: if ADRs pile up without ever dying, context grows indefinitely and the tool becomes the problem it was meant to solve. A numeric ceiling merely relocates the arbitrariness.

**Mechanisation is the natural sink.** A constraint is born in prose because we do not yet know how to check it. Many become mechanisable — "no `jwt.verify` in `src/auth`" is a lint rule. Once in CI, it has no business in context any more: the feedback loop catches it better, and deterministically. The `enforced_by` field takes it out of injected context without deactivating it.

This turns the pressure the right way round: growing context pushes you to write checks, not to delete decisions.

**Three complementary signals, none arbitrary:**

- **Relative pressure** — what fraction of the `context` budget constraints consume on a scope. Self-scaling, dependent on no hand-picked number.
- **Scope shrinkage** — a constraint declared on `src/**` whose related tasks have only ever touched `src/auth/**` is over-declared. Precision gained, information kept.
- **Structural death** — a scope that no longer matches any file, a broken supersede chain. Verifiable, unlike temporal decay: a three-year-old constraint can be vital. The same rule applies to tasks: a task whose scope is dead is flagged by `check`, never closed automatically — the code may simply have moved.

**Absolute rule: no automatic deletion.** A constraint that was never violated looks exactly like a useless one. The tool detects and proposes; a human ratifies. The only automatism allowed is removal from injected context for what is mechanised — and there it is safe, since CI has taken over.

---

## 12. Implementation

**Rust.** Justified here for two reasons aligned with the goals: a static binary with no runtime (agent agnosticism means imposing nothing on the host environment), and a type system that makes the state machine's invariants — illegal transitions, frozen fields — checkable at compile time rather than at run time.

The "suitable ecosystem" argument used to be third here; it is withdrawn rather than watered down. It rested on `gix` and `clap`, both rejected — git plumbing goes through the binary (below), argument parsing is written by hand (below) — leaving only `rusqlite` and `globset`, whose equivalents exist in every candidate language. That is not what decides it.

**Argument parsing is written by hand**, with no library. The reason is not saving a dependency but character-level control over two surfaces read by agents: the self-correcting errors (§4), which a generic parser would replace with its own messages, and `ank help`, which §9 says carries the flag details — generated help is verbose, and its cost is paid on every call that triggers it. Neither argument depends on the surface being frozen, which it is not (§4): the cost of hand-written parsing is paid once per verb, and the verbs share one parser, so what grows is linear and small against a `help` and an error surface that stay exactly as written.

The real cost is iteration speed on a design that is still moving. Mitigation: freeze and implement the **format parser** first, independently of the CLI. It is the stable part and the only one interoperability depends on.

### Git plumbing

Ank calls the **git binary**, never a library reimplementation. `gix`, considered in order to avoid depending on the system binary, would save nothing: git is a hard dependency anyway (§7). The decisive argument is elsewhere — `accept` and `check` rest on signing. Producing a signed commit and verifying it against `allowed_signers` (§8) is three lines of plumbing with `git commit -S` and `git verify-commit`; it is a cryptographic project with a library, for a result at best equivalent and at worst subtly different from what the user will check by hand.

**Plumbing only**: `update-ref`, `for-each-ref`, `symbolic-ref`, `rev-parse`, `rev-list`, `merge-base`, `verify-commit`, `hash-object`, `cat-file`. Never porcelain — its output offers no stability contract across versions, and parsing it would be exactly the debt that resorting to the binary is meant to avoid.

Three entries are new and serve the ref lifecycle and the default branch (§7). `for-each-ref` enumerates `refs/ank/*` — pruning without enumeration is not implementable, and that gap predated revision e: `check` was already supposed to prune orphan refs. `symbolic-ref` reads `refs/remotes/origin/HEAD` and the current branch. `merge-base --is-ancestor` serves reachability, which feeds the diagnostic and never the pruning decision (§7). `cat-file`, already present, gains a use: reading a task file as it appears on the default branch.

The rule that matters is not the enumeration but its criterion: a command enters this list only if its output is stable by contract across git versions. That criterion is what excludes porcelain, and it excludes it by its reason rather than by its name — a closed list goes stale at every new need, as this one just did.

**Minimum version: git 2.34.** That is the version introducing SSH signing and `gpg.ssh.allowedSignersFile`; below it, `accept` and `check` cannot fulfil their contract. The version is checked at startup, and too old a version exits with **code 9** — an environment to repair, not a task failure — with the upgrade link.

### Ank and git: who commits

**Ank never commits, with one exception.** The writes of `new`, `log`, `done` and `release` land in the working tree and propagate at the pace of the agent's commits, together with its code — the organisational state and the code it describes travel together, which is exactly the tool's promise. Accepted consequence: at level 1, another clone sees a transition only once the commit is pushed; real-time coordination, meanwhile, goes through claims, which depend on no commit.

The exception is **`accept`, which produces the signed ratification commit itself**, containing only the promoted ADR's file (and, where applicable, the replaced ADR moving to `superseded`). The authority model rests on this commit; leaving it to the caller's discretion would make it optional.

That is also why `accept` is the only command carrying a branch precondition (§4): it is the only one writing into history rather than into the working tree. The other commands have no need to know which branch they run on — their write travels with the agent's code and will be arbitrated at merge time like any file. A ratification commit, by contrast, cannot wait for the merge to become authoritative: it is authoritative as soon as it exists, on the branch where it exists.

### Recovery

Ank implements no undo mechanism, no trash and no history of its own, and that is deliberate: **git is already all of those**. A mistaken `done`, an ADR superseded in error, a deleted file are all recovered with the tools the user already knows.

The consequence to respect in the implementation: every operation must produce a clean, readable diff. The log format — an append-only section at the end of the file (§3) — is chosen precisely for that: every `log` is a one-line diff, every transition a minimal frontmatter diff.

---

## 13. Final decisions

**Licence: GPL-3.0.** The criterion applied: modification and commercialisation are free, but a distributed fork must publish its sources — that is the definition of strong copyleft. Two honest clarifications about what the GPL actually guarantees: the obligation to publish is triggered only by *distribution* (a fork kept internal is not bound, and no classic licence imposes that), and it covers the CLI's code, not the format — users' `.ank/` files and the third-party tools that read or write them are not derivative works, which preserves "the format is the specification". A hosted service built on Ank is not bound (the network clause would be the AGPL); for a local CLI that case is marginal, and the AGPL would slow adoption for nothing.

**Platforms: Linux, macOS, Windows, native in v1.** Rust cross-compiles all three without friction, and the portability of the plumbing comes not from a library but from git itself, identical on all three operating systems — git is also what provides the verifiers' `sh` on Windows (§4). One external dependency, present everywhere, rather than a per-platform reimplementation. Distribution: `curl | sh`, Homebrew, Scoop/winget, npm.

**Deferred to v1.1, shape frozen now**: the `.ank/` merge driver (rules fixed in §7 — `version` = max + 1, log = timestamped union — automated at the first real conflicts). `ank attest` was on this list and is not any more: it ships in v1 (§10), and what remains deferred is a CI calling it unprompted, which is an integration and not a verb.

No open points remain: the format, the agent loop, the three platforms and levels 0 and 1 are fully specified.
