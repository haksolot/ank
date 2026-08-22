---
id: SPEC-d3adbd77dca9
type: spec
slug: the-cli-surface
title: The CLI surface
created: 2026-08-22T20:00:14Z
author: claude-code/opus-5
status: accepted
scope:
  - crates/ank-cli/**
references: [SPEC-a89ba4f755e9, SPEC-89070ce7f3b8, SPEC-6aed60cd3717, SPEC-88e1ba60a95d]
supersedes: SPEC-c937ff8fa70a
ratified: b88639687d46
verified:
  - by: claude-code/opus-5
    at: 2026-08-22T20:39:47Z
schema: 4
version: 3
---

One of the ten documents that carry the Ank specification (ADR-5a690829388d).
This one carries **§4 without two of its halves**: the one surface and what the
skill teaches, the verbs, the refusals and their exit codes, the commands block,
configuration, short forms, self-correcting errors, and the catalogue `check`
prints.

## Why two halves left, and why the rest did not

§4 was the longest section in the monolith and the only one this decomposition
splits. Two passages sitting inside it failed the test in the direction nobody
looks for — not *this cannot be read alone*, but *this is read by documents that
are not this one*.

**The presentation grammar left.** Its alphabet and its palette are declared
before any code uses them, they read alone, and their dependencies point away
from the verbs: the palette is keyed on the statuses the data model declares and
on the markers synchronisation writes, not on anything in the verb table it sat
inside. A passage whose dependencies lie outside the document holding it is a
passage in the wrong document.

**The proof half left**, and joined §8. Proofs, named verifiers and the trust
hierarchy answer the same question §8 answers about identity — what is a claim
about the world worth, and who validated it. The data model reads it for the
`proof` field, synchronisation reads it for detached proofs, and a passage two
documents depend on should not be buried inside a third.

**The catalogue of `check` stayed.** It is the strongest candidate for a document
of its own and it fails the test cleanly: nearly every finding restates a rule
that lives elsewhere and cites the section that states it, so read alone it is an
index of things it does not contain. An index that cannot be read without the
things it indexes is not a document, and this one belongs beside the verb that
prints it.

Everything else here is one subject: what the caller may type, what comes back,
and on what state it is refused. `config` and the short-form table are part of it
for the same reason the exit codes are — they are the surface, not a mechanism
behind it.

## What it rests on

- **The data model** — every verb here acts on the fields it declares.
- **Presentation** — the grammar of what these verbs print.
- **Proof, anchoring and authority** — what `done` requires and why a refusal is
  never on identity.
- **The attention budget** — `context`'s two modes, and the over-constrained
  finding this catalogue reports.

---

## 4. CLI surface

**Ank exposes one surface** (ADR-e17e1bbd93ff, superseding ADR-c656cbcc33a9). Every verb is available to every caller, and the CLI refuses on state, never on identity. Git has more than a hundred commands, every one of them reachable by anybody who types it, and stays learnable because of what a newcomer is taught first — not because the porcelain sorts callers into classes. Ank borrows that shape.

The memorisation budget is real; it is simply not the binary's job. It is spent on documentation, and it is enforced where it operates: what an agent is taught is a contract every session loads and one policy per activity, each anchored where it lives. What a human may type is everything.

### What the skills teach, and what anchors them

```
Loop:          context → claim → show → log → done
Off-loop:      new, find, release
Planning:      new adr, amend, review, graph, check, find --status open
Investigation: context <path>, scope, find, show, log <id>, status, check <path>
```

**The teaching surface is plural, and no part of it is frozen by hash any more** (ADR-91b77f036884, superseding ADR-5dd7b4a9c875, which carried the freeze forward from ADR-f61e2d2c75e8 and ADR-e17e1bbd93ff unchanged). `skill/SKILL.md` is the contract: the verbs above, grouped by the moment each is used, the model behind them, and the rules that are not negotiable. Beside it lives one skill per activity: `ank-plan` interviews a goal into entities, `ank-drift` audits accepted decisions against the code, and `ank-loop` consumes planned tasks, each loaded when its activity calls for it.

**What replaced the freeze is three anchors, and they are what the freeze was actually protecting.** Every skill declares `metadata.revision`, the hash of its own body, recomputed by test so it cannot drift by being forgotten; the binary names the contract revision it was built alongside, so a stale installed copy has nothing to hide behind (§9); and `accept` stays described and never invited, in every skill that mentions it. Content now evolves under review rather than by supersession, because the lock priced every edit and the anchors price none while catching the failure that was actually measured.

**A description is the trigger, and it is the one line every session pays for** whether or not the skill fires. It names the activity that should load the skill, exactly, and it does not grow without a measurement (§9). The freeze constrained the documentation and never the dispatch table, and that is unchanged: an agent that runs `ank edit` gets the editor; it was simply never told the verb existed, and being untold is not being refused.

**A policy is not restated in the contract, and the contract is not restated in a policy.** A rule written in two files drifts, and the drift between an installed skill and a ratified decision is a measured failure of this project rather than a hypothetical one (TASK-b495234f192c). The contract stays self-sufficient for executing work, since it is the only skill with a broad trigger and the routes of §9 may install it alone, so it names the others as invitations and never depends on them.

**Three modes, because an agent that can only execute is half an agent, and one that cannot look is working blind.** The loop is how work gets done; planning is how the work that gets done comes to exist — deciding which tasks should exist, in what order, under which constraints. ADR-c656cbcc33a9 taught the loop alone, and an agent taught by it could not propose a decision, correct a graph, or notice that the corpus had gone incoherent: `ank new adr` was never mentioned, so an agent seeing an architectural problem had no documented path from the observation to a recorded ADR. Planning is the highest-leverage activity in the corpus, and a badly shaped backlog wastes more downstream than the teaching costs upstream.

**Investigation is the third, and it is the one an agent needs first.** Before choosing a task — and often instead of choosing one — the question is *what governs this file* and *where am I*. `scope <path>` answers the first and `status` the second, both shipped and neither taught, so an agent holding the skill had the question and not the verb. The read form of `log <id>` is the third: it is how the next holder learns where the last one stopped, which is the whole reason `release --reason` is mandatory. And `find --type spec` reaches the specification itself, which is an entity of the corpus since ADR-5a690829388d — the normative answer is one `show` away, from inside the tool, under the rule that closes `.ank/` to direct reads. **Every verb the mode adds is a reader**: it teaches how to arrive informed, and adds no way to change anything.

**The file says why before it says how.** It opened on where the files live and went straight to the verbs, which taught the moves and not what they protect: that constraints and work are two planes joined only by scope, and that nothing here is trusted because everything is anchored. An agent that does not know what a rule protects is the one that works around it, in good faith.

**The ceiling is at most 180 lines and 1500 words, now per skill**, and it stays a ceiling to notice drift rather than a target to fill. The number rests on a measurement, because a ceiling raised to accommodate what was just written is not a ceiling. What every session pays for is the **frontmatter** and not the body: `claude plugin details ank` projects `Always-on: ~282 tok` across four skills, which is their `name` and `description` lines, against `~58 tok` when there was one, and a body is read when its skill is invoked. Splitting the teaching moved cost from the invoked column to the permanent one, and bought the trade §9 states: an agent executing a task no longer loads the planning policy it will not use. The ceiling is kept for the case it was always protecting: §9's by-hand route copies whole files into whatever a harness loads, and some load all of it every session. So it bounds the worst route rather than the best one, and the `description` lines do not grow at all, being the part paid for whether or not a skill is ever used.

**`accept` is described and never invited.** The skill says what it is — a human act, signed, on the default branch only — so that a planning agent knows where its own authority ends, and it never shows the command as something to run. That is the one hard authority line in the system (§8, §12), and describing it is what makes it legible rather than mysterious. `close`, `attest` and `edit` stay outside for the ordinary reason: nothing has decided they are worth what every session pays for them.

That distinction is the whole revision. ADR-3859eb46bdc3 froze an *agent surface* at these same eight verbs and sent everything else to a human side, but the split it protected was never a boundary — the CLI told callers apart by `$ANK_AGENT`, a variable the caller sets itself (§8). A wall whose bricks are self-declared identity is a sign, not a wall. What the freeze actually protected was the token budget, and that protection moves here, where it holds without pretending to be a check.

`show` is in the loop by that route, and the reasoning is worth keeping because it is the argument any addition to SKILL.md has to beat. It sat outside at first, as the only unbounded reader in the system. That argument was about output size, and §5 answers size with a budget and a truncation notice that says what it cut. What it did not answer is the body: `context` serves the criterion and the constraints, never the prose that justifies them, and the prose is where the reasoning an agent is supposed to inherit actually lives. With `.ank/` closed to direct reads (ADR-01b6dd05f0db), withholding it entirely was the worse trade.

### The rest of the surface

These verbs are the rest of the surface. Four of them — `review`, `check`, `amend` and `graph` — are what the planning mode above teaches, and the others are outside what the skill teaches rather than withheld from anyone: none of the refusals below consults who is calling, and an agent that types one gets its answer.

```
review    ratification queue, pending proposals, corpus health
accept    promotes a proposed ADR or spec to accepted (produces the signed commit,
          §8, §12; requires the default branch)
check     mechanical invariants, exit code usable in CI
close     closes a task that will never be done (--reason mandatory)
amend     changes `blocked_by`, `scope`, and a `done_criteria` no live claim freezes
attest    appends a proof to a finished task: the one write §3 allows after `done`
status    where am I: branch, claim, perimeter, queue, findings
edit      changes the content field named, or opens the entity in the editor
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

**`amend` covers the three fields a plan actually changes on**, `blocked_by`, `scope` and `done_criteria`, and covers nothing else. A subtask discovered after a task was filed is a `blocked_by` added to something that already exists; a scope found to omit the files the work must touch is a scope corrected before the task can be claimed at all. Both are ordinary, both were done by hand until this verb existed, and an edit done by hand is indistinguishable in the resulting file from any other edit done by hand — which is the argument that put `attest` on this surface too.

It **adds and removes explicitly** and never takes a replacement list: a verb given the whole list silently drops whatever the caller forgot to repeat. It refuses a `done` or `closed` task, since §3 allows exactly one write after completion and that write is `attest`'s. And it refuses the `scope` of an accepted ADR or spec, whose `scope` is hashed into the ratification commit (§8) — changing a ratified decision is a succession, and succession has its own verb. The two are refused on the same terms and for the same reason, and a spec's anchor simply reaches wider: it covers the body as well, so what an ADR keeps editable after ratification a spec does not (§3).

Amending the `scope` of a task under a live claim is **allowed and warned about**. The claim record anchors the hash of the constraints that scope selects (§7), so the change moves what binds the work in progress. Refusing would be wrong — a scope discovered false mid-task is exactly the situation the verb exists for — and allowing it silently would be worse.

**`amend --criteria` is refused while a live claim freezes the criterion, and allowed the rest of the time.** That is the same state test `edit` already applies (below), and stating it once for both is the point: a criterion under no claim is anchored by nothing, so there is no freeze to respect and nothing for `check` to notice. Under a live claim the refusal is code 6 and names `release --reason`, which is what a criterion discovered wrong mid-work actually calls for.

The route exists because a criterion can turn out **unmeasurable rather than wrong**, and that case had no continuation. Measured on this corpus: a task whose criterion ended on a clause about `gh api community/profile` reporting `issue_template`, which is `null` for any repository using an `ISSUE_TEMPLATE/` directory — the layout that task existed to produce (TASK-7c2fa14284ff). The work was finished and correct; the measurement never could be. The release was right and happened, and then the corrected criterion had two ways back in and both were wrong: a hand edit, which the tool exists to make unnecessary, or `claim --criteria`, which recorded a creator's correction as the claimer's.

**`amend` does not touch `criteria_by`.** That field answers one question — was this criterion set at claim time, by the party the freeze constrains (§3) — and an amend is not a claim. Writing `claimer` there for a correction made under no claim would launder the correction into exactly the shape the signal exists to expose; writing `creator` would assert something about the caller, which no refusal and no field here does. What records the amend is the log entry, as it does for `scope` and `blocked_by`.

**`claim --criteria` sets a criterion, and never replaces one.** On a task that already carries one it is refused, code 6, naming `ank amend <id> --criteria`. §3 gives the flag one job — a task cannot be claimed without a criterion, and the refusal for the empty case names the command that sets it and claims in the same call — and silently overwriting an existing one is not that job. It also keeps `criteria_by: claimer` meaning exactly one thing: nobody wrote a criterion for this task before the agent that took it.

Everything else (editing fields, reordering, deleting) goes through **`ank edit`**, which is the paved road rather than a gate: it opens the same file in the same editor and validates what comes back. Below it, the direct edit remains possible, since the format is the specification and the CLI is not a gatekeeper. The actor matters there and is not decoration: ADR-01b6dd05f0db closes `.ank/` to direct reads and writes *by an agent*, and leaves **a human** with an editor every power they had. Written without the subject, that sentence reads as a general permission and hands back what the ADR withdrew.

### Refusals are on state

The binary refuses, and what it refuses on is always a fact about the corpus or the repository, never a fact about the caller.

| Refusal | Code |
|---|---|
| frozen field diverged from its anchoring hash, or illegal transition | 6 |
| claim held by another agent, or task already finished on another branch | 4 |
| task blocked, no `done_criteria`, `accept` off the default branch, or a live claim already held by the calling identity | 7 |
| proof missing or invalid | 5 |

That list is the guarantee. A state refusal applies to every caller equally and means the same thing to all of them, which is what makes it worth writing an exit code for; a refusal conditioned on identity would mean whatever the caller declared itself to be. `$ANK_AGENT` names who acted — it goes into the log, into the claim ref and into `check`'s signals — and it is never consulted to decide whether a verb runs.

**The one hard line of authority is the signed ratification commit** (§8, §12), and it holds precisely because it is not a role check: `accept` produces it, `check` verifies it against `allowed_signers`, and an anchor no key covers is reported as unverifiable. That is a proof requirement, the same shape as every other anchor in the system. Everything else that looks like permission is policy, and policy lives above the binary (§8).

### Four verbs and two forms, for the reader at the keyboard

With the surface no longer a boundary, verbs serving human ergonomics enter it without ceremony. None of them introduces state: each one composes what `context`, `find`, `review` and `check` already derive, or wraps a write that was being done by hand anyway.

**`status`** answers *where am I* in one call: the branch, the identity in effect and where it came from, the active claim and its expiry, the constraints on the current perimeter, the ratification queue, completion refs the default branch has not caught up with (§7), and the corpus findings `check` would report. It **degrades with a warning** rather than failing when there is no remote or no determinable default branch — the parts that need neither are still worth printing. Terse `git status` register, and it ends with the next command to run, like every other output here.

The identity line names its **source** and not only its value, for the reason `config` prints `8000 (default)` rather than `8000`: `$ANK_AGENT` names a session, the `<user>@<hostname>` fallback names a machine, and only the second is a trap — two sessions in one checkout that both let it fall back are one agent to every ref the tool writes. Nothing else on that path says so, since a `log` or a `done` from the wrong identity is refused on state, correctly talking about the claim somebody else holds (§8). Under `--json` the two are separate fields, `value` and `source`, on the terms `config` already set.

**`edit <id>`** opens the entity in the editor named by `$EDITOR`, validates the result on save, and writes it back in canonical form (§3). **A change to a frozen field is refused by naming the command that legally performs it**: `release --reason` for a `done_criteria` frozen at claim, `new adr --supersedes` for the `constraint` of a ratified ADR. Frozen means anchored *now*: a `done_criteria` no live claim covers is not refused here, which is the same state test `amend --criteria` applies above and the reason the two agree by construction rather than by restatement. An invalid result leaves the entity untouched and says why, so a mistyped frontmatter costs a re-edit and never a corrupt file. This is the argument that put `amend` and `attest` on this surface, generalised: an edit performed by hand is indistinguishable, in the resulting file, from any other edit performed by hand, and chaperoning it through the tool strengthens the invariants instead of relaxing them.

**A field may be named instead** (ADR-5bd8257dfeac): `--title`, `--body` and, on an ADR, `--constraint`, with `--body -` reading the body from stdin as `new` already does. A named edit writes only what is named, leaves every other field exactly as the file had it, and **never opens an editor** — so an unset `$EDITOR` is no failure for a caller who did not want one. With no field named, the editor opens on the whole entity, which stays the road for an edit that rewrites prose around a new argument, where a flag taking a body on stdin is the worse tool.

**The refusals do not fork, and that is the load-bearing half.** A named edit meets `Freeze` where the editor path meets it, refuses with the same code and the same sentence, and differs in one thing only: the editor path adds where it kept the text that was typed, because a refusal after twenty minutes of typing must not discard them, and a flag value is still in the caller's shell. A flag naming a field the addressed kind does not carry is refused by name and nothing is written, since a caller who typed `--constraint` on a task believes they changed something and a verb answering `edited` to that would be lying about the corpus. What the surface changes is how a change is expressed, never what may be changed: `author` is unchangeable (§8), `version` is the store's, and `ratified` is `accept`'s.

**`graph [<path>]`** prints the `blocked_by` DAG in readable text, restricted by an optional path the way `context` is, with `--json` for the raw edges. It **names the perimeter it drew** and says so explicitly when that perimeter holds no task. The ordering of §5 already walks these edges to count what a task unblocks, and this makes the same structure visible to a reader. `show` surfaces the narrow case from the same derivation: on a task it lists what that task **directly unblocks** alongside its blockers, one line each with status. Both are computed from the corpus at read time and stored nowhere — a stored reverse edge is a second copy of `blocked_by` that can disagree with the first.

**`scope <path>`** lists every entity whose declared scope matches the path, grouped by type, one line each with its status, and says so explicitly when nothing matches. The resolution is the **same glob matching `context` uses**, resolved deterministically against the filesystem, so the answer is the one that will actually bind. It is the `check-ignore` of Ank: a dead or over-broad scope is otherwise visible only after the fact, through `check`, and this makes glob resolution observable before an entity is written wrong.

**`new` without its mandatory flags opens a pre-filled template** in `$EDITOR` instead of failing, validates the result, and refuses to write an entity that would not pass validation. The `git commit` pattern: no `-m`, an editor. **The flag form is unchanged and remains the scripted path** — it is what SKILL.md teaches and what an agent uses, and nothing about the interactive form reaches it.

**`log <id>` with no message reads** the entity's entries, newest first, and requires no claim. An entity nobody has logged against reads as an empty log, never as an error. `log` with a message keeps writing and renewing the claim, unchanged (§3). The disambiguation is stated rather than inferred: an argument that resolves to an entity id is a read, anything else is a message, and a message that also resolves to an id is an error naming both readings rather than picking one. This closes the one place the git intuition was betrayed — `git log` reads — without renaming the verb.

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

**Enforcing it is what `claim` does**, and for a long time this paragraph said so while the binary only warned. A `claim` made while the calling identity already holds a live claim on another task refuses with **code 7**, naming the task held, its expiry, and both ways through:

```
$ ank claim 8f3a
error[7]: claude-code@host-3 holds a live claim on TASK-51c2 (expires in 24m)
  -> ank release --reason "<why>"   (a second session on this machine sets its own ANK_AGENT)
```

**Code 7 and not 4.** The task asked for is available; it is the caller that is not. 4 means "take something else" and the reaction called for here is the opposite — finish or hand back what is already held — so the code that carries "missing prerequisite" is the one that says it.

**The second way out is not decoration**, and it is why this hint carries a parenthetical the way the "another ready task" hints do. The default identity is `<user>@<hostname>` (§8), so two sessions in one working tree read as one agent, and the session being refused may never have claimed anything: it is being answered about somebody else's claim, and `ANK_AGENT` is the only thing that separates the two. That is also why the message names the identity rather than addressing the caller as the holder — under a shared identity, telling it "you already hold" would be false.

The refusal comes **after** the refusals about the task itself — no criterion, blocked, already claimed elsewhere, illegal transition. An agent told to release the work it holds, for a task that would have refused it anyway, has been made to pay for nothing.

The refusal is on state and not on identity, in the sense of the table above: what is read is the coordination plane — which refs exist and who holds them — and the answer is the same for every caller. A **lapsed** claim is not a live one, so pickup after expiry (§3) is untouched, and an agent returning to a task whose lease ran out claims it exactly as before.

**It does not make the state unreachable, and nothing here assumes it does.** The CLI is not a gatekeeper (§7, §12): a ref written by hand, a claim taken by an earlier binary, or a claim that lapsed and was revived all produce one identity holding two live claims. What `log`, `release` and `done` do when they meet it is settled in §3, not here.

The optional ID on `log`, `done` and `release` is therefore redundant in the nominal case: it exists for explicitness in scripts, and it **must name a task this agent holds a live claim on**, otherwise error 6. It is never a way to act on somebody else's task. Holding one claim, that is the same rule as "must match HEAD"; holding several — the state §3 describes and this section refuses to create — it is what names which of them a verb acts on.

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
ank claim <id>              [--criteria <c>: sets an absent one, never replaces] [--ttl 30m]
ank show <id>
ank log [<id>] [<message>]  (id alone reads; a message writes)
ank done [<id>]             [--proof <type>:<ref>]
ank release [<id>] --reason <r>
ank new task --title <t> --scope <glob>... [--criteria <c>] [--blocked-by <id>...]
ank new adr  --title <t> --scope <glob>... --constraint <c> [--supersedes <id>]
ank new spec --title <t> --scope <glob>... [--supersedes <id>] [--reference <id>...]
ank new task|adr|spec       (no flags: pre-filled template in $EDITOR)
ank find <query>            [--type task|adr|spec|log] [--status ...] [--scope <path>] [--free]
ank status                  [--remote]
ank review [<path>]
ank accept <id>
ank close <id> --reason <r>
ank amend <id>              [--blocked-by <id>...] [--drop-blocked-by <id>...]
                            [--scope <glob>...] [--drop-scope <glob>...]
                            [--criteria <c>]
                            [--reference <id>...] [--drop-reference <id>...]
ank attest <id> --proof <type>:<ref>   [--detached]
ank edit <id>              [--title <t>] [--body <b>] [--constraint <c>]
ank graph [<path>]
ank scope <path>
ank check [<path>]
ank migrate                 (the previous log directory becomes entries; the command `check` names)
ank config <key> [<value>]  [--unset] [--user]   (a value writes; the key alone reads)
ank init [<path>]           [--at <path>]      (§9)
ank help [<verb>]           (§9)

ank --version               (the build itself: version, commit and skill revision, no verb, no repository)
```

**This block is the whole dispatch table**, and that is a property worth stating rather than assuming: `attest`, `init` and `help` were reachable in the binary while appearing nowhere here, so a reader comparing the two documents could not tell which one was wrong (TASK-5c868c20472f). `init` and `help` are specified in §9 and listed here only so the list is complete; `ank help` prints these verbs, minus whatever is not yet implemented, grouped by the moment each is reached for and in this order inside a group, which is what ADR-f61e2d2c75e8 requires of it. It prints one short description beside each verb rather than the verb's flag names, and the rules that description answers to are in §9.

**`ank context` with no argument** covers the whole repository. It is the first call an agent should make, before it even knows which path it works on — an agent launched on "fix the login bug" does not yet know its perimeter.

**`context` with an active claim is always in execution mode** (§5). A path argument is then ignored, with a warning line: `active claim on TASK-8f3a, execution context (release to explore elsewhere)`. Exploring another perimeter mid-task is precisely what the one-claim-per-agent rule discourages.

`--limit` applies **only to tasks**, never to constraints.

**`find` is subject to the same cap as `context`**, one line per result, and it announces what it cut. A search command without a budget is a context-explosion vector at least as effective as a badly bounded `context`. `--scope <path>` filters by scope match — it is the command the truncation counters point to (§5).

**Scope overlap is reported and never refused** (ADR-052accd6e3b2). `claim` prints one line per live claim held by another identity whose scope meets the scope of the task being taken — the holder, the task, and the ground the two have in common — and then takes the task: the exit code is 0 and the claim stands. The line names paths rather than the fact of an overlap, because `crates/ank-cli/**` against `crates/ank-cli/tests/**` overlaps on everything under the second, and "these overlap" leaves the reader exactly where they started. Where both globs are literal the answer is a path; where one is a pattern the answer is the narrower pattern, written as a glob rather than expanded into a file list that would be wrong the moment a file is added.

Refusing on it is the mistake, not the omission. Scope overlap is coarse: one held task scoped `crates/ank-cli/tests/**` locks every task that touches any test, and refusing would make the glob a mutex — which pushes agents to declare narrower scopes than the truth to get past it, the one failure mode a guard must not have. A **lapsed** claim is not a live one here either, exactly as in the refusals above: the signal would otherwise fire on abandoned work forever.

**Every `elsewhere` line of `status` carries the title of the task it names**, after the id. The rows are already loaded where the line is built, so the join costs nothing, and without it a reader holds an id and has to run `show` once per claim to learn what anybody is doing — which is the question the section exists to answer.

**`status --remote` is the only opt-in to the network outside `claim`** (ADR-47e2ac102f58). Without it, `status` describes the plane this clone has, as §7 says every verb but `claim` does; no network call is made at all. With it, the claims namespace is read off origin with `ls-remote` — read, never fetched, because writing refs into a clone as a side effect of a question would be a reader sanitising the plane underneath everybody else. What that costs is said on the line rather than papered over: `ls-remote` carries ref names and objects, never contents, so a claim seen only on origin is named with its id and the title the corpus already carries, is marked as being only there, and is never given a holder this clone cannot read. With no remote, or one that cannot be reached, the flag **warns once and answers on the local plane** instead of failing (§2).

**`status` says how far this checkout's corpus is from the default branch**, on one line, whether or not it has drifted (§4, ADR-47e2ac102f58). The count comes out of the same inspection `check` renders, so there is one answer and not two, and it is a comparison of two revisions this clone already holds — `status` without `--remote` still makes no network call. Where the question could not be asked the line is absent, which is the one case `check`'s own signal covers in words.

`find --free` is the same computation for the agent choosing rather than taking: it keeps the open tasks whose scope meets no live claim, and **says how many it hid**. A filter that silently returns two candidates out of seven is a filter that will be trusted for the wrong reason. Without the flag, `find` is unchanged.

**Without the flag, a listing counts the open rows a `claim` would refuse, and names it.** `--status` filters on the status the file carries and the markers come from the coordination plane (§7), so a checkout behind the default branch lists ten rows under `--status open` that all read `[finished:… on …]` — every row correct, and the whole answering a question the reader was not asking. Measured: the reader concluded the filter was broken. The markers say it one row at a time; the line says it once and names `--free`, which is the service the truncation counter and the hidden count already perform for their own filters. It counts **open tasks alone**, because that is all `--free` keeps: a `--status done` listing carries `[finished:…]` too until `check` prunes the ref, and sending that reader to `--free` would name a command answering something else — the rule §7 states for hints, applied to a listing. Nothing is dropped, which is why the word is not *hidden*: the rows were listed, and only their number is restated.

**Writing to `log` requires holding the claim**; reading it requires nothing. It is the task's anchoring register: if anyone can write to it, it stops being a reliable trace of what the holder did — that is a state condition on the claim, not a condition on who is calling, and it is why `log <id>` with no message is a read available to everybody. Someone who wants to annotate without holding the task edits the body of the entity, which is already the normal route for anything that is not a state transition.

Commands that take a perimeter take **paths**, uniformly: `review [<path>]`, `check [<path>]` and `graph [<path>]` share the same semantics as `context`, and `scope <path>` resolves against the same globs.

**A path is normalised before it is matched, and a directory has one meaning however it is typed.** Separators are unified to `/`, repeated and trailing ones collapse, `.` disappears and `..` is resolved lexically — so `docs`, `docs/`, `docs\`, `./docs`, `.\docs\` and `docs/../docs` are one perimeter. A path naming nothing inside the repository, because it is absolute or because it climbs above the root, is **refused with the command to run next**; it is never answered. Case is not folded, deliberately: `DOCS` matches nothing here and matches nothing on Linux, and folding it on Windows alone would give one corpus two meanings depending on the machine reading it.

The rule is not cosmetic, and it is written down because its absence was not obvious. Measured on this repository against `docs/`, which five accepted ADRs bind: `docs` answered five, `docs\` answered **four**, `./docs` answered zero (TASK-df4c39031583). The zeros announce themselves. The four does not — it is the form Windows tab-completion produces, it looks like an answer, and it withholds a binding rule from an agent that has no way to know one is missing. A perimeter resolved two ways is a constraint set that depends on typing.

**The rule is about every path and every glob a caller supplies, not about the positional ones.** It covers flag values — `find --scope`, `new --scope`, `amend --scope` and `--drop-scope` — and a glob typed into the `$EDITOR` template, on the same terms: normalised before it reaches matching or is written into an entity, refused with the command to run next when it names nothing inside the repository. A glob normalising to nothing names the repository root, which is a perimeter and not a pattern; it is refused by name, with `**` as the pattern that means what was meant.

Stating it as a property rather than as a list of verbs is the correction TASK-8dd89053fa33 exists for, and the reason is instructive. TASK-df4c39031583 measured the defect, fixed it, and wrote a criterion naming four verbs where the property was general. The fix satisfied that text exactly and its proof is real; the flag values were simply never in it, and the freeze then made the enumeration authoritative. Re-measured with `docs/` present: `--scope docs` and `docs/` answered eight tasks, `docs\` answered **five**, `./docs` and `.\docs\` answered none.

`new --scope` was worse than a wrong answer, because it persisted. It stored `.\docs\` verbatim into the entity, where it matches nothing on any platform for the life of the corpus, and `check` then reported the consequence as `scope '.\docs\' matches no file yet: work not started, or a typo`. It was not a typo — it was the form the caller's own shell completed, which `new` accepted without a word. **A tool that writes what it cannot read back has no way to tell the two apart later**, which is why the normalisation belongs before storage and not in the reader.

Global flags, deliberately limited to four: `--json`, `--quiet`, `--repo <path>`, `--worktree <path>`. Every global flag is a memorisation cost. `--json` is available on every command without exception: full scriptability is an invariant, not an option.

**`--repo <path>` names a repository that already exists**, and that is the whole of what it means: it short-circuits the walk up of §6 without ever contradicting it, so the path it is given carries a `.ank/` or the flag fails. `init` is therefore the one verb that refuses it by name (§9), because `init` is what creates the `.ank/` the flag requires — and where the target is not the current directory, `ank init <path>` is how it is said. The refusal exists because the alternative was measured: accepting it, `init` initialised the *current* repository, appended the pointer paragraph to an `AGENTS.md` the caller was not editing, reported success, and left the named directory empty (TASK-b8a12d60686d). A flag that means an existing repository on twenty verbs and a directory to create on the twenty-first is the same defect as a `-s` meaning `--status` in one verb and `--scope` in another: not a saving, a silent wrong answer.

**`--worktree <path>` names the tree the corpus is anchored to**, which is the half of an address `--repo` does not give (ADR-9e56318631f3). A corpus is confronted with a filesystem for four things: a scope glob, the path argument of a path-taking verb, the working directory of a verifier, and the commit a `commit:` proof names. Until this flag existed all four were held against the directory that happens to hold `.ank/`, because there was only one directory to reach for. §6 carries the assignment; this is the flag that makes the second root sayable.

**Absent, the work tree is the corpus's own directory.** That default is what makes the flag additive rather than a change of behaviour, and it is what keeps the layout §6 describes as usable actually usable: one `.ank/` standing above several checkouts, with scopes written `repoA/src/**`, is a corpus whose work tree is its own directory. A work tree read off the caller's current directory would have killed that layout without anybody deciding to. **The divergence is named, never derived.**

The two flags imply each other in neither direction. `--repo` alone says which corpus and leaves the anchor where it was; `--worktree` alone anchors the corpus the walk found. A work tree that is not a git repository is refused **by the verb that needs git there**, and named as the work tree, so that a reader is not sent to look at a corpus that is plainly a repository. `done` and `attest` are those verbs and no others: git is required per verb and never at startup (ADR-9307e5d214a7), so `show`, `find` and `graph` answer perfectly well from a corpus anchored to a directory that is no repository at all.

### Configuration

**`ank init --at <path>` puts the corpus outside the tree and declares it**, in one gesture (ADR-96174f1ac2b7). It creates the corpus at that path, writes the declaration keyed on this repository's identity, and writes nothing into the working tree -- not a pointer, not a gitignore line, not a comment. The moment it did, the promise would be gone and the design would be the committed pointer that decision refused.

Three refusals, and each is what makes the detachment one. A target **inside the current work tree** is refused naming the tree: a corpus under the code is what `ank init` is for, and a declaration promising otherwise would promise something it does not deliver. A tree with **no repository identity** is refused, because a declaration would have nothing to be keyed on and a fallback is not a key. And a target that is **not itself inside a git repository** is refused naming `git init`, on the terms `ank init` already refuses one: the corpus repository is where claims and proofs land (ADR-9e56318631f3), and `git init` is not on the plumbing ADR-9307e5d214a7 allows, so this verb names the command rather than running it.

What that buys, end to end: a task claimed, logged and finished with its proof anchored, and `git for-each-ref refs/ank` in the code repository printing nothing at all. That is the requirement in its original words -- leave no traces -- and it is the one assertion the whole design exists to make true.

`.ank/config.yml` is read and written through `ank config` (ADR-e64dfaafd578). ADR-01b6dd05f0db closed `.ank/` to agents and stopped at the configuration, which left six errors telling their caller to open a file the same tool forbids them to open. This is the verb those errors name instead.

```
$ ank config claim_ttl_max
2h
$ ank config context_budget
8000 (default)
$ ank config verifiers.cargo-test.run "cargo test --workspace"
verifiers.cargo-test.run (unset) -> cargo test --workspace
$ ank config --unset default_branch
**`--user` reads and writes the reader's declarations instead** of the repository's configuration (ADR-96174f1ac2b7), and it is the same verb for the same reason: the map is where a stale entry makes a corpus vanish, and the discipline `ank config` carries is what a file like that needs. Its key set is closed and holds two, `schema` and `corpora.<identity>`; an unknown key is refused by name with the set it knows; a key that is not a repository identity is refused on the terms the map itself applies, so the two surfaces cannot disagree about what a key is; comments and key order survive a write byte for byte outside the line named; no default is materialised, and no file is created to answer a read. A write that would leave the file unreadable is refused and the file is what it was -- differential, as the repository file's is, so the verb still repairs a file that already did not parse.default_branch main -> (unset)
```

**The key set is closed**, and it is the set the parser knows:

| Key | Value |
|---|---|
| `schema` | the format revision of the file |
| `context_budget` | tokens, default `8000` |
| `claim_ttl_max` | duration, default `2h` |
| `claim_ttl_default` | duration, default `30m`; what `claim` grants without `--ttl`, capped by `claim_ttl_max` (§3) |
| `default_branch` | branch name; unset falls back to `refs/remotes/origin/HEAD` (§7) |
| `peers.<name>` | the path to a peer corpus's root, read and never written (§7) |
| `verifiers.<name>.run` | the command line, §4 |
| `verifiers.<name>.timeout` | duration, default `10m` |

Nested values are addressed by dotted path, and `<name>` is any verifier name — writing `verifiers.<name>.run` for a name the file does not carry is how a verifier is declared. `peers.<name>` works the same way and is how a peer is declared, which is what keeps the closed key set closed: federation adds a key rather than removing the strictness that refuses an unknown one. `verifiers.<name>` addresses the whole block and is legal for `--unset` alone, which is what makes declaring one reversible; reading it, or writing a value to it, is refused by name with `verifiers.<name>.run` to type instead. `roles` and `identities` are keys the parser knows and this verb does not address: their values are structured, the surgery below has no safe edit for them, and they are refused by name rather than guessed at. A key the parser does not know is refused by name too, with the set it does know, and nothing is written.

**Reading prints the value in effect, and marks a resolved default as one.** `context_budget` on a file that does not carry it is `8000 (default)`, not `8000` — the difference between "the tool's value" and "this repository's value" is the whole question a reader is asking, and a release that moves a default moves the first and not the second. A key with no value in effect is `(unset)`. The marker is on the human surface only: `--json` carries `value` and `source` as separate fields, which is what a script reads.

**Writing is text surgery, and never a round-trip through a serializer.** The file is a repository artifact, reviewed like code: comments, blank lines, key order and quoting style survive a write untouched, and every key other than the one named is byte-identical afterwards. A serializer would return `verifiers`, `roles` and `identities` alphabetised, drop every comment, and — the reason this is not a matter of taste — write out every field carrying a default. An unset key means "follows the tool"; a written one means "pinned here", and a round-trip that materialises the defaults converts every repository that ever ran one `ank config` into one that silently pins the old values the day a default moves. **No default is ever materialised**: a key the file did not carry is still absent after a write to another key.

Two edits touch a byte outside the line named, and both are the parent of the key being written rather than a byte beside it: `verifiers: {}` becomes a block mapping when the first verifier is declared into it, and a block mapping becomes `verifiers: {}` when the last one is removed. Without the first, the file `ank init` writes could never receive a verifier; without the second, `verifiers:` would be left with no children, which is not an empty map but a parse error.

**A form the surgery cannot edit safely is refused by name, never rewritten.** A `run` written as a block or folded scalar — `|` or `>` — is one: its resolved string depends on line structure the replacement would flatten, and `verify::definition_hash` is taken over the resolved value, so flattening it would move a hash that anchors historical proofs. The refusal names the key and says to edit the file by hand, which a human always may. Quoting alone is safe and stays safe: the hash is over the resolved string and over the timeout in seconds, so `run: cargo test` and `run: "cargo test"` hash identically, as do `600s` and `10m`.

**A write that would produce a file the parser cannot read fails, and leaves the file as it was.** The result is parsed before anything is written, so the check is not a repair after the fact. The test is differential and has to be: it refuses a write that *introduces* a parse failure, not one performed on a file that already had one. A file carrying an unknown key fails every other verb, and a repair verb that refused to run on it would refuse exactly where it is needed — so on a file that did not parse to begin with, the write goes through and the parse error is reported as a warning rather than a refusal.

**`ank config` runs without a parsed configuration**, as `init` and `help` do (§9). `startup` loads `config.yml` for every other verb, so a file that does not parse fails all of them — `check` included. A verb that exists to repair the file and is disabled by exactly the file it repairs is not a verb; the caller who most needs it is the one whose configuration does not load.

This constrains the agent and not the human. A human with an editor keeps every power they had, and the file stays reviewable text in the repository.

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
| `-u` | `--unset` | `config` |
| `-v` | `--verify` | `new` |

**The letter is the first letter of the long flag, and one letter has one meaning in every verb.** Where several long flags begin with the same letter, exactly one takes it and the others keep only their long form. That is the whole rule, and it is worth the flags it costs: a `-s` meaning `--status` in `find` and `--scope` in `new` would not be a saving but a silent wrong answer, since `ank find -s open` would filter on a scope named `open` and return nothing at all. A short form is only useful if it can be typed without checking which verb it is being typed at.

The long flags left without one, with the letter that would have been theirs: `--body` and `--drop-blocked-by` (`b`), `--constraint` (`c`), `--reason` (`r`), `--scope`, `--supersedes` and `--drop-scope` (`s`), `--title` and `--ttl` (`t`), `--worktree` (`w`). The three globals that carry one take their letter ahead of everything else, because they are legal on every verb and a letter they did not hold everywhere would be a letter nobody could rely on — that is what leaves `--reason` on its long form, `r` being `--repo`'s.

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

`ank help <verb>` shows both forms; `ank help` does not, and the listing is unchanged (§9). The overview buys its token economy by staying short, and the detail is one call away for the caller who wants it.

**`ank --version` is not a fourth global flag**, and the limit above is untouched. The three modify a verb; this one replaces it — there is no `ank check --version`, and asking for the build while also asking for something else is not a question anyone has. It prints the crate version, the commit the binary was built from and the revision of the `skill/SKILL.md` it was built alongside, on one line, and exits 0.

It answers **before the foundation**, like `help` and for a sharper reason: the caller who needs it is the one holding an artifact they cannot identify. A version that required a resolved repository, a git of 2.34 and a readable `config.yml` would fall silent in exactly the situation it exists for. Outside any git repository, in a directory with no `.ank/`, it still prints and still exits 0.

The commit is embedded at build time through `rev-parse`, which §8's plumbing rule already allows, and is `unknown` when the build had no checkout to ask — a source tarball, a vendored dependency. Naming the absence beats inventing a value, and it beats the silence that made TASK-1ea38a17d854 cost an investigation to conclude that the binary in hand predated the feature being measured for.

**What the stamp guarantees, and on which builds.** It names the commit the build script last ran at, and the script reruns whenever the commit moves — including a commit that changes no source file, which is the case that catches a binary quietly left behind. It does not depend on the source having changed. The files a commit moves are located through `rev-parse --git-path` rather than assumed at `.git/`, so a linked worktree and a packed ref are covered rather than hoped for; on a detached HEAD the sha lives in the HEAD file, which is watched on its own account. A release, built from a fresh checkout, is exact by construction. The stamp is not a claim about the working tree: uncommitted edits are invisible to it, and a binary built from a dirty tree names the commit it was based on, not the code it contains (TASK-0b26c8b5bfc5).

**The skill revision, and what it lets a reader do offline.** The third value is `skill <rev>`, where `<rev>` is the short form of the same freeze hash that anchors a frozen `done_criteria` (§3), taken over the body of `skill/SKILL.md` — the same value that file declares under `metadata.revision`, computed at build time from the file rather than typed. An agent has both halves in hand and nothing else: the skill is loaded into its context, the binary is on its `PATH`. Comparing the two strings tells it, with no repository and no network, whether the instructions it is following predate the tool it is holding. That is the check TASK-1ea38a17d854 needed and did not have, and the case that motivated it was measured (TASK-b495234f192c): an installed `SKILL.md` two commits behind a tree that had just withdrawn the invitation to read `.ank/` by hand. The hash covers the body and not the frontmatter, so the revision the frontmatter carries does not change its own input. It is `unknown` on a build with no `skill/` to read, for the same reason and with the same honesty as the commit.


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
| 7 | missing prerequisite — no criterion, task blocked, `accept` off the default branch, or a live claim already held by the calling identity |
| 8 | `check`: invariants violated (reserved for `check`, for CI) |
| 9 | environment unavailable — not a task failure |

Codes 3 and 4 are the ones the agentic loop must know how to handle. 3 literally means "redo `context`, somebody moved". 4 means "take something else", and its two causes call for the same reaction, which is the reason for uniting them under a single code: in both cases the task is not to be taken, and the message says which one to take instead. `check` exits 0 when the corpus is healthy and 8 when it has findings — never 1, so that CI can distinguish a sick corpus from a broken tool.

9 covers the environment broadly and not only the verifiers': `sh` not found (§4), git absent or older than 2.34 (§12), default branch indeterminable (§7), a detached proof whose ref never reached the remote (§7). What they have in common is that none of them is a failure of the agent's work — it is an environment to repair, and confusing it with a 1 or a 5 would send the agent to fix sound code.

### Self-correcting errors

Never generic help, always the exact command to run next. One well-designed error round trip costs less than three blind attempts.

```
$ ank done
error[5]: proof required to move TASK-8f3a to done
  done_criteria: "Auth integration tests pass, and no reference
                  to jwt.verify remains in src/auth/"
  -> ank done --proof commit:<sha>
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


### Scope of `check`

Summary of the invariants and signals, all mechanical:

- expired claims, `blocked_by` cycles, broken supersede chains, dead scopes (no file matched, and see below), over-constrained scopes (§5);
- frozen fields diverging from their anchoring hash — `done_criteria` against the claim, `constraint`/`scope` against the ratification commit, and the signature on that commit against `allowed_signers` (§8): an anchor read from a commit nobody signed anchors nothing;
- weak proofs (`assertion`, unverified), `done` tasks modified beyond appending a proof;
- a `commit:` proof naming **no commit this clone can reach** — rebased away, a branch never fetched — reported as a signal naming the task and the reference, one line per detached reference, never a fault and never re-anchored (see the trust hierarchy above). Skipped outright where the clone cannot see: shallow, or reaching no commit at all, since a truncated clone would otherwise have every commit proof in the corpus reported at once;
- behavioural signals, reported without being faults: blockers created by the holder after claiming (`author` of the blocker is the current holder and its `created` is later than the claim), criterion set by the claimer, verifier modified inside the task's activity window or proof hash diverging from its definition, scope test files modified by the task that invokes them, burst creation by a single identity (**more than 10 entities by one `author` within an hour**, through `created`), implausible `created` (in the future, or well before the commit that introduces the file — the field is declarative, git is the anchor), repeated claim renewals with no modification to the scope files (possible hoarding; a best-effort signal, since another agent's tree is not observable), constraint accepted after the claim of a task in progress, tasks blocked by a `closed` task;
- a **discrepancy recorded against a frozen criterion** (§3) — a log entry whose message opens with `discrepancy:` — reported as a signal on the task, at any status, and never as a fault: it says a holder measured one part of the criterion wrong and kept the criterion anyway, which is a judgement and not a corpus defect, and a criterion that actually moved is the divergence fault above. **One finding per task, with the entries listed under it**, because the task is what is being judged; and it is worth most on a `done` one, where it is the only thing telling a reader that the proof was produced under a disagreement. A log this reading cannot parse is said out loud on the same task rather than counted as no record, since a check that reports nothing because it read nothing is the quiet failure §4 refuses everywhere else;
- a `done` task carrying no `test` proof **that anything validated** — none at all, or only ones a caller submitted — once the **default branch** carries it as `done`: the completion rests on a local run and nothing external anchors it. Read on `via` and not on the type (see the trust hierarchy above), so a submitted reference is named in the finding rather than counted by it, and an entry predating `via` counts as it always did. A signal and never a fault — the corpus is intact, the record is thin, and exiting 8 would redden CI on the very merge that introduces the task. The gate on the default branch is what makes it actionable: before the merge there is no run to cite, and reporting there would name work the reader cannot do. The window between the merge landing and its run going green is left to fire, since the statement is true when printed and clears when someone attests; buying that quiet would cost a grace constant, and the constants below are justified for flooding alone;
- entities predating `author`, **reported once for the corpus and never per file**: they are skipped by the two signals above, and saying so once is what keeps that fact visible. One line per file would add a line for every entity written before the field existed — the volume that teaches a reader to stop reading `check`;
- actor values not matching the typed convention of §3, **reported once for the corpus and never per file**, on the same reasoning and for the same volume as the line above: the convention binds new writes, and the entities that predate it mean what they meant. A malformed actor is a finding here and never a parse error, or a rule would lock out the files it postdates;
- an entity whose `author` is an agent and which carries **no reading by a `human:`** in `verified` (§3): a signal, one per entity, and never a fault. Nothing requires `verified`, and `check` derives what the fields state and nothing further — no score, no confidence, no ranking;
- a corpus still in the previous per-kind layout, or still holding a work trace in the shape that preceded entries (§6), **reported once each**, naming the command that moves it: a signal and never a fault, since such a corpus parses, round-trips and answers every verb;
- **a `references` entry of a spec that does not resolve** (§3, ADR-c88f99e1c16e), one line per entry. **A reference names a document and not a revision of it**, so the entry is followed through its succession and what is judged is the entity the chain ends on, whatever the chain's length. A **fault** where the corpus holds no such entity, or holds one of a kind a specification does not cite — the reader following it finds nothing, and the repair is `ank amend <spec> --drop-reference <id>`; a **signal** where the chain ends on a document that is not `accepted`, naming `ank accept` on the entity it ends on, since two documents are legitimately drafted at once; and a **signal** where it ends on a `superseded` entity that nothing replaces, which is a citation with nowhere to follow to and a corpus defect reported against that entity rather than against whoever cites it. A chain ending on an accepted document is reported by nothing at all. **Nothing is written to make a reference resolve**: the file keeps the identifier its author wrote, no version moves, and no machinery entry is deposited by a read — which is the argument that settled the alternative, where repairing citations in place would have had one `accept` write to nine entities and leave nine entries behind (ADR-16813b3bcf37). The rule it replaces let a citation off only when the citing document *also* stored the end of the chain, which is this same resolution spelled by hand, stored twice, and re-stored on every citing document after every revision. Only the declared field is read: a section number written in prose is not a reference and no finding pretends otherwise;
- **an entity whose content its entries cannot account for** (§3, ADR-f7dc76886db2): a signal naming both hashes, one line per entity, and never a fault. A machinery entry records the hash of the content its write produced, where content is every field a transition does not write -- `status`, `proof`, `ratified` and `verified` belong to a transition and `version` to the store -- and `check` compares the newest such entry against the entity as it stands. Equal, and the entity has not moved since the CLI last wrote it; unequal, and it has. **The comparison is what survives a claim**, and that is why it replaced a count: `claim` and `release` each write a task file and leave no durable record naming a version, so a task claimed and released five times carries ten versions no reader can evidence, and a rule counting them would fire on the most ordinary sequence in the corpus. It is also stronger where both apply -- a hand edit that does not move `version`, which is the likelier one since `version` is machinery a human has no reason to touch, is invisible to a count and caught here. **It says the write happened and never that it was wrong**: editing an entity file by hand is legal, is what ADR-01b6dd05f0db permits a human while asking it of no agent, and exiting 8 over it would redden a pipeline on an act the corpus allows. An entry carrying no produced hash is silent, and an entity carrying no entry is silent: no corpus is migrated by a rule it predates;

- **an entity whose version its entries cannot account for** (§3, ADR-f7dc76886db2), which is the count kept where it closes: a signal naming both numbers, one line per entity, never a fault, and reported only where the hash above found nothing, so one write is one finding. The regime opens with an entity's first machinery entry, so everything before it is forgiven and the baseline is that entry's `version <from>`; each entry accounts for one version, and so does each write the entity's own fields evidence -- `ratified` for a ratification, `status: superseded` for the far side of a succession. A version above that total is reported. **It is not attempted for a task**, and the silence is derived rather than chosen: `claim` and `release` leave nothing an evidence count could read, which is the whole of why the hash exists. What it still catches that the hash cannot is a version moved and nothing else. **And it concludes nothing about an entry whose message it cannot read**: an entry is written once, one marked as machinery by a newer build is entitled to a message of its own shape, and the accounting steps aside rather than reporting on prose it does not own;
- unresolved git conflict markers in `.ank/` files (§7);
- **the corpus this checkout carries against the corpus on the default branch** (§7, ADR-47e2ac102f58), **reported once for the corpus and never per entity**: a signal naming how many entity files differ — held here and not there, there and not here, or held on both sides with different content — and the git command that closes the gap. Nothing is fetched to answer and nothing is merged: both revisions are already in this clone, and the gap is git's to close on the operator's word;
- maintenance of the coordination plane (§7): pruning orphan refs, and completion refs whose task is `done` or `closed` on the default branch. `check` is the only command that prunes. A task carrying a completion ref for a long time without the default branch catching up is reported as a signal — that is a branch never merged, not a corpus anomaly, and the answer is human.

**A dead scope git can explain is a signal, and the severity rule only ever lowers.** Structural death is reported as a signal for an open or `in_progress` task — work not started, or a typo — and as a fault for an ADR or a finished task, which claimed to touch files that are not there. On top of that, and only for the entities that would fault, a second question is asked: did git record where the path went (ADR-97beaf55e73a)? A yes lowers the fault to a signal. Nothing here raises a severity, so an open task whose dead scope git cannot explain is a signal exactly as before.

The reason is that the two states are not the same fact. When git names the commit that moved the path, the corpus is not broken — it is outdated in a way the reader can see and follow, which is what the rename walk was built to show. When git cannot explain it, the reader has nothing, and that is what the fault is for. Without the split, any directory rename reddens a corpus permanently: `amend` refuses a `done` task, so such a task gets the rename named and no repair command (§3), and a fault nobody can clear is a finding readers learn to skip.

**A history that cannot answer is a third state, and it is not a fault.** A shallow clone holds no commit that could record where a path went, so the question the paragraph above asks has three answers and not two: git names the rename, git has the history and records none, or git has no history to consult. The last is reported as a signal saying so, naming the command that deepens the clone, and it is never rendered as a rename that did not happen nor as a defect in the corpus. This is the answer §3 already gives for a ratification anchor a shallow clone cannot verify, and for the same reason — a check that cries divergence over the shape of a clone is a check people learn to ignore. It follows that a pipeline running `check` must check out the history the walk needs, or it reports that it cannot verify and verifies nothing, which is worse than the failure it replaces because it is quiet.

**The walk serves a glob through its literal prefix.** `rev-list` answers about a path and has no answer for "where did `src/**` go", so a glob is asked about the part of it before the first wildcard, truncated at the last separator: `.ank/adr/**` asks about `.ank/adr`. A prefix whose files git records as renamed into one directory is explained by that directory, and the repair proposal keeps the wildcard tail — `.ank/adr/**` becomes `.ank/entities/**`. Sources landing in more than one destination, or under a rename that also changed a file's name, produce no explanation: silence is never evidence, and "the prefix moved mostly there" is not a statement this document authorises anyone to print.

**Drift is counted in entity files, never in commits.** How far this corpus has drifted from the default branch is a question about two trees, and `rev-list` answers a different one: a default branch can move ten times without touching `.ank/`, so a count in commits would fire on every merge and mean nothing. What is counted is entity files that differ, which is what a reader can act on — and it is one line for the corpus rather than one per file, the volume rule this section has already applied twice above.

**Unable to compare is not "nothing has moved", and the two are never printed the same way.** The question is skipped in silence where it cannot be asked at all: no repository (the coordination half already says so once), or no resolvable default branch (the line about pruning already says so once). It is said out loud where it was asked and refused — a `default_branch` naming no commit in this clone — because a mistyped branch name rendered as silence is a reader told the corpus is level by a check that never looked. A repository with no commit at all is neither: that is the nominal state of one freshly `ank init`-ed, there is nothing to compare against yet, and it is silent.

**The same fact reaches `status` on one line**, out of the same inspection pass rather than computed a second way, because two answers able to disagree is the defect that surface exists to avoid. `status` prints it whether or not there is drift — a reader who has to tell "level" from "not asked" by the absence of a line is reading silence, which is the thing this section refuses everywhere else.

**The two signals that need `author`, and why they are signals.** A blocker written by the agent currently holding the task is the shape of an agent building itself an excuse — but it is also the shape of an agent doing exactly what §3 asks, since a discovered subtask *is* a new task with a `blocked_by`. Only a reader knows which, so it is reported and never refused. Burst creation is the same argument at the corpus scale: §3 accepts task flooding without a quota, on the grounds that the defence is visibility rather than restriction, and this is that visibility.

**The numbers are constants, not configuration.** More than 10 entities by one `author` within an hour: a threshold high enough that a session filing the four tasks of a plan passes in silence, low enough that a runaway loop is named within minutes. They live in the tool rather than in `config.yml` because a repository that can raise its own flooding threshold has a flooding threshold that will be raised the first time it fires — and the signal costs nothing to ignore, which is what makes it safe to leave unadjustable.
