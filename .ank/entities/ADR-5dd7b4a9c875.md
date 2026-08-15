---
id: ADR-5dd7b4a9c875
type: adr
slug: the-skill-teaches-investigation-too-and-says-why
title: The skill teaches investigation too, and says why before how
created: 2026-08-15T19:04:57Z
author: claude-code/opus-5
status: proposed
scope:
  - skill/SKILL.md
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/src/cli.rs
  - docs/**
constraint: |
  The CLI exposes one surface: every verb is available to every caller, and the
  CLI refuses on state, never on identity. The only hard authority line is the
  signed ratification commit produced by accept. The verbs themselves do not
  change.
  
  ank help lists every verb. No verb is hidden, and there is no second listing to
  ask for. The listing is grouped by the moment a verb is reached for, under
  lowercase headings, and within a group the order stays section 4's. A group says
  when a verb is used and never who may use it; a heading that sorted callers would
  be the wall this project has already refused twice.
  
  SKILL.md teaches three modes, and its content remains frozen by revision hash.
  The loop: context, claim, show, log, done, with new, find and release off-loop.
  Planning: new adr, amend, review, graph, check, find --status open.
  Investigation: context on a path, scope, find, show, log read with no message,
  status, check on a path, and find --type spec for the specification the corpus
  carries. It states why before how -- two planes joined by scope, and anchoring
  rather than trust. accept stays described and never invited; close, attest and
  edit stay untaught. The ceiling is at most 180 lines and 1500 words, and the
  frontmatter description does not grow: it is the only part a session pays for
  whether or not the skill is invoked.
supersedes: ADR-f61e2d2c75e8
schema: 3
version: 1
---

## Context

`skill/SKILL.md` is the only description of ank most agents will ever read, and
ADR-f61e2d2c75e8 froze it at two modes: the loop, and the planning that fills
it. Two things have made that shape too small, and one of them arrived this
week.

**The file never says why.** It opens on where the files live and goes straight
to the verbs. Nothing in it says that constraints and work are two planes joined
only by scope, or that nothing in the system is trusted because everything is
anchored. An agent taught only the moves knows what to type and not what any of
it protects — and an agent that does not know what a rule protects is the one
that works around it, politely, in good faith. The freeze was written to keep
the file small; it also kept out the twenty lines that make the rest of it make
sense.

**Two verbs exist for a question the skill does not let an agent ask.**
`ank scope <path>` answers *what governs this file* and `ank status` answers
*where am I*. Both were shipped, both are outside the taught set, and the
question they answer is the one an agent actually arrives with — before it has
chosen a task, and often instead of choosing one. The read form of `ank log
<id>` is the third: it is how the next holder learns where the last one stopped,
which is the whole reason `release --reason` is mandatory, and the skill teaches
the write form only.

**And the specification is now in the corpus.** ADR-5a690829388d split it into
ten `spec` entities, ratified and anchored. The normative answer to *what is the
rule* is one `ank find --type spec` and one `ank show` away, from inside the
tool, under the same rule that closes `.ank/` to direct reads. The skill does
not mention the kind at all, so an agent holding it has no route to the document
that defines what it is doing.

## What is added, and what is not

Investigation joins the loop and planning as a third mode: `context` on a path,
`scope`, `find`, `show`, the read form of `log`, `status`, `check` on a path,
and `find --type spec`. Every one of them is a reader. That is the line this ADR
draws — the mode adds nothing that writes, so the thing being taught is how to
arrive informed, not a new way to change the corpus.

`close`, `attest` and `edit` stay untaught, on the reason ADR-f61e2d2c75e8 gave
and this ADR keeps: nothing has decided they are worth what every session pays
for them. `accept` stays described and never invited — the skill says what it is
so that an agent knows where its authority ends, and never shows the command.

**Being untold is still not being refused.** The freeze constrains the
documentation and not the dispatch table, exactly as before. An agent that types
`ank edit` gets the editor.

## The ceiling moves, and the measurement is why

At most **180 lines and 1500 words**, up from 140 and 1200. A ceiling raised to
accommodate whatever was just written is not a ceiling, so the number rests on a
measurement rather than on the length of the draft.

What is loaded on every session is the **frontmatter**, not the body:
`claude plugin details ank` projects `Always-on: ~58 tok added to every session`,
which is the `name` and the `description`, and the body is read when the skill
is invoked. So on the harness this project is developed against, the body costs
nothing until it is used.

That is **not** a licence to let it grow, and the ceiling is kept for the case
it was always really protecting: `docs/agents.md` documents a by-hand route that
copies `skill/SKILL.md` into "whatever that harness loads", and some harnesses
load the whole file every session. The ceiling therefore protects the worst
route rather than the best one, and it is stated here so that the next person to
move it knows which case they are trading against.

The **`description` does not grow**, and that is the half of this with teeth: it
is the only part every session pays for whether or not the skill is ever
invoked. Words go in the body, where they are paid for by the sessions that
wanted them.

## Rejected

**Teaching investigation with the verbs already taught** — `context`, `find`,
`show`, `graph`, `check` — which would have needed no ADR at all, on the
precedent of the two additions that added neither a verb nor a flag. It was the
cheaper path and it leaves out `scope` and `status`, which are the two verbs
built for the question. A mode assembled out of what was already permitted, and
missing exactly the tools for the job, is a workaround wearing the shape of a
decision.

**Raising the ceiling to fit and saying no more.** The number without the
measurement is the thing this project refuses everywhere else: `8000 (default)`
is printed as a default for the same reason.

**Growing the `description` to say the skill now covers investigation.** It
reads as free and is the one thing that is not: it is the always-on half. An
agent that needs to know invokes the skill and finds out in the first four
lines.

## Consequences

`crates/ank-cli/tests/skill.rs` loses `status` and `scope` from the list of
verbs the file must not name, gains a test for the investigation verbs beside
the ones for the loop and planning, and carries the new ceiling. The revision
hash in `metadata.revision` moves, as it does on any edit, and
`the_binary_names_the_skill_revision_it_was_built_alongside` keeps the binary
and the file agreeing.

Section 4 of the specification states the frozen content and section 9 states
what the file carries, so both move — as supersessions, since an accepted spec
whose body is edited is reported `altered`. The four documents citing the CLI
surface follow the chain afterwards with `amend --reference`, which is the
mechanism ADR-5a690829388d asked for, used for the first time.
