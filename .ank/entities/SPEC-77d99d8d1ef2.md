---
id: SPEC-77d99d8d1ef2
type: spec
slug: proof-anchoring-and-authority
title: Proof, anchoring and authority
created: 2026-09-05T14:21:06Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/verify.rs
  - .ank/allowed_signers
references: [SPEC-183d297253ac, SPEC-93531977642f, SPEC-e258796162c4]
supersedes: SPEC-88e1ba60a95d
schema: 4
version: 2
---

One of the ten documents that carry the Ank specification (ADR-5a690829388d).
This one carries the **proof half of §4** — proofs, named verifiers, the trust
hierarchy — together with **§8 entire**: the three places policy lives, roles and
what they are worth, and the anchoring of human identity.

## Why two sections four apart are one document

This is the longest merge the decomposition makes, and it is the one the monolith
argued for without performing. Both halves answer a single question: **what is a
claim about the world worth, and who validated it.**

The trust hierarchy answers it about a run. A reference is worth what its route
is worth — a `test:` reference is the strongest row in the table when a pipeline
wrote it and the weakest when somebody typed it, and the type alone cannot tell
them apart. §8 answers it about a person. An identity is declared and never
proved, roles are advisory because a wall whose bricks are self-declared identity
is a sign and not a wall, and the only hard line in the system is a signature
verifiable against a versioned key file.

Those are the same sentence about two subjects, and the corpus already treats
them as one: the CLI refuses on state and never on identity **because** roles are
advisory, and the ratification anchor is a proof requirement in exactly the shape
every other anchor in this document has. The monolith put four sections between
them, so a reader arriving at the trust hierarchy met the run half of the
argument and had to reach §8 to learn why the person half is weaker — which is
the entanglement the test names, in the form that costs most: two halves of one
argument, each readable and each incomplete.

**What this document does not carry** is the freeze machinery itself. `ratified`,
the hash it holds and the walk that reaches the commit are the data model's,
because they are fields of an entity; what is here is what that anchor is worth
and what verifies the signature on it.

## What it rests on

- **The data model** — the `proof` list, `verify`, and the freezes this anchors.
- **Synchronisation** — detached proofs on `refs/ank/proof/*`, the third
  category of state, and the push that arbitrates them.
- **Implementation, and the decisions that bound it** — the git plumbing that
  produces and verifies a signature, and the rule that Ank never commits except
  `accept`.

---

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

**Two modes, never ambiguous.** If the task declares a `verify`, `ank done` runs **all** the verifiers in the list — a composite `done_criteria` ("the tests pass *and* no more jwt.verify") is mechanised by several verifiers, not by one that covers only part of it — and `--proof` is refused: the agent cannot short-circuit. Every verifier that runs produces **its own proof entry**, with its output hash and its definition anchored. Without `verify` — field absent or empty list, the two forms are equivalent and the canonical form omits the empty field — `--proof` is mandatory and Ank validates what it can: `commit:` is checked with git, `human-review:` and `assertion:` are recorded as they are and marked unverified. Every entry recorded on that path carries `via: submitted`, whatever its type, because that is what happened — the entries produced by a verifier above carry `via: verifier`, and an attestation reaching the task on a ref carries `via: attested` (§3).

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

| Type | `via` | What is guaranteed | What validated it |
|---|---|---|---|
| `assertion:"..."` | `submitted` | Nothing. The agent asserts. **Marked weak** in `check`. | Nobody. Recorded as given. |
| `test:<ref>` | `submitted` | Nothing. A reference typed at a keyboard, in the shape of the strongest proof this table has. | Nobody. Recorded as given. |
| `human-review:<ref>` | `submitted` | A person says they read it. **Marked weak** in `check`. | Nobody. Recorded as given. |
| `test:local/<hash>@<sha>` | `verifier` | Ank executed it, in an environment the agent controls | Ank, running a verifier `config.yml` declares, its definition hashed into the entry |
| `commit:<sha>` | `submitted` | Verifiable by anyone with `git` | Ank, against `git rev-parse` at the moment it was recorded |
| `test:ci://<ref>` | `attested` | Third-party environment, out of the agent's reach | The pipeline that wrote it to `refs/ank/proof/<id>`, under its own identity (§7) |

The dividing line is not local versus hosted, it is **who controls the environment**. Locally, an agent can weaken a test to make it pass — the same class of problem as an ADR edited to unblock oneself.

**The type says what the reference points at; `via` says who put it there, and the second is what the trust rests on** (ADR-b6b69053a47b). The two rows for `test:` are the whole reason the column exists. A run reference is the strongest thing in this table when a pipeline wrote it and the weakest when somebody typed it, and until `via` existed the file said the same thing in both cases — so `--proof test:<anything>` was recorded as strong, unchecked, and silenced the one finding designed to catch a completion nothing external anchors. The fix is not to demote `test:`, which would punish the anchor `attest --detached` and §7 exist to build. It is to record the route and let the signal read it.

**So the `done with no test proof` signal below is derived from the route and never from the type alone.** A `test` entry whose `via` is `verifier` or `attested` silences it; one whose `via` is `submitted` does not, and the finding names the reference it declined to count. **An entry carrying no `via` at all silences it, exactly as it always did**: the field is optional, its absence means the entry predates the distinction, and a rule that reinterpreted the corpus it postdates would redden every completion recorded before it landed. That is the same reading §3 gives to pre-convention actors and to entities predating `author` — the entities mean what they meant.

**This changes what is reported and never what is refused.** A typed reference is still accepted by `done` and by `attest`, still recorded, still part of the proof list. The CLI is not a gatekeeper (§2), and a proof somebody typed in good faith is worth having in the record — it is worth having as what it is.

**A `commit:` reference is validated once, and a rebase detaches it.** The table says what `git rev-parse` answered *at the moment the entry was recorded*, and nothing asked again afterwards — so the strongest proof this tool checks itself comes undone in silence under the routine this project prescribes: a branch rebased onto a newer default branch has its commits replaced, the recorded sha then resolves only on the stale branch, and nowhere at all once that branch is force-pushed. `check` therefore asks the question a second time, of the clone rather than of the entry: a `commit:` proof naming no commit this clone can reach is reported as a **signal naming the task and the reference**, never as a fault and never re-anchored. Never a fault, because an unreachable commit here is a shallow clone, a branch never fetched, or a rebase on somebody else's machine — a check that reddens over the shape of a clone is a check people learn to ignore, which is the reading this section already gives to a dead scope a truncated history cannot explain. Never re-anchored, because choosing which commit now carries the work is a judgement the tool cannot make — the rebase may have split it, or dropped it — and the proof list is append-only (§3): the repair is `ank attest <id> --proof commit:<sha>`, and it is the reader's to make. The dead entry stays where it is; a proof list is append-only, and removing an entry is the rewrite that append-only exists to prevent.

**A clone that cannot see is not asked, and the question costs one git process.** A shallow clone reaches almost no history, so the question is skipped there outright rather than allowed to accuse every commit proof in the corpus at once — the volume failure this section legislates against everywhere else — and so is a clone with no reachable commit at all. It is asked only of `commit:` entries, and only of references in the shape of an object name: a reference git could not read as one is not a question this can ask, and silence is never evidence. The whole corpus is answered by **one git process per invocation and never one per proof**: the set of proofs is tested against a single listing of what this clone can reach.

**What local proof anchors.** An agent's nominal case is an uncommitted working tree: anchoring proof on the HEAD SHA alone would almost always point at a stale state. The proof therefore records three things: the HEAD SHA, a dirty-tree indicator, and **a hash of the scope files' content at execution time** (`tree:scope/<hash>`, git hash-object style). That last one is what actually captures what was tested. `check` additionally reports the case where the task itself modified the test files it invokes.

The levels stack: local proof at `done` time, a CI reference **appended** later to the `proof` list — the proof list is the record a later anchor reaches by appending (§3).

**`ank attest` is in v1, and this settles it.** Earlier revisions deferred it with its shape frozen, on the reasoning that the data structure was ready and the command would come *when a CI called it*. The command was implemented before that happened, and has been used: this repository's own corpus carries `attest`ed CI references. A deferral whose condition has been overtaken is not a plan, it is a document disagreeing with its binary — the state ADR-63b59c5c26f7 orders the work to prevent, and the one a reader could not resolve from §4 and §10 alone (TASK-5c868c20472f).

What that deferral was actually protecting is worth keeping, because it is still deferred and it is not a command: **nothing calls `attest` automatically**. A CI provider appending its own run reference at the end of a pipeline is an integration, not a verb, and it is listed below as such. The verb exists and anyone — an agent, a human, a CI script — can call it today.

**What `check` does instead is notice the omission**, which is the same choice read from the other side. A pipeline that attests itself would assert "a green run existed on a tree containing this task"; the entry carries the frozen criteria hash, so that statement is mechanically true, and it is still not the statement a `test` proof makes. CI proves the tree passes its tests, not that *this* criterion was met — the link between the two exists only because somebody wrote a test encoding the criterion, and no pipeline can know whether they did. Collapsing the two is the rollup hole of §3 one level out. So the assertion stays human, and forgetting to make it is what gets reported (§4).

The `assertion` type exists because "refactor for readability" has no hash to attach. It is allowed but visible as weak, which avoids a `--force` becoming the default path within two weeks.


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

The anchor that holds is therefore external: **a ratification is a commit, and a signature is what makes it answerable to somebody**. `accept` produces the ratification commit itself (§12), recording the SHA and the hash of what was made binding — the accepted ADR's `constraint` + `scope`, or the accepted spec's body + `scope`, under the key that names which (§3, [format.md](format.md)) — and `check` verifies that signature against the keys in `.ank/allowed_signers`. The key file is versioned: adding a key to it is a diff in review.

**Signing is a regime the corpus is in, not a precondition of ratifying** (ADR-964be4d940b2). `accept` signs where the repository is configured to sign and produces an unsigned commit where it is not; it never refuses for want of a key. The two halves of this section used to disagree about that: `check` had the advisory mode described below, in which no key is declared and no signature is judged, while `accept` passed `-S` unconditionally and exited 9 — so a corpus running the very mode this document defines could not produce a ratification at all. Nobody chose that asymmetry; it was two halves written at different times.

What permits the repair is stated a few lines down and is not withdrawn: we protect against drift, not against an adversary. A signature nobody is obliged to produce still anchors every ratification made by someone who does produce one, and git has taken this shape for twenty years without anybody calling its history unauthenticated.

**What may never degrade is the corpus saying which regime it is in.** An optional signature is honest; a corpus that reads as ratified-and-verified when nothing verified it is not. That is the rule this section already applies to its fourth outcome below — a verification that degrades to success is not a verification — and it now governs the unsigned regime on the same terms: every surface reporting on ratification states it, and none lets an unsigned corpus read as a signed one.

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

**So the entity records who ran the ratification, and the signature is not asked to carry it.** `accept` writes the actor into `verified` (§3), typed like every other actor field, beside the anchor it writes into `ratified`. Three statements, and each is the answer to a question the other two do not ask: the anchor says *what* was made binding, the signature says *a key authorised it*, and the reading says *who ran the command*. Without the third, a ratification an agent performed under a cached passphrase and one a human typed at a keyboard are the same bytes in the file. That is not a hypothesis — ADR-782a3556cf2d, SPEC-93531977642f and SPEC-dbbd533cbc78 were ratified on 2026-08-16 by an agent, at the maintainer's explicit instruction from a phone, and every mechanism in this section reported exactly what it was built to report while none of them could say it.

**It is a record and not a defence, and that is this document's bargain rather than an exception to it.** `$ANK_AGENT` is declared and never proved, so an agent can write `human:` in front of its own name exactly as it can write anything else there. What the reading buys is that an honest ratification leaves a trace and that a reader can tell the two cases apart — not that a dishonest one becomes impossible. It is a sign in the sense this section already gives to roles, the signature remains the only hard line, and nothing about the reading is verified by anything.

**`check` reports self-ratification, as a signal and never as a fault**: an entity whose ratifying actor is its own `author`. It is the case the human act exists to prevent, and it is also what a solo maintainer legitimately does every time they write a decision and ratify it — a rule that reddened over it would redden a one-person corpus wholesale and be silenced within a week, which is the volume failure §4 legislates against everywhere else. Reported, and the reader judges.

With no signing configured, **the hard line is gone and nothing replaces it**: roles were already advisory, and the ratification anchor becomes a hash in a commit message that anyone can write. Ank protects against accidental drift there, not against an agent actively trying to get around it. That is an accepted limitation, and `check` displays it rather than hiding it — a corpus whose ratifications cannot be verified says so, since a verification that degrades to success is not a verification.
