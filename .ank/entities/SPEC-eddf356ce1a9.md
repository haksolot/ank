---
id: SPEC-eddf356ce1a9
type: spec
slug: bootstrapping-teaching-and-distribution
title: Bootstrapping, teaching and distribution
created: 2026-08-15T19:07:03Z
author: claude-code/opus-5
status: superseded
scope:
  - skill/**
  - npm/**
  - crates/ank-cli/src/init.rs
  - .github/workflows/**
references: [SPEC-cd0d3377b37f, SPEC-9f510cad4be6]
supersedes: SPEC-fa2f8c49dba4
ratified: 0230435f712c
schema: 3
version: 4
---

One of the ten documents that carry the Ank specification (ADR-5a690829388d).
This one carries **§9 entire**: how the skill is installed and what its token
economy buys, what `ank help` owes its caller, what `init` writes and what it
refuses, and how the binary is distributed.

## Why the help surface is here and not with the verbs

The obvious boundary would put `ank help` in the CLI surface, beside the verbs it
lists. It is here, and the reason is what this document is about.

The surface document says **what a verb does and on what state it refuses**. This
one says **what the caller is told, and by which of three surfaces** — SKILL.md
carries the mental model, `ank help` carries the surface, the error carries the
next command. That division of labour is the subject here, and every rule below
is a consequence of it: a description names a flag only to offer it or to refuse
it, because a description is a fourth surface able to misinform; the listing
prints the same sentence the per-verb page does, because two strings are two
things to keep true; an unknown verb is a code 2 and never a fallback to the
general listing.

Read the other way, the test holds too. The verb semantics are readable without
knowing how they are announced, and the announcement rules are readable without
the verb table — they are rules about a surface, not about any particular verb.

**Bootstrapping, teaching and distribution are one subject** for the same reason:
they are everything that happens on a machine that does not have Ank yet, and
they are governed by one economy. The skill is loaded permanently, so its content
is frozen and its size is a ceiling. `ank help` is loaded on demand, so it carries
the detail. `init` writes the few files that make a repository a corpus. npm
carries the binary inside the package because the workstation this channel exists
for cannot download an executable. Cut anywhere and one of those loses the reason
it is shaped that way.

## What it rests on

- **The CLI surface** — the verbs, flags and refusals `ank help` is required to
  print faithfully.
- **Implementation, and the decisions that bound it** — the platforms and the
  licence the distribution channels carry.

---

## 9. Bootstrapping

Skill installation leans on the existing ecosystem rather than a home-made mechanism:

```
npx skills add <owner>/ank
```

Skills install from repositories rather than npm packages: the identifier is `owner/repo`, there is no bare name. The skill's repository is therefore called simply `ank`. Installation happens through a symlink, and a `skills-lock.json` versioned in the repository reproduces the same set of skills on every machine in the team — consistent with the rest of the design, where everything that matters is in the repository.

The `skills` CLI handles multi-agent detection (Claude Code, Codex, Cursor, OpenCode, and many others) and creates links from each agent to a canonical copy — exactly the desired design, already maintained by a third party.

**Token economy, and what is actually always-on.** The content of SKILL.md is frozen (§4) because what it teaches is what every agent knows: it carries the loop, the planning that fills it, the investigation that precedes both, and the mental model behind all three — nothing else, and growing that costs a superseding ADR (ADR-5dd7b4a9c875). Flag details and the rest of the surface stay in `ank help`, loaded on demand.

**What a session pays for unconditionally is the frontmatter, not the body**, and the distinction is worth stating because the ceiling was justified without it. Measured on the plugin route: `claude plugin details ank` reports `Always-on: ~58 tok added to every session`, which is the `name` and the `description`; the body is read when the skill is invoked. The `description` is therefore the expensive line in the file and does not grow. The **ceiling on the body is kept regardless**, because the by-hand route below copies the whole file into whatever a harness loads and some harnesses load all of it every session: the ceiling bounds the worst route rather than the best one, and a number that protected only the measured case would be a number that stops protecting the moment somebody installs it differently.

**`ank help` lists every verb, grouped by the moment a verb is reached for** (ADR-f61e2d2c75e8, superseding ADR-e17e1bbd93ff on this clause alone): every verb with a one-line description, under a lowercase heading, and inside a group the order stays §4's. No verb is hidden and there is no second listing to ask for. A group says **when** a verb is used and never **who** may use it — a heading named after a caller is the claim §4 withdraws, and it is the one this grouping cannot make: `check` sits under keeping the corpus honest whether a human or an agent types it. The order alone carried what a heading would have said while the surface was five verbs; at twenty-one it carries it only to a reader who already knows the order is meaningful, which is precisely what a first reader does not know. `ank help <verb>` answers about that verb alone: what it does, its usage, its flags with their value placeholders, the globals, and **the state conditions on which it refuses, each with its exit code**. An unknown verb is a **code 2** and never a fallback to the general listing — an agent that asked about one verb and received all sixteen has to work out that its question went unanswered.

**The refusals belong here because nothing else owns them.** §4 carries a whole subsection on refusing by state rather than by identity, and that is a promise to the reader of this document; the caller of the binary had no surface stating which states, or what code came back. The division of labour §9 bets on — SKILL.md carries the mental model, `ank help` carries the surface, the error carries the next command — only holds if the middle one answers *what will this refuse*. It did not, and a session of dogfooding fell through the gap five times, recovering through the source each time. In another repository ank arrives as a binary and a SKILL.md: there is no source to fall through to, and the same five become dead ends.

Three consequences, each of which was one of those five:

- **A flag that is always refused is not a flag.** `ank help` lists what a caller can use; a name the verb rejects by design belongs in the refusals, not in the flag list. Listing it is worse than silence, because the caller reads an offer.
- **A requirement that lives in the state is stated as one.** `done` needs a proof it cannot always produce for itself, and the grammar of one — `<type>:<ref>`, where the type is `commit`, `human-review`, `assertion` or `test` — is part of the surface, not part of the error that fires once the caller has already guessed wrong.
- **Where a value is interpreted, say by what.** `$EDITOR` is a command line run through `sh`, not a program name. A caller told `EDITOR=vi` reasonably supplies a GUI editor, gets no `--wait`, and the editor returns before anything is typed: ank writes the file back unedited and nothing anywhere says so.

The listing and the per-verb page stay two surfaces, and **the listing carries one short description per verb**. It used to carry the verb and its flag names, which is the shape that says what none of them does: `--criteria` beside `amend` names a flag without saying what the verb is for, and a caller choosing between twenty-one verbs is choosing on the usage line alone. Git's listing answers that question in one line each, and this one now does too.

**The description takes the place of the flag names.** They are one `ank help <verb>` away, where they carry their value placeholders and the refusals that qualify them — a bare flag name in an overview is the least informative thing the line could hold, and keeping both would double the height of the listing, which is the token economy this paragraph used to invoke for saying nothing at all. An economy that sends the caller to the source is not one: recovering what a description would have carried cost one session more than twenty lines of reading `human.rs`, `done.rs` and `editor.rs`, plus a scratch repository (TASK-fe130d2b732c).

**The description states what the verb refuses wherever the refusal is what distinguishes it.** This is not a stylistic preference, it is what makes the line honest here: ank's verbs refuse on state (§4), so a description naming only a purpose is a fourth surface able to misinform. `change a task's scope, blockers or criteria` would have confirmed the defect TASK-84cfad83c308 recorded rather than caught it; `change a task's scope or blockers, and a criterion no live claim freezes` is the same line doing the work. Each description is the one-line compression of what `ank help <verb>` already says, never a second text with its own opinion.

**A description names a flag only to offer it or to refuse it, never by accident.** Every `--flag` a description mentions is either one the verb offers, or one it names as refused — `creates .ank/ here or at <path>; refuses --repo` is honest, `changes blockers, scope or --criteria` on a verb that rejects `--criteria` is the defect. The rule is mechanical, and a test walks both surfaces through the binary: it reads the flags and the refusals `ank help <verb> --json` already carries and fails when a description advertises something absent from the first. It is *a flag that is always refused is not a flag*, applied to the surface where a caller reads fastest and checks least.

**The listing prints the same sentence the per-verb page does**, and that is what makes the compression a compression rather than a second text. Two strings would be two things to keep true, and the one that drifts is the one nobody reads twice — which is how `amend` came to advertise a criterion edit the binary refused always (TASK-84cfad83c308). There is one description per verb, `ank help` prints it beside the verb, and `ank help <verb>` prints it above the flags and the refusals that qualify it.

**None of this is a claim about who a verb is for** (ADR-f61e2d2c75e8, superseding ADR-e17e1bbd93ff). A description says what a verb does, and the heading above it says when that verb is reached for; neither sorts callers, which is the claim §4 withdrew and the only one either could make. Both carry what the order alone never could: the order is meaningful only to a reader who already knows it is, and a description is what that reader does not have to know.

`ank init` keeps a narrow perimeter: create `.ank/`, write `config.yml`, add the `refs/ank/*` refspec (§7), place a pointer in `AGENTS.md`, write a `.gitattributes` (§2) and a `.gitignore` (§6).

**`init` takes its target positionally, and refuses `--repo` by name.** `ank init <path>` initialises `<path>`; with no argument it initialises the current directory. `--repo` names a repository that already carries a `.ank/` (§4), which is precisely what this verb is run to produce, so it is refused with `ank init <path>` as the command to type — the shape §9 requires of every refusal, and the reason this one is a refusal rather than a second spelling of the target. `--json` and `--quiet` are unaffected: they say how to answer, not what to act on. Being a refusal by design, `--repo` is absent from what `ank help init` offers and present in what it says the verb refuses, per the rule above.

**What `init` writes, `ank config` maintains** (§4). `config.yml` is written through the verb rather than by hand: it is the last thing under `.ank/` that ADR-01b6dd05f0db left an agent no route to, and the six errors that used to say "add it under `verifiers:` in `.ank/config.yml`" now name the command. `config` joins `init` and `help` as the third verb that runs without the foundation, and for the same reason those two do — the caller who needs it is the one whose environment is wrong, and a repair verb gated on the thing it repairs answers nobody.

Both git files are written at the root of the initialised directory, and both carry a single line added idempotently to whatever is already there. The `.gitignore` line is `.ank/index.db`: §6 calls the index derived, disposable and gitignored, and the third adjective is only true of a repository where something wrote the rule. It goes at the root rather than inside `.ank/` for the same reason `.gitattributes` does — one place to look for what `init` changed — and because ADR-01b6dd05f0db makes `.ank/` opaque to agents, so a rule hidden there could not be read by the agent asking why the index is tracked.

**Binary distribution**: the skill says *how* to use Ank, it does not install the CLI. Plan for `curl | sh` and Homebrew in addition to npm.

**npm is the first channel implemented**, and it is implemented for one reason that shapes it entirely: the workstation whose firewall blocks downloading a bare executable but lets the registry through. The binaries therefore travel **inside** the packages — one package per target, listed as `optionalDependencies` on a wrapper that carries `os` and `cpu`, so npm installs exactly the one that matches. **No `postinstall` download**, because a `postinstall` that fetched the binary would die behind the very firewall the channel exists to cross, and would do it after the install appeared to succeed. The wrapper resolves and executes; it forwards the child's exit code unchanged, since the codes above are an interface an agent branches on, and it reserves 9 — the environment code — for its own failures. The package name is scoped, `@haksolot/ank`: the bare name was taken on the registry before this project existed, and the scope is the closest thing to the name the project actually has (ADR-85e6664c67d8).

**A prerelease never becomes what a bare install resolves to.** The dist-tag is derived from the version and from nothing else: a version carrying a prerelease identifier — a hyphen after the patch, `0.2.0-rc1` — publishes under `next`, and every other version publishes under `latest`. `npm install @haksolot/ank` therefore resolves to the newest release and never to a candidate, while `npm install @haksolot/ank@next` fetches the candidate for whoever asks for it by name. Shipping a candidate stays possible, which is why this is a derivation and not a refusal.

Derived rather than chosen when the tag is pushed, because a flag someone has to remember while tagging is a flag that is eventually forgotten, and this one fails silently: npm 10 applied `latest` to a prerelease without a word, so `v0.2.0-rc1` would have put a release candidate on every machine running a bare install, and nothing would have said so. The rule is **written once and read by both the rehearsal and the publish** — a smoke job passing different flags from the publish rehearses nothing, and the publish is the one step in the pipeline that cannot be taken back. Build metadata is not a prerelease identifier: `1.2.3+build` is a release, and the derivation strips it before looking for the hyphen.

The GitHub release follows the same reading of the same version, and is marked as a prerelease when it is one. Two channels describing one tag differently is how a reader ends up trusting whichever they happened to open.
