//! Conformance of the bootstrap skill.
//!
//! `skill/SKILL.md` is not documentation: it is loaded into an agent's context
//! permanently, on every session, in every repository that installs it. That is
//! what makes its content the thing actually frozen (ADR-91b77f036884) -- the
//! dispatch table refuses nobody, and the two modes exist because this file
//! teaches them and teaches nothing else. Four properties are therefore worth a
//! test rather than a habit: that it carries the whole loop, that it carries
//! the planning that fills the loop, that it stays small enough to be worth
//! loading, and that a copy in the wild says which revision it is.
//!
//! This file exists because the task's declared verifier is `cargo-test`. A
//! criterion nothing executes is a criterion nobody checked, and a proof that
//! covers less than it appears to is worse than a missing one.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

fn repo_file(rel: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

fn skill() -> String {
    repo_file("skill/SKILL.md")
}

// ---------------------------------------------------------------------------
// §4's order, and the listing that has to match it (TASK-973fc0173b98)
// ---------------------------------------------------------------------------

/// The leading verb of a usage line, `ank <verb> ...`.
///
/// `ank --version` yields nothing and is meant to: it is not a verb, and §4
/// says so in as many words.
fn leading_verb(line: &str) -> Option<String> {
    let rest = line.strip_prefix("ank ")?;
    let verb: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
    (!verb.is_empty()).then_some(verb)
}

fn push_once(verbs: &mut Vec<String>, verb: String) {
    if !verbs.contains(&verb) {
        verbs.push(verb);
    }
}

/// The specification document carrying §4's `Commands` block.
///
/// **Read through the binary, and that is the point rather than a detour.** The
/// specification is no longer a file in `docs/` — it is ten entities of kind
/// `spec` in `.ank/` (ADR-c88f99e1c16e) — and `.ank/` is reached through the CLI
/// and never by opening the files (ADR-01b6dd05f0db). A test that walked the
/// directory would be the one reader in the repository exempt from the rule the
/// repository enforces on every agent.
///
/// The document is found by what it carries rather than by its id, so a
/// supersession that replaces it keeps this test green: an id pinned here would
/// have to be re-typed on every revision, and the revision that forgot would
/// look like a passing suite.
///
/// **Read once for the whole binary.** Three tests want this document and
/// `cargo test` runs them on three threads, so this used to put three
/// concurrent readers on one corpus — which is how TASK-e9dfaf187a1b was found:
/// green here, red on all three platforms in CI, the shape of a concurrency
/// defect rather than of a flake. That defect is fixed in the index itself and
/// has its own test, so this is now what it looks like: eleven process spawns
/// instead of thirty-three, for a document that does not change between three
/// reads of it.
fn section_4_document() -> String {
    static DOC: OnceLock<String> = OnceLock::new();
    DOC.get_or_init(read_section_4_document).clone()
}

/// **A proposed successor is not the specification yet, and this is where that
/// bites.** While a supersession is in flight two documents carry the block —
/// the ratified one and its replacement — and the suite has to read the one the
/// corpus is actually held to. `accept` is what makes a document binding, so
/// the ratified one wins, and after ratification the question does not arise
/// again: the predecessor is `superseded` and carries nothing anybody reads.
///
/// The fallback to *any* document that carries it is for the corpus that has
/// none accepted at all — a fresh split, a bootstrap — where reading the draft
/// is better than refusing to read.
fn read_section_4_document() -> String {
    let ids = ank(&["find", "--type", "spec", "--json"]);
    let mut carrying: Vec<String> = Vec::new();
    for id in ids
        .split("\"id\":\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
    {
        let body = ank(&["show", id]);
        if body.lines().any(|l| l.trim() == "### Commands") {
            carrying.push(body);
        }
    }
    // `superseded` is filtered out with `proposed` rather than kept beside
    // `accepted`: a replaced document is not what the corpus is held to either,
    // and letting one through would make the suite read the surface out of the
    // document that was just retired.
    let ratified: Vec<String> = carrying
        .iter()
        .filter(|body| body.lines().any(|l| l.trim_end() == "status: accepted"))
        .cloned()
        .collect();
    let mut candidates = if ratified.is_empty() {
        carrying
    } else {
        ratified
    };
    assert_eq!(
        candidates.len(),
        1,
        "exactly one ratified spec document must carry §4's Commands block, and \
         {} do: the block is what this suite reads the surface out of, so \
         neither zero nor two is a state it can guess its way through",
        candidates.len()
    );
    candidates.pop().expect("one document carries the block")
}

/// The binary, run from this crate's directory so that the walk of §6 resolves
/// the repository's own corpus.
fn ank(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_ank"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("the binary must have been built");
    assert!(
        out.status.success(),
        "ank {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// The verbs of §4's `Commands` block, in the order the block lists them.
///
/// Read from the specification rather than restated here: a second
/// hand-maintained copy of the order is the very drift this is checking for.
fn section_4_order() -> Vec<String> {
    let spec = section_4_document();
    let mut verbs = Vec::new();
    let mut seen_heading = false;
    let mut inside = false;
    for line in spec.lines() {
        if line.trim() == "### Commands" {
            seen_heading = true;
            continue;
        }
        if !seen_heading {
            continue;
        }
        if line.trim_start().starts_with("```") {
            // The opening fence, then the closing one: the block is over.
            if inside {
                break;
            }
            inside = true;
            continue;
        }
        if inside {
            if let Some(v) = leading_verb(line) {
                push_once(&mut verbs, v);
            }
        }
    }
    assert!(
        !verbs.is_empty(),
        "the Commands block of §4 was not found: this test reads the \
         specification, so a renamed heading must fail loudly rather than pass \
         on an empty list"
    );
    verbs
}

/// The listing, as the headings `ank help` prints and the verbs under each of
/// them (ADR-91b77f036884).
///
/// Through the binary, because the ADR is a statement about what the process
/// prints, not about the table it is derived from. The listing is everything
/// above the trailer, which opens with `global:` at column 0: a blank line is a
/// boundary *between* groups now, so a parser that stopped at the first one
/// would read six verbs and call that the whole surface. A folded description is
/// indented, and so opens neither a verb nor a group.
fn help_groups() -> Vec<(String, Vec<String>)> {
    let out = Command::new(env!("CARGO_BIN_EXE_ank"))
        .arg("help")
        .output()
        .expect("the binary must have been built");
    assert!(out.status.success(), "ank help must succeed");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for line in text.lines().take_while(|l| !l.starts_with("global:")) {
        if let Some(v) = leading_verb(line) {
            let group = groups
                .last_mut()
                .unwrap_or_else(|| panic!("a verb stands above every heading:\n{text}"));
            push_once(&mut group.1, v);
        } else if !line.trim().is_empty() && !line.starts_with(' ') {
            groups.push((line.to_string(), Vec::new()));
        }
    }
    assert!(!groups.is_empty(), "ank help printed no group");
    groups
}

/// The verbs `ank help` prints, in the order it prints them.
fn help_order() -> Vec<String> {
    let verbs: Vec<String> = help_groups()
        .into_iter()
        .flat_map(|(_, verbs)| verbs)
        .collect();
    assert!(!verbs.is_empty(), "ank help printed no verb");
    verbs
}

/// Verbs §4 specifies and the binary does not dispatch yet.
///
/// **`mcp` and `watch` are here, and the round trip is the guard working.**
/// `read` was declared here while its ratification was in flight
/// (TASK-d055f8ab836b) and removed in the commit that shipped it
/// (TASK-e3370ef322d8), because `read_section_4_document` reads the ratified
/// document on purpose and a verb therefore cannot ship in the landing that
/// puts it in §4: between the two sits a signed `accept`, which is a human act.
///
/// `tui` went the same way. TASK-8108e3771ba0 put the line in the Commands
/// block of the §4 document of the day and declared the verb here, the
/// ratification of that revision was signed, and TASK-49746735127f dispatched
/// the verb and removed the declaration in the commit that did it -- which is
/// what the second test below forces: a declared verb that has started
/// shipping fails until the line is gone.
///
/// These two are in exactly that state (ADR-fd98f4bc6dea, ADR-a22cd3196529,
/// TASK-36666e36744e). The protocol surface and the watcher stop being sibling
/// executables and become verbs of the one binary distribution carries
/// (ADR-1ea31c2f3c5a), and the Commands block of SPEC-fe8bdb84faca lists them.
/// That document is `proposed`, so the test below does not check either entry
/// yet -- it walks the *ratified* block, which is the predecessor and does not
/// carry the lines. Declaring them now is what makes the ratification
/// itself green: the signed `accept` is the moment §4 gains the two verbs, and
/// a suite that first learns of them there turns red on a human act rather
/// than on a landing anybody is watching. `crates/ank-mcp/**` is
/// TASK-e655d28c83cb's and `crates/ank-daemon/**` is TASK-9dd22f2b0430's, and
/// each removes its line in the commit that dispatches its verb.
///
/// The list stays even when it is empty, because it is the only place a future
/// verb may be declared missing, and because the test below fails the moment a
/// declared one starts shipping -- so the declaration cannot outlive the gap it
/// describes.
///
/// `scope`, `graph`, `status`, `edit` and `tui` were all here in turn, which is
/// the guard working as intended rather than a maintenance chore: shipping each
/// of them (TASK-e717ee625c5c, TASK-253e897d3330, TASK-15336a0012d5,
/// TASK-7ed19b16895e, TASK-49746735127f) turned the suite red until this line
/// was edited, in the same commit.
const NOT_YET_DISPATCHED: [&str; 2] = ["mcp", "watch"];

/// A verb the binary answers to and §4 never mentions. `attest`, `init` and
/// `help` were exactly that until TASK-5c868c20472f, and a reader comparing the
/// two documents could not tell which one was wrong.
#[test]
fn every_dispatched_verb_is_listed_in_section_4() {
    let spec = section_4_order();
    for verb in help_order() {
        assert!(
            spec.contains(&verb),
            "`ank {verb}` is dispatched and §4's Commands block does not list \
             it: the specification is the source of truth (ADR-63b59c5c26f7), \
             so the block is what has to change"
        );
    }
}

/// The other direction, and the reason the exemption list stays honest. Every
/// verb §4 lists either ships or is declared unimplemented -- and a declared
/// one that has started shipping fails here until the declaration is removed.
#[test]
fn every_verb_section_4_lists_ships_or_is_declared_unimplemented() {
    let dispatched = help_order();
    for verb in section_4_order() {
        let exempt = NOT_YET_DISPATCHED.contains(&verb.as_str());
        let ships = dispatched.contains(&verb);
        assert!(
            ships || exempt,
            "§4 lists `ank {verb}`, the binary does not dispatch it, and it is \
             not in NOT_YET_DISPATCHED: either implement it or declare it there \
             with its task"
        );
        assert!(
            !(ships && exempt),
            "`ank {verb}` ships and is still declared unimplemented: remove it \
             from NOT_YET_DISPATCHED, in the commit that implemented it"
        );
    }
}

/// **Grouped by the moment a verb is used, and §4's order inside every group**
/// (ADR-91b77f036884). Until
/// this test the ADR described the output rather than constraining it, and the
/// two orders agreed only while somebody remembered. Fixing the drift of
/// TASK-5c868c20472f introduced a fresh one in the same edit -- `attest` placed
/// after `check` where the binary prints it before -- and only a diff caught it.
///
/// The assertion moved from the listing as a whole to each of its sections, and
/// that is the whole of what the grouping changed: it is a second axis laid over
/// §4's order, not a re-sort, so a verb still never moves relative to its
/// neighbours. Asserting the old global order would now be asserting the clause
/// that was superseded.
#[test]
fn help_prints_section_4s_order_inside_every_group() {
    let spec = section_4_order();
    let groups = help_groups();
    assert!(
        groups.len() > 1,
        "the listing prints one section, so nothing groups it"
    );
    for (heading, printed) in &groups {
        assert!(!printed.is_empty(), "'{heading}' has no verb under it");
        let expected: Vec<String> = spec
            .iter()
            .filter(|v| printed.contains(v))
            .cloned()
            .collect();
        assert_eq!(
            expected, *printed,
            "'{heading}' must print §4's order, minus what it does not hold"
        );
    }
}

/// The loop and what is off it: how work gets done (ADR-91b77f036884).
const LOOP_VERBS: [&str; 8] = [
    "context", "claim", "show", "log", "done", "new", "find", "release",
];

/// Planning: how the work that gets done comes to exist (ADR-91b77f036884).
///
/// `new adr` is spelled out rather than covered by `new`, because the whole
/// point of the addition is the path from noticing an architectural problem to
/// recording one — and `ank new task` alone does not carry it.
const PLANNING_VERBS: [&str; 5] = ["review", "graph", "check", "amend", "new adr"];

/// Investigation: how an agent arrives informed (ADR-91b77f036884).
///
/// The two that carry the mode are `scope` and `status`: they answer *what
/// governs this file* and *where am I*, they were shipped long before this, and
/// an agent taught only the loop had the question and not the verb. `find
/// --type spec` is the third, and it is what the corpus gained when the
/// specification became entities of it (ADR-c88f99e1c16e) — the normative
/// answer reachable from inside the tool.
///
/// `log` is deliberately absent from this list although the mode teaches its
/// read form: the string `ank log` is already required by the loop, so
/// asserting it here would assert nothing. What guards the read form is
/// `the_skill_teaches_the_read_form_of_log` below.
const INVESTIGATION_VERBS: [&str; 3] = ["scope", "status", "find --type spec"];

#[test]
fn the_skill_carries_the_whole_loop() {
    let text = skill();
    for verb in LOOP_VERBS {
        assert!(
            text.contains(&format!("ank {verb}")),
            "SKILL.md never shows `ank {verb}`, and an agent only knows the \
             verbs this file names"
        );
    }
}

/// The half ADR-91b77f036884 added. An agent taught only the loop can execute
/// work and cannot propose a decision, correct a graph, or notice the corpus
/// has gone incoherent -- which is the gap that ADR names and this asserts is
/// closed.
///
/// **What this checks is that the verb is named, not that it is explained**,
/// exactly as `the_skill_carries_the_whole_loop` has always done: the summary
/// block at the top of the file satisfies it on its own. That is the right
/// standard here, because the property being protected is the one the ADR
/// states -- "a verb it does not name is a verb that does not come up" (§11) --
/// and it was measured: deleting the explanatory paragraph alone leaves this
/// green, deleting the name from both places turns it red.
#[test]
fn the_skill_carries_the_planning_mode() {
    let text = skill();
    for verb in PLANNING_VERBS {
        assert!(
            text.contains(&format!("ank {verb}")),
            "SKILL.md never shows `ank {verb}`: planning is frozen content \
             now, not an optional extra"
        );
    }
}

/// The third mode ADR-91b77f036884 added, asserted on the same terms as the
/// other two: the verb is named, because an agent only knows the verbs this
/// file names.
#[test]
fn the_skill_carries_the_investigation_mode() {
    let text = skill();
    for verb in INVESTIGATION_VERBS {
        assert!(
            text.contains(&format!("ank {verb}")),
            "SKILL.md never shows `ank {verb}`: an agent that cannot ask what \
             governs a file, or where it is, investigates by guessing"
        );
    }
}

/// **The read form of `log` is taught, and it is the half that is easy to lose**
/// (ADR-91b77f036884).
///
/// `ank log "<message>"` writes and renews the claim; `ank log <id>` reads, needs
/// no claim, and is where the previous holder said why they handed the task
/// back. The loop already forces the write form to appear, so a test on `ank
/// log` alone would pass on a file that teaches only writing — which is exactly
/// the state this mode was added to end.
#[test]
fn the_skill_teaches_the_read_form_of_log() {
    let text = skill();
    assert!(
        text.contains("ank log <id>"),
        "SKILL.md shows `ank log` only as a write: the read form is how a \
         holder inherits what the last one learned, and an agent that has not \
         been told about it repeats them"
    );
}

/// **The file says why before it says how** (ADR-91b77f036884).
///
/// Asserted by the two things the argument has to establish rather than by any
/// phrasing, on the standard the accept and styling tests already set: a test
/// pinned to one sentence fails on a correct rewrite and reports it as a
/// removal. An agent that knows the moves and not what they protect is the one
/// that works around them in good faith.
#[test]
fn the_skill_says_what_the_rules_protect() {
    let text = skill();
    for token in ["two planes", "anchored"] {
        assert!(
            text.contains(token),
            "SKILL.md does not say {token:?}: it teaches the verbs without the \
             model behind them, which is the gap ADR-91b77f036884 closed"
        );
    }
}

/// **`accept` is described and never invited** (ADR-91b77f036884).
///
/// The two halves are one rule and neither works alone. The skill must say what
/// `accept` is, so a planning agent knows where its own authority ends rather
/// than discovering the boundary by hitting it; and it must never show the
/// command form, because showing a command is how a file that is loaded
/// permanently invites it to be run.
/// The description is asserted by the three things it has to say rather than by
/// one exact phrase: a test pinned to `` `accept` `` with its backticks fails
/// on a rewording that is perfectly correct, and a test that cannot tell a
/// rewrite from a removal is a test nobody trusts twice.
#[test]
fn the_skill_describes_accept_without_inviting_it() {
    let text = skill();
    for token in ["accept", "signed", "default branch"] {
        assert!(
            text.contains(token),
            "SKILL.md does not say {token:?} about `accept`: an agent that \
             cannot see the one hard authority line will find it by hitting it"
        );
    }
    // The other half is `the_skill_teaches_nothing_beyond_what_is_frozen`,
    // which keeps `ank accept` out. Described, never invited -- and neither
    // assertion means anything without the other.
}

/// The budget is the reason: §9 puts the loop, the planning that fills it and
/// the mental model behind both here, and sends flag detail to `ank help`,
/// loaded on demand. The number is a ceiling to notice drift, not a target to
/// fill.
///
/// It moved from 80/700 to 140/1200 with ADR-91b77f036884 and to 180/1500 with
/// ADR-91b77f036884, and each time because a decision said so — which is the
/// only way it is allowed to move. A ceiling raised to accommodate whatever was
/// just written is not a ceiling.
///
/// The last move carries a measurement the earlier ones did not: what a session
/// pays for unconditionally is the frontmatter, `~58 tok` of `name` and
/// `description`, and the body is read when the skill is invoked. The ceiling
/// is kept for the by-hand route, which copies the whole file into whatever a
/// harness loads — it bounds the worst route rather than the measured one.
#[test]
fn the_skill_stays_within_one_page() {
    let text = skill();
    let lines = text.lines().count();
    let words = text.split_whitespace().count();
    assert!(
        lines <= 180,
        "SKILL.md is {lines} lines: a harness that loads the whole file pays \
         for every one of them, every session. Move detail to `ank help`."
    );
    assert!(words <= 1500, "SKILL.md is {words} words, over the ceiling");
}

/// These verbs run for whoever types them -- nothing here is refused to an
/// agent (ADR-91b77f036884). What this asserts is narrower and is the whole of
/// the freeze: SKILL.md does not *teach* them. Naming one as a thing to run
/// would grow what every session pays for, by habit rather than by decision,
/// which is how a permanently loaded file actually grows.
///
/// The list shrinks only by decision, and it has three times. `show` left it
/// when it moved into the loop, back when the loop was still called a surface;
/// `review`, `check` and `amend` left it with ADR-91b77f036884, which bought
/// planning with a raised ceiling and said so; `status` and `scope` left it
/// with ADR-91b77f036884, which bought investigation the same way. `accept` is
/// the interesting survivor: the skill now *describes* it, and still must not
/// show the command, so it stays here while
/// `the_skill_describes_accept_without_inviting_it` holds the other half.
///
/// What is left is `close`, `attest` and `edit`, and they are left for the
/// ordinary reason rather than by oversight: nothing has decided they are worth
/// what every session pays for them. Each one costs a succession.
#[test]
fn the_skill_teaches_nothing_beyond_what_is_frozen() {
    let text = skill();
    for verb in ["accept", "close", "attest", "edit"] {
        assert!(
            !text.contains(&format!("ank {verb}")),
            "SKILL.md shows `ank {verb}`, which is outside the content it is \
             frozen at"
        );
    }
}

/// **What an agent reads is plain bytes, and the skill says so**
/// (TASK-21031b516bb2).
///
/// The guarantee existed and was written down everywhere except the one file
/// every agent actually loads. That gap has a cost the others do not: an agent
/// that does not know the rule can reasonably suspect its input of carrying
/// escape sequences, and the repairs it would then reach for -- stripping
/// output, hunting for a `--no-color` that does not exist, preferring `--json`
/// for cleanliness rather than for parsing -- are all wasted work built on a
/// guess. Saying it once costs one line of a permanently loaded file and
/// removes the whole class.
///
/// Asserted by what the sentence has to establish rather than by its exact
/// wording, for the reason `the_skill_describes_accept_without_inviting_it`
/// records: a test pinned to one phrasing fails on a correct rewrite, and
/// reports it as a removal.
#[test]
fn the_skill_states_that_what_an_agent_reads_is_never_styled() {
    let text = skill();
    for token in ["terminal", "pipe", "--json"] {
        assert!(
            text.contains(token),
            "SKILL.md does not say {token:?}: an agent that cannot read the \
             guarantee has to guess whether its input is styled, and every \
             repair it reaches for from that guess is wasted"
        );
    }
}

/// **The skill states the execution model the loop it teaches assumes**
/// (TASK-e3f4b6295b23).
///
/// §7 states it, `docs/getting-started.md` repeats it, and the one file every
/// agent actually loads said none of it: never a working tree of its own, never
/// a branch of its own, never `ANK_AGENT`. That gap costs more than a
/// documentation gap normally would, because the coordination the skill *does*
/// teach is only correct under the model. "It refuses when the task is held" is
/// true between agents with distinct identities; two sessions sharing the
/// fallback identity are one agent to the refs, and what they get instead is a
/// shared claim and a refusal handed to whichever of them asks second
/// (TASK-a548c95261a5).
///
/// Not a superseding ADR, and the reason is measured rather than assumed. What
/// ADR-91b77f036884 freezes is which *verbs* the file teaches, which
/// `the_skill_teaches_nothing_beyond_what_is_frozen` enforces, plus the ceiling
/// below. This adds neither a verb nor a flag: it is a fact about the
/// arrangement an agent is already working inside, the same register as "the
/// criterion is frozen at claim". TASK-21031b516bb2 added the styling guarantee
/// on exactly those terms — ceiling held, revision regenerated, a test to keep
/// it — and that is the recipe followed here.
///
/// Asserted by the three things the sentence has to establish, and by tokens no
/// other line of the file supplies, so that a correct rewrite passes and a
/// removal fails. `branch` would have been the natural fourth and is deliberately
/// not used: the file already says "finished on another branch", so a test
/// resting on it would stay green over a deleted paragraph.
#[test]
fn the_skill_states_the_execution_model_it_assumes() {
    let text = skill();
    for token in ["worktree", "ANK_AGENT", "degraded"] {
        assert!(
            text.contains(token),
            "SKILL.md does not say {token:?}: an agent that has loaded only this \
             file knows the loop and not the arrangement the loop assumes, and \
             the coordination it teaches is only correct under that arrangement"
        );
    }
}

/// **The skill says how to choose work when other agents are running**
/// (TASK-182a7f58cdd6).
///
/// The file already stated the arrangement — a tree per agent, an identity per
/// session — which is how *not* to become the second agent in one perimeter. It
/// said nothing about being one legitimately, and two ratified decisions
/// produce exactly the output that case needs: ADR-052accd6e3b2 names a live
/// claim whose scope intersects yours at claim time and takes the task anyway,
/// and ADR-47e2ac102f58 makes `status` report this checkout's drift from the
/// default branch. Both are answers to a question the skill never told anyone
/// to ask, so the answers reached no reader.
///
/// The failures that costs are silent when made and expensive when they
/// surface: a task picked off the top of a list overlaps work already running
/// and is found at review, and a branch cut from a stale base is green here and
/// red on the machine that merges.
///
/// Not a superseding ADR, on the same measurement as the test above. What is
/// frozen is which *verbs* the file teaches plus the ceiling, and this adds
/// neither a verb nor a flag: `status`, `claim` and `graph` are all taught
/// already, and what is new is which question to put to them. The ceiling held
/// at 173 lines and 1489 words of 180 and 1500 — written to fit rather than
/// measured and then excused.
///
/// Asserted on tokens no other line of the file supplies, so a rewrite that
/// keeps the teaching passes and a deletion fails. `graph` and `status` would
/// each have been the natural choice and are deliberately not used: the file
/// names both verbs elsewhere, so a test resting on them would stay green over
/// a deleted paragraph.
#[test]
fn the_skill_says_how_to_choose_work_beside_another_agent() {
    let text = skill();
    for token in ["intersects", "unblocked", "collide"] {
        assert!(
            text.contains(token),
            "SKILL.md does not say {token:?}: an agent that has loaded only this \
             file knows it should have a tree of its own, and not how to pick a \
             task when somebody else already holds one — nor that taking none \
             is an available answer"
        );
    }
}

// ---------------------------------------------------------------------------
// Which revision is installed (TASK-b495234f192c)
// ---------------------------------------------------------------------------

/// Frontmatter and body, split on the same delimiters the entity format uses.
/// A second rule for a second file is a second thing to get wrong.
///
/// Line endings are unified first: `.gitattributes` covers `.ank/**` and not
/// `skill/`, so a Windows checkout of this file can legitimately be CRLF and
/// the closing delimiter is then `\r\n---\r\n`.
fn split_skill(text: &str) -> (String, String) {
    let lf = text.replace("\r\n", "\n");
    let rest = lf
        .strip_prefix("---\n")
        .expect("SKILL.md must open with frontmatter")
        .to_string();
    let end = rest
        .find("\n---\n")
        .expect("SKILL.md frontmatter must be closed");
    (
        rest[..end].to_string(),
        rest[end + "\n---\n".len()..].to_string(),
    )
}

/// The value the frontmatter declares under `metadata.revision`, unquoted.
fn declared_revision(front: &str) -> Option<String> {
    front.lines().find_map(|l| {
        let v = l.trim().strip_prefix("revision:")?;
        Some(v.trim().trim_matches('"').to_string())
    })
}

/// **A copy in the wild says which revision it is.** Measured on 2026-08-02:
/// the SKILL.md installed at `~/.claude/skills/ank` was byte-identical to the
/// blob at a004ac7, two commits and nine hours behind a tree that had just
/// withdrawn the invitation to read `.ank/` by hand (ADR-01b6dd05f0db). It was
/// not merely old, it instructed against a ratified decision -- and it carried
/// nothing by which its reader or its owner could have noticed.
///
/// The marker is a hash of the body rather than a version anyone keeps by hand,
/// for two reasons. A hand-kept number drifts the first time somebody edits the
/// body and forgets it, whereas this one cannot: the assertion below recomputes
/// it. And a date would not have caught the case above -- a004ac7 at 10:18 and
/// 7429cdd at 19:16 shipped on the same day, so a date-stamped stale copy would
/// have looked current.
///
/// It sits in `metadata`, which the Agent Skills standard defines as an
/// arbitrary map for properties the standard does not itself define, so it is
/// metadata about the file and not part of what the file teaches. That is the
/// whole of the freeze (ADR-91b77f036884), and the three assertions above are
/// the enforcement -- none of them moves because of a fingerprint.
#[test]
fn the_skill_says_which_revision_it_is() {
    let text = skill();
    let (front, body) = split_skill(&text);

    let declared = declared_revision(&front).unwrap_or_else(|| {
        panic!(
            "SKILL.md declares no metadata.revision, so an installed copy \
             identifies itself to nobody"
        )
    });
    let actual = ank_core::freeze_hash_short(&body);

    assert_eq!(
        declared, actual,
        "SKILL.md was edited without its revision: set metadata.revision to \
         \"{actual}\""
    );
}

// ---------------------------------------------------------------------------
// The siblings, anchored the same way (ADR-91b77f036884, TASK-e26516d35da9)
// ---------------------------------------------------------------------------

/// The sibling skills beside the contract: every `skill/<dir>/SKILL.md`, as
/// `(dir, content)`. Discovered rather than listed, so a skill added later is
/// anchored by the tests below without anyone remembering to enrol it.
fn sibling_skills() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../skill");
    let mut found = Vec::new();
    for entry in fs::read_dir(&root).expect("skill/ must exist") {
        let path = entry.expect("skill/ must be readable").path();
        let md = path.join("SKILL.md");
        if path.is_dir() && md.is_file() {
            found.push((
                path.file_name()
                    .expect("a directory has a name")
                    .to_string_lossy()
                    .to_string(),
                fs::read_to_string(&md).unwrap_or_else(|e| panic!("{}: {e}", md.display())),
            ));
        }
    }
    found.sort();
    found
}

/// The three activities ADR-91b77f036884 names. A sibling beyond these is
/// allowed and anchored like the rest; one of these missing is the skill
/// system shipping a contract without its policies.
#[test]
fn the_three_sibling_skills_exist() {
    let names: Vec<String> = sibling_skills().into_iter().map(|(n, _)| n).collect();
    for expected in ["drift", "loop", "plan"] {
        assert!(
            names.contains(&expected.to_string()),
            "skill/{expected}/SKILL.md is missing: ADR-91b77f036884 names it as \
             one of the three policies beside the contract"
        );
    }
}

/// Each sibling is anchored exactly as the contract is: a declared revision the
/// test recomputes, because TASK-b495234f192c measured what an unanchored
/// installed copy costs and the failure does not care which file it happens to.
#[test]
fn every_sibling_skill_says_which_revision_it_is() {
    for (name, text) in sibling_skills() {
        let (front, body) = split_skill(&text);
        let declared = declared_revision(&front)
            .unwrap_or_else(|| panic!("skill/{name}/SKILL.md declares no metadata.revision"));
        let actual = ank_core::freeze_hash_short(&body);
        assert_eq!(
            declared, actual,
            "skill/{name}/SKILL.md was edited without its revision: set \
             metadata.revision to \"{actual}\""
        );
    }
}

/// The ceiling of ADR-91b77f036884 is per skill, for the by-hand route where a
/// harness loads whole files.
#[test]
fn every_sibling_skill_stays_within_one_page() {
    for (name, text) in sibling_skills() {
        let lines = text.lines().count();
        let words = text.split_whitespace().count();
        assert!(
            lines <= 180,
            "skill/{name}/SKILL.md is {lines} lines, over the 180 ceiling"
        );
        assert!(
            words <= 1500,
            "skill/{name}/SKILL.md is {words} words, over the 1500 ceiling"
        );
    }
}

/// A sibling states the contract applies and adds its own policy; it never
/// restates the rules, and it never shows the one command that is not an
/// agent's to run. Two files teaching one rule drift, and the drift between an
/// installed skill and a ratified decision is the measured failure this whole
/// suite exists around.
#[test]
fn every_sibling_defers_to_the_contract_and_never_invites_accept() {
    for (name, text) in sibling_skills() {
        assert!(
            text.contains("contract"),
            "skill/{name}/SKILL.md never says the contract applies: a policy \
             that restates the rules is a second copy waiting to drift"
        );
        assert!(
            !text.contains("ank accept"),
            "skill/{name}/SKILL.md shows `ank accept`: described, never invited \
             holds in every skill that mentions it (ADR-91b77f036884)"
        );
    }
}

/// The plugin manifest is how a harness finds the skills, so a sibling the
/// manifest does not list ships to nobody.
#[test]
fn the_plugin_manifest_lists_every_skill_directory() {
    let manifest = repo_file(".claude-plugin/plugin.json");
    assert!(
        manifest.contains("\"./skill\""),
        ".claude-plugin/plugin.json does not list ./skill, the contract"
    );
    for (name, _) in sibling_skills() {
        assert!(
            manifest.contains(&format!("\"./skill/{name}\"")),
            ".claude-plugin/plugin.json does not list ./skill/{name}: the \
             sibling exists and no harness will find it"
        );
    }
}

// ---------------------------------------------------------------------------
// Whether the two halves agree (TASK-ecda4070354f)
// ---------------------------------------------------------------------------

/// The `skill <rev>` token of `ank --version`, or `None` if the line does not
/// carry one.
fn printed_skill_revision() -> Option<String> {
    let out = Command::new(env!("CARGO_BIN_EXE_ank"))
        .arg("--version")
        .output()
        .expect("the binary must have been built");
    assert!(out.status.success(), "ank --version must succeed");
    let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let (_, after) = said.split_once("skill ")?;
    Some(
        after
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect(),
    )
}

/// **The binary and the skill can be compared with nothing else in hand.**
///
/// The two markers existed and said nothing about each other: TASK-548c518cb705
/// made the binary name its commit, TASK-b495234f192c made the skill name its
/// revision, and telling a stale installed skill from a current one still needed
/// a third value from somewhere — the repository, a release note, a person who
/// remembers.
///
/// What closes it is that the printed value is *derived* at build time from
/// `skill/SKILL.md` rather than typed. A stamp somebody maintains by hand would
/// reproduce the original failure one edit later, and would reproduce it
/// confidently.
///
/// Through the binary, because the claim is about what an agent holding only
/// that binary can read — a unit test on the formatting would pass on a build
/// whose stamp was never taken.
#[test]
fn the_binary_names_the_skill_revision_it_was_built_alongside() {
    let (front, body) = split_skill(&skill());
    let declared = declared_revision(&front).expect("SKILL.md declares no metadata.revision");

    let printed = printed_skill_revision().unwrap_or_else(|| {
        panic!(
            "`ank --version` names no skill revision, so a reader holding the \
             binary and the skill cannot tell whether the two agree"
        )
    });

    assert_ne!(
        printed, "unknown",
        "the build found no skill/SKILL.md to hash: this suite runs from the \
         repository, where it is there to be read"
    );
    assert_eq!(
        printed,
        ank_core::freeze_hash_short(&body),
        "the binary was built alongside a different SKILL.md than the one in \
         this tree: rebuild, and if it persists the stamp is not being derived \
         from the file"
    );
    assert_eq!(
        printed, declared,
        "the binary and the file disagree on the revision: whichever is stale, \
         the comparison this exists for would mislead its reader"
    );
}
