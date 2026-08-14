//! check, review, accept, close, attest, amend and show (§4, §8, §11).
//!
//! **This is a file, not a side.** The CLI exposes one surface
//! (ADR-c656cbcc33a9): every verb here is available to every caller, and what
//! the module holds is what grew together rather than what a class of caller
//! was allowed to run. Successive headers described this as the human half, the
//! side that may grow freely, the half outside the loop — each true when
//! written and each false a decision later. A header describing an audience is
//! exactly the prose that rots into a lie one decision at a time, so there is
//! none.
//!
//! **`check` is the only command that prunes.** `claim` and `context` are
//! readers, and a reader does not sanitise the coordination plane underneath
//! everyone else; concentrating maintenance in one command is what makes its
//! timing predictable (§7).
//!
//! **`accept` is the only command that commits** (§12). It produces the signed
//! ratification commit itself, because the authority model rests on that commit
//! and leaving it to the caller's discretion would make it optional. It refuses
//! to run outside the default branch, with no bypass: a constraint ratified on
//! a feature branch is a constraint of variable geometry, and a `--force` would
//! become the default path within two weeks.
//!
//! **Faults and signals are not the same thing.** A fault is a corpus defect
//! and exits 8, which is what CI routes on. A signal is reported and exits 0:
//! "criterion set by the claimer" is worth seeing and is not a violation.
//! Conflating them would make `check` a command people learn to ignore.

use crate::claim::{self, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::git;
use crate::index::Index;
use crate::repo::Repo;
use crate::store::{version_of, LogHome, Store};
use crate::style;
use crate::verify;
use ank_core::{
    freeze, has_crlf, normalise_line_endings, parse_entity, serialize_entity, verify_frozen, Adr,
    AdrStatus, Entity, EntityId, EntityKind, LogEntry, Task, TaskStatus, Verified,
};
use ank_core::{CriteriaBy, ProofType, ScopeSet};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Findings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// A corpus defect. `check` exits 8, which is what CI routes on.
    Fault,
    /// Reported, never a fault. Behavioural signals live here, and so do the
    /// states that are somebody's decision rather than an error.
    Signal,
}

/// One constraint's share of what a perimeter charges, kept as a number rather
/// than as a rendered line.
///
/// A total says the perimeter is expensive; the charge per constraint says
/// *which* constraint is expensive, and that is the only form of the fact
/// anybody can act on (§11). It is a pair and not a sentence because `--json`
/// has to hand it over as data: a caller ranking constraints by cost must not
/// have to parse prose back into integers, which is the same reason `note` was
/// split out of `message`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Charge {
    pub id: String,
    pub characters: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub level: Level,
    pub subject: String,
    pub message: String,
    /// What the finding does not say and the reader needs anyway, printed
    /// under it rather than folded into `message`.
    ///
    /// Separate from the message for three reasons that all point the same way:
    /// a note carries no severity of its own and must not count as a second
    /// finding; `review` filters findings by the opening of `message`, which a
    /// longer sentence would break; and a caller reading `--json` gets the
    /// explanation as data instead of as prose it would have to split.
    pub note: Vec<String>,
    /// The breakdown behind a quantity the message states as a total, largest
    /// first. Empty on every finding that reports no quantity.
    ///
    /// Rendered as lines above the note for a human and as an array for a
    /// caller, so neither reader gets the other one's format: the lines are
    /// derived from these numbers, never the other way round.
    pub charge: Vec<Charge>,
}

impl Finding {
    fn fault(subject: impl std::fmt::Display, message: impl Into<String>) -> Finding {
        Finding {
            level: Level::Fault,
            subject: subject.to_string(),
            message: message.into(),
            note: Vec::new(),
            charge: Vec::new(),
        }
    }
    fn signal(subject: impl std::fmt::Display, message: impl Into<String>) -> Finding {
        Finding {
            level: Level::Signal,
            subject: subject.to_string(),
            message: message.into(),
            note: Vec::new(),
            charge: Vec::new(),
        }
    }
    fn with_note(mut self, lines: Vec<String>) -> Finding {
        self.note = lines;
        self
    }
    fn with_charge(mut self, charge: Vec<Charge>) -> Finding {
        self.charge = charge;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
    pub pruned: Vec<String>,
    pub tasks: usize,
    pub adrs: usize,
    /// Ratification signatures no local key could check. Accumulated across the
    /// corpus and reported once (§4), never per ADR.
    pub unchecked_signatures: usize,
    /// Ratification signatures git refused to answer about at all. Counted
    /// apart from `unchecked_signatures` because the two are different
    /// failures: no key for the answer, versus no answer.
    pub unreadable_signatures: usize,
    /// What git said the first time it refused, so the one corpus line can name
    /// the cause instead of only reporting that there was one.
    pub signature_failure: Option<String>,
    /// How far this checkout's corpus is from the corpus the default branch
    /// carries (§4, ADR-47e2ac102f58).
    ///
    /// `None` is the question never answered, and it is not "nothing has
    /// moved": no repository, no resolvable default branch, a default branch
    /// naming no commit. The cases that deserve a line get one from `inspect`
    /// itself; the field exists so `status` says the same thing out of the same
    /// pass rather than computing a second answer able to disagree.
    pub drift: Option<Drift>,
}

/// The corpus of this checkout against the corpus of the default branch, once
/// the two have actually been compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    /// The branch compared against, so a reader is never left guessing which
    /// one answered.
    pub branch: String,
    /// Entity files held here and not there, there and not here, or held on
    /// both sides with different content. Zero is level, and is a fact worth
    /// printing.
    pub entities: usize,
}

impl Report {
    pub fn faults(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.level == Level::Fault)
            .count()
    }
    pub fn signals(&self) -> usize {
        self.findings.len() - self.faults()
    }
    /// 0 when the corpus is healthy, 8 when it has faults. Never 1, so CI can
    /// tell a sick corpus from a broken tool (§4).
    pub fn exit_code(&self) -> i32 {
        if self.faults() > 0 {
            8
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// check
// ---------------------------------------------------------------------------

pub fn check(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<i32> {
    let path = crate::context::perimeter(inv, repo)?;
    let report = inspect(repo, cfg, path.as_deref(), true)?;
    render(&report, inv, out);
    Ok(report.exit_code())
}

/// Everything mechanical §4 lists, plus the maintenance of §7.
///
/// `prune` is a parameter so `review` can reuse the inspection without touching
/// the coordination plane: reporting is safe from anywhere, deleting is not.
pub fn inspect(repo: &Repo, cfg: &Config, path: Option<&str>, prune: bool) -> Result<Report> {
    let store = Store::new(&repo.ank);
    let mut report = Report::default();
    let mut entities: Vec<(PathBuf, Entity)> = Vec::new();

    // Parsing and canonical form: the corpus is unreadable before it is wrong.
    //
    // Both layouts are walked, the flat one first, and an entity present in
    // both is inspected **once** — from the canonical copy, which is the newer
    // by construction since every write lands there (§6). File-level faults are
    // still reported for each file, because each file exists.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut still_legacy = 0usize;
    // Every entity file the working tree holds, keyed by the id its name
    // carries, canonical copy first. Collected from the name rather than from
    // the parse, because the drift comparison below is about files: a file that
    // does not parse is already a fault, and dropping it from the comparison
    // would report it as absent from a corpus that holds it.
    let mut here: BTreeMap<String, PathBuf> = BTreeMap::new();
    for (kind_of_dir, dirname) in [
        (None, Store::ENTITIES_DIR),
        (Some(EntityKind::Task), "tasks"),
        (Some(EntityKind::Adr), "adr"),
    ] {
        let dir = repo.ank.join(dirname);
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            if let Some(id) = name
                .strip_suffix(".md")
                .filter(|stem| EntityId::parse(stem).is_ok())
            {
                here.entry(id.to_string()).or_insert_with(|| p.clone());
            }
            let Ok(text) = std::fs::read_to_string(&p) else {
                report
                    .findings
                    .push(Finding::fault(&name, "unreadable file"));
                continue;
            };
            // A merge left half-done is a corpus fault long before it is a
            // parse error, and the message has to say so plainly (§7).
            if text
                .lines()
                .any(|l| l.starts_with("<<<<<<< ") || l.starts_with(">>>>>>> ") || l == "=======")
            {
                report
                    .findings
                    .push(Finding::fault(&name, "unresolved git conflict markers"));
                continue;
            }
            match parse_entity(&text) {
                Err(e) => report.findings.push(Finding::fault(&name, format!("{e}"))),
                Ok(entity) => {
                    let canonical = serialize_entity(&entity);
                    if canonical != text {
                        // Two deviations wearing one symptom. If dropping the
                        // carriage returns leaves the canonical form, the
                        // content is right and only git is wrong: nobody wrote
                        // a malformed file, and failing CI over a checkout
                        // setting would be reporting the wrong culprit. A
                        // signal, exit 0. Anything else is a corpus defect.
                        if has_crlf(&text) && canonical == normalise_line_endings(&text) {
                            report.findings.push(Finding::signal(
                                &name,
                                ank_core::Error::CrlfLineEndings.to_string(),
                            ));
                        } else {
                            report.findings.push(Finding::fault(
                                &name,
                                "non-canonical form (round-trip differs)",
                            ));
                        }
                    }
                    if name != format!("{}.md", entity.id()) {
                        report.findings.push(Finding::fault(
                            &name,
                            format!("file name does not carry {}", entity.id()),
                        ));
                    }
                    // The previous layout put the kind in the directory, so a
                    // file in the wrong one contradicts its own name. The flat
                    // layout has no directory to disagree with, and the file
                    // name is the only thing stating the kind — which is what
                    // makes the name-against-id check above load-bearing.
                    if let Some(kind) = kind_of_dir {
                        if entity.id().kind() != kind {
                            report.findings.push(Finding::fault(
                                &name,
                                format!(
                                    "a {} filed under {}",
                                    entity.id().kind().as_str(),
                                    kind.as_str()
                                ),
                            ));
                        }
                    }
                    if kind_of_dir.is_some() {
                        still_legacy += 1;
                    }
                    // One corpus, no entity counted twice.
                    if !seen.insert(entity.id().to_string()) {
                        continue;
                    }
                    entities.push((p, entity));
                }
            }
        }
    }

    // A corpus still in the previous layout is a **signal and never a fault**:
    // it parses, it round-trips, and it answers every verb. Exiting 8 over a
    // file location would redden a pipeline for something no reader suffers
    // from, which is the kind of finding that teaches people to stop reading
    // `check`. Reported once, with the command that moves it.
    if still_legacy > 0 {
        report.findings.push(Finding::signal(
            "corpus",
            format!(
                "{still_legacy} entities are in the previous layout: entities live \
                 in .ank/entities/ since schema 3 (git mv .ank/tasks/*.md \
                 .ank/adr/*.md .ank/entities/)"
            ),
        ));
    }

    let in_scope = |e: &Entity| match path {
        None => true,
        Some(p) => ScopeSet::new(e.scope())
            .map(|s| s.overlaps_dir(p, e.scope()))
            .unwrap_or(false),
    };

    let statuses: HashMap<EntityId, TaskStatus> = entities
        .iter()
        .filter_map(|(_, e)| match e {
            Entity::Task(t) => Some((t.id.clone(), t.status)),
            _ => None,
        })
        .collect();
    let adr_ids: HashSet<EntityId> = entities
        .iter()
        .filter(|(_, e)| e.id().kind() == EntityKind::Adr)
        .map(|(_, e)| e.id().clone())
        .collect();

    report.tasks = statuses.len();
    report.adrs = adr_ids.len();

    // One walk of the tree, reused by every dead-scope test: reading the
    // repository once and matching many globs against it beats walking it per
    // entity, and the corpus is small where the tree is not.
    let files = tracked_files(&repo.root);

    // From here the inspection has two halves, and one of them can be absent
    // (ADR-9307e5d214a7). Everything above is the corpus — parse, canonical
    // form, filename against id, `blocked_by` references, glob validity — and a
    // parser answers all of it. Everything below needs an arbiter: claim refs,
    // the default branch, ratification signatures, completion-ref pruning.
    //
    // Where there is no repository to ask, the second half is skipped and said
    // so, in exactly one line. That line is not optional: a check that silently
    // examines less than it did is how a corpus passes a gate that stopped
    // looking. It is a signal rather than a fault, because a corpus outside a
    // repository is not a sick corpus — it is an inspection with a smaller
    // reach, and the exit code has to keep meaning what §4 says it means.
    // `None` is the half never asked for, and it is distinct from `Some(Err)` —
    // a repository whose default branch cannot be resolved. The two produce one
    // line each and must not produce two between them.
    let has_git = git::usable_here(&repo.root);
    let (coord, detached, default_branch) = if has_git {
        let (coord, detached) = coordination(&repo.root, &mut report)?;
        let branch = git::resolve_default_branch(
            cfg.default_branch.as_deref(),
            git::origin_head(&repo.root)?.as_deref(),
        );
        check_signers(repo, &mut report);
        (coord, detached, Some(branch))
    } else {
        report.findings.push(Finding::signal(
            "coordination",
            "half skipped: no git repository here, so claim refs, ratification \
             signatures, completion refs and detached proofs were neither read \
             nor judged (git init to coordinate)",
        ));
        (HashMap::new(), HashMap::new(), None)
    };

    for (_, entity) in &entities {
        if !in_scope(entity) {
            continue;
        }
        check_scope_alive(
            entity,
            &files,
            has_git.then_some(repo.root.as_path()),
            &mut report,
        );
        match entity {
            Entity::Task(t) => check_task(
                t,
                repo,
                &statuses,
                &coord,
                detached.get(&t.id).map(Vec::as_slice).unwrap_or(&[]),
                cfg,
                &store,
                default_branch.as_ref().and_then(|b| b.as_deref().ok()),
                &mut report,
            ),
            Entity::Adr(a) => check_adr(a, repo, &adr_ids, &entities, &mut report),
        }
    }

    check_cycles(&entities, &mut report);
    check_authorship(&entities, &coord, &in_scope, &mut report);

    // One line for the machine, not one per ADR (§8, and the rule of §4 about
    // reporting a corpus-wide absence once). Says what it would take to make
    // the answer real, because the reader who sees this is usually one
    // `gpg --recv-keys` away from a verification.
    if report.unchecked_signatures > 0 {
        let n = report.unchecked_signatures;
        report.findings.push(Finding::signal(
            "allowed_signers",
            format!(
                "{n} ratification signature(s) could not be checked here: no public key \
                 for the signing identity, so they are not verified and not refused \
                 (import the key declared in .ank/allowed_signers)"
            ),
        ));
    }

    // Same rule, other cause: git was asked and did not answer. Silence here
    // was the defect — an ADR whose signature could not be read used to look
    // exactly like one in a corpus that declares no key at all.
    if report.unreadable_signatures > 0 {
        let n = report.unreadable_signatures;
        let why = report.signature_failure.as_deref().unwrap_or("git failed");
        report.findings.push(Finding::signal(
            "allowed_signers",
            format!(
                "{n} ratification signature(s) could not be read, so they are \
                 neither verified nor refused: {why}"
            ),
        ));
    }

    // Maintenance last, so a corpus fault is still reported when pruning cannot
    // run for want of a default branch.
    match &default_branch {
        Some(Ok(branch)) => {
            // The corpus before the plane: the question a reader asks first is
            // whether the corpus they are looking at is the one everybody else
            // reads (ADR-47e2ac102f58).
            corpus_drift(repo, branch, &here, &mut report);
            maintain(repo, branch, &coord, &statuses, prune, &mut report)?;
            maintain_proofs(repo, branch, &detached, &statuses, prune, &mut report)?;
        }
        Some(Err(_)) => report.findings.push(Finding::signal(
            "coordination",
            "default branch indeterminable, completion refs neither pruned nor judged \
             (ank config default_branch <name>)",
        )),
        // The coordination half was skipped, and it has already said so once.
        // A second line here would report the consequence as if it were a
        // separate finding.
        None => {}
    }

    report.findings.sort_by(|a, b| {
        a.level
            .cmp(&b.level)
            .then(a.subject.cmp(&b.subject))
            .then(a.message.cmp(&b.message))
    });
    Ok(report)
}

type Plane = (
    HashMap<EntityId, Record>,
    HashMap<EntityId, Vec<claim::AttestedProof>>,
);

fn coordination(cwd: &Path, report: &mut Report) -> Result<Plane> {
    let mut map = HashMap::new();
    let mut proofs: HashMap<EntityId, Vec<claim::AttestedProof>> = HashMap::new();
    for r in git::ank_refs(cwd)? {
        // One walk over both namespaces. `check` asks the same question of
        // every ref under `refs/ank/`, and two loops would be free to disagree
        // about which of them a given ref belongs to.
        let (rest, proof_ns) = match (
            r.name.strip_prefix(claim::CLAIMS_PREFIX),
            r.name.strip_prefix(claim::PROOF_PREFIX),
        ) {
            (Some(rest), _) => (rest, false),
            (_, Some(rest)) => (rest, true),
            _ => continue,
        };
        let Ok(id) = EntityId::parse(rest) else {
            // A ref in one of these namespaces whose tail is not an identifier
            // is an orphan by construction: nothing will ever address it.
            report
                .findings
                .push(Finding::fault(&r.name, "ref name is not an identifier"));
            continue;
        };
        match claim::read_at(cwd, &r.name) {
            Ok(Some(held)) => match (held.record, proof_ns) {
                (Record::Proof(p), true) => {
                    proofs.insert(id, p.proofs);
                }
                (record, false) if !matches!(record, Record::Proof(_)) => {
                    map.insert(id, record);
                }
                // A record whose state contradicts its namespace. Named rather
                // than coerced: read as the other kind it would either present
                // a held task as free or lose an attestation, and both are the
                // silent fallback this file refuses everywhere else.
                _ => report.findings.push(Finding::fault(
                    &r.name,
                    "record of the wrong kind for its namespace",
                )),
            },
            Ok(None) => {}
            Err(e) => report.findings.push(Finding::fault(&r.name, e.message)),
        }
    }
    Ok((map, proofs))
}

/// One entry of `.ank/allowed_signers`: an identity allowed to ratify (§8).
///
/// Git's allowed-signers layout, `principal [options] keytype key`, read from
/// the end because the optional middle is what varies: the key is the last
/// field and its type the one before, whatever sits between them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signer {
    pub principal: String,
    pub keytype: String,
    pub key: String,
}

/// Parses `allowed_signers`. Unreadable lines are dropped rather than reported:
/// this file is versioned and reviewed, and a check that refuses to start over a
/// stray line is a check that stops answering the question it exists for.
pub fn parse_signers(text: &str) -> Vec<Signer> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let fields: Vec<&str> = l.split_whitespace().collect();
            match fields.as_slice() {
                [principal, .., keytype, key] => Some(Signer {
                    principal: (*principal).to_string(),
                    keytype: keytype.to_lowercase(),
                    key: (*key).to_string(),
                }),
                _ => None,
            }
        })
        .collect()
}

/// What a ratification commit's signature is worth.
///
/// Six states and not three, because collapsing any two of them loses the
/// distinction the check exists to draw. `Unchecked` in particular must never
/// read as `Trusted`: a verification that degrades to success on a machine
/// missing a public key is not a verification. `Unreadable` is the same rule
/// applied to the error path, where the degradation was to silence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    /// Good signature from a key `allowed_signers` declares.
    Trusted,
    /// Good signature, from a key nobody declared.
    Undeclared { fingerprint: String },
    /// A signature is there and this machine cannot check it: no local key, or
    /// no working gpg to ask (TASK-f4ed2020c964).
    Unchecked,
    /// No signature at all — the forgery this task was filed for.
    ///
    /// Reserved for a commit that carries none. It is the strongest negative
    /// verdict there is, and it must never be reached by a machine that merely
    /// failed to look.
    Absent,
    /// Present, checkable, and refused: bad, expired or revoked.
    Invalid { status: char },
    /// Git could not be asked at all: it exited non-zero instead of answering.
    /// Not a verdict about the commit — a verdict about this machine.
    ///
    /// The one state `classify_signature` never produces, because it is the
    /// absence of the facts that function classifies. It exists so that a
    /// failure to ask is something rather than nothing (TASK-c92b7cc10f13).
    Unreadable { reason: String },
}

/// Turns git's facts into the verdict, against the declared keys.
///
/// Pure, and that is deliberate: every state is reachable in a unit test
/// without a keyring, a network or a signed fixture, which is what makes "each
/// one is tested" affordable rather than aspirational.
///
/// `carries_signature` is the fact `%G?` cannot express, and it only decides the
/// `N` case — see [`commit_carries_signature`] for why `N` is two states wearing
/// one letter.
pub fn classify_signature(
    facts: &git::SignatureFacts,
    declared: &[Signer],
    carries_signature: bool,
) -> Signature {
    match facts.status {
        // `N` is git saying it has no good signature to report, which covers
        // both a commit nobody signed and a signature it could not attempt.
        // The object decides between them.
        'N' if carries_signature => Signature::Unchecked,
        'N' => Signature::Absent,
        'E' => Signature::Unchecked,
        'G' | 'U' => {
            if is_ssh(&facts.fingerprint) {
                // Under SSH git has already done the allowlist check, because
                // it is the only format it will do it for: measured against
                // git, `G` is a key the allowed-signers file matched and `U` is
                // a good signature whose principal it did not. Re-deciding it
                // here would mean comparing an `SHA256:` fingerprint against a
                // base64 key and calling every correct signature undeclared.
                if facts.status == 'G' {
                    Signature::Trusted
                } else {
                    Signature::Undeclared {
                        fingerprint: facts.fingerprint.clone(),
                    }
                }
            } else if declares(declared, &facts.fingerprint) {
                // OpenPGP, where git never reads the file at all. `U` here is a
                // good signature from a key the local keyring does not
                // *ultimately trust*, which is a statement about the operator's
                // web of trust and not about this repository — so both good
                // statuses are judged the same way, by the declaration.
                Signature::Trusted
            } else {
                Signature::Undeclared {
                    fingerprint: facts.fingerprint.clone(),
                }
            }
        }
        other => Signature::Invalid { status: other },
    }
}

/// An SSH signature names its key as `SHA256:<base64>`; OpenPGP names it as a
/// bare hex fingerprint. Which of the two decided the allowlist depends on it.
fn is_ssh(fingerprint: &str) -> bool {
    fingerprint.contains(':')
}

/// Whether any declared key names this fingerprint.
///
/// A suffix match, not equality: git reports a full 40-hex OpenPGP fingerprint
/// while a file may perfectly reasonably declare the 16-hex long key id, and
/// the long id is the tail of the fingerprint. The comparison is
/// case-insensitive because hex is written both ways and neither is wrong.
fn declares(declared: &[Signer], fingerprint: &str) -> bool {
    let fp = fingerprint.replace(' ', "").to_lowercase();
    if fp.is_empty() {
        return false;
    }
    declared.iter().any(|s| {
        let key = s.key.replace(' ', "").to_lowercase();
        // The SSH case is already decided by git, which refuses the signature
        // outright when the allowed-signers file does not cover it; comparing
        // an `SHA256:` fingerprint against a base64 key here would fail on a
        // signature git already accepted.
        !key.is_empty() && (fp == key || fp.ends_with(&key))
    })
}

fn declared_signers(repo: &Repo) -> Vec<Signer> {
    let text = std::fs::read_to_string(repo.ank.join("allowed_signers")).unwrap_or_default();
    parse_signers(&text)
}

/// §8: with no signing configured, permissions are advisory. Displayed rather
/// than hidden, and once rather than once per entity.
fn check_signers(repo: &Repo, report: &mut Report) {
    let keys = declared_signers(repo).len();
    if keys == 0 {
        report.findings.push(Finding::signal(
            "allowed_signers",
            "no ratification key declared: permissions are advisory, not enforced (§8)",
        ));
    }
}

/// Every file the repository holds, relative and `/`-separated. `.git` and
/// `target` are skipped: neither is ever in a scope, and both would dominate
/// the walk.
fn tracked_files(root: &Path) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<String>, depth: usize) {
        if depth > 24 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git" || name == "target" || name == "node_modules" {
                continue;
            }
            if p.is_dir() {
                walk(root, &p, out, depth + 1);
            } else if let Ok(rel) = p.strip_prefix(root) {
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out, 0);
    out
}

/// Structural death (§11): a scope matching no file. Verifiable, unlike
/// temporal decay — a three-year-old constraint can be vital. Never acted on
/// automatically: the code may simply have moved.
///
/// `git_root` is `Some` only where there is a repository to ask, and it is what
/// turns "the scope matches nothing" into "the file moved here, and this is the
/// command that follows it" (ADR-97beaf55e73a). Where it is `None` the walk is
/// skipped in silence: a corpus outside a repository already says so once, in
/// the coordination line, and a second sentence about a question nobody could
/// ask would be noise.
fn check_scope_alive(
    entity: &Entity,
    files: &[String],
    git_root: Option<&Path>,
    report: &mut Report,
) {
    let globs = entity.scope();
    if globs.is_empty() {
        report
            .findings
            .push(Finding::fault(entity.id(), "no scope: attached to nothing"));
        return;
    }
    // Fault for a constraint, signal for work still to come. The asymmetry is
    // not a concession: an ADR's scope states what existing code it binds, so
    // one matching nothing binds nothing, and that is a defect. A task's scope
    // states where work *will* happen, and a task scoping a file it is about to
    // create is the normal case — this repository's own release task scopes a
    // workflow it exists to write. Making that a fault would exit 8 in CI on
    // every repository that plans anything. A task already done or closed is
    // judged the other way: it claimed to touch files that are not there.
    let ahead_of_the_code = matches!(
        entity,
        Entity::Task(t) if matches!(t.status, TaskStatus::Open | TaskStatus::InProgress)
    );
    for glob in globs {
        let Ok(one) = ScopeSet::new(std::slice::from_ref(glob)) else {
            report.findings.push(Finding::fault(
                entity.id(),
                format!("invalid glob '{glob}'"),
            ));
            continue;
        };
        if files.iter().any(|f| one.matches(f)) {
            continue;
        }
        // The cost clause of ADR-97beaf55e73a, and the reason this sits here
        // rather than above the loop: two git processes per glob, paid only by a
        // glob that already matches nothing. A healthy corpus reaches this line
        // no times.
        let mut note = match git_root {
            Some(root) => scope_moved(entity, glob, root),
            None => Vec::new(),
        };
        // The severity rule, and it only ever lowers. A dead scope git can
        // explain is not a broken corpus: the reader can see where the path went
        // and follow it, which is what the walk above was built to show. The
        // fault is for the death git cannot explain, where the reader has
        // nothing — and keeping it for the rest would mean any directory rename
        // reddens a corpus permanently, since `amend` refuses a finished task and
        // the finding then names no act at all.
        //
        // `ahead_of_the_code` is read first and alone, so a signal never becomes
        // a fault here whatever git says.
        let explained = !note.is_empty();
        // The third state (§4). A shallow clone holds no commit that could
        // record the rename, so "git recorded none" and "there is no history to
        // ask" are different answers, and only the first is evidence. Faulting
        // on the second makes the health of a corpus depend on how it was
        // cloned — measured on this repository's own pipeline, where a
        // depth-1 checkout turned six signals into six faults with no note at
        // all. Same answer §3 gives a ratification anchor a shallow clone
        // cannot verify, and the same reasoning.
        //
        // Asked only here: a corpus with nothing dead never reaches this line,
        // and the answer is memoised so a corpus with eight dead scopes asks
        // once.
        let unverifiable = !explained && git_root.is_some_and(git::is_shallow);
        if unverifiable {
            note.push(
                "the history here is shallow, so where it went cannot be verified \
                 (git fetch --unshallow)"
                    .to_string(),
            );
        }
        let finding = if ahead_of_the_code {
            Finding::signal(
                entity.id(),
                format!("scope '{glob}' matches no file yet: work not started, or a typo"),
            )
        } else {
            let message = format!("dead scope '{glob}': no file matches it");
            match explained || unverifiable {
                true => Finding::signal(entity.id(), message),
                false => Finding::fault(entity.id(), message),
            }
        };
        report.findings.push(finding.with_note(note));
    }
}

/// The note under a dead scope: where git says the path went, and what repairs
/// the entity (ADR-97beaf55e73a).
///
/// Empty for everything git cannot explain, which is most of what can go wrong
/// with a scope. **Silence here is not evidence.** A deletion, a move under the
/// similarity threshold and a typo that never named a real file all produce the
/// same nothing, so no wording below may suggest which — the reader is left
/// exactly where they stand today, and the finding above says all that is known.
fn scope_moved(entity: &Entity, glob: &str, root: &Path) -> Vec<String> {
    // A git failure is not a corpus fault and must never become one: the scope
    // is dead either way, and that is already reported. What is lost is the
    // explanation, which is exactly what `None` means everywhere else here.
    let (asked, moved, to) = match literal_prefix(glob) {
        // A path. The common entry, and the one git answers directly: 343 of the
        // 462 scope entries in this repository's own corpus name one file with
        // no wildcard.
        None => {
            let Ok(Some(moved)) = git::rename_of(root, glob) else {
                return Vec::new();
            };
            let to = moved.to.clone();
            (glob.to_string(), moved, to)
        }
        // A glob. git has no answer for "where did `src/**` go", so the question
        // put to it is about the literal prefix, and the wildcard tail is carried
        // across to the proposal unchanged.
        Some((prefix, tail)) => {
            let Ok(Some(moved)) = git::directory_rename_of(root, &prefix) else {
                return Vec::new();
            };
            let to = format!("{}{tail}", moved.to);
            (prefix, moved, to)
        }
    };
    let mut note = vec![format!(
        "git records {asked} renamed to {} in {}",
        moved.to, moved.sha
    )];
    note.extend(repair(entity, glob, &to));
    note
}

/// The part of a glob git can be asked about, and the wildcard tail that is not.
///
/// `None` for a glob with no wildcard, which is a path and belongs to
/// [`git::rename_of`]. `None` too when nothing literal precedes the first
/// wildcard — `**/foo` names no directory to ask about, and guessing one would
/// be the invented answer this whole path refuses to give.
///
/// The cut is at the last separator before the first wildcard, so `src/*.rs`
/// asks about `src` and never about `src/`: a partial path component is not a
/// directory, and `rev-list` would answer about nothing.
fn literal_prefix(glob: &str) -> Option<(String, String)> {
    let first = glob.find(['*', '?', '[', ']', '{', '}'])?;
    let cut = glob[..first].rfind('/')?;
    Some((glob[..cut].to_string(), glob[cut..].to_string()))
}

/// The one command that changes the scope of this entity without refusing on
/// the spot.
///
/// Four states and three answers, because `amend` accepts a scope change on
/// exactly two of them. The refusals it would otherwise raise are what decides
/// each branch, and each is a refusal this file writes itself.
fn repair(entity: &Entity, from: &str, to: &str) -> Option<String> {
    let id = entity.id();
    let amend = format!("ank amend {id} --drop-scope \"{from}\" --scope \"{to}\"");
    match entity {
        // A plan still in flight: `amend` is the verb, and it is what this
        // reader wants — the rename is what tells a typo from a file that moved
        // under the work.
        Entity::Task(t) if matches!(t.status, TaskStatus::Open | TaskStatus::InProgress) => {
            Some(amend)
        }
        // Done or closed, and `amend` refuses it: §3 allows one write to a
        // finished task and it is a proof. The scope records where the work
        // happened, which is a fact about the past that a later rename does not
        // falsify — so the rename is named and nothing is proposed. Naming a
        // command that exits 7 would be worse than naming none.
        Entity::Task(_) => None,
        Entity::Adr(a) if a.status == AdrStatus::Proposed => Some(amend),
        // Accepted or superseded: `constraint` and `scope` are hashed into the
        // ratification commit, so `amend` refuses with code 6 and the change is
        // a succession. Worded as that refusal words it, and deliberately not
        // shortened — a supersession is a decision, and a one-flag command
        // would read as a formality.
        Entity::Adr(_) => Some(format!(
            "ank new adr --supersedes {id} --title \"<t>\" --scope \"{to}\" \
             --constraint \"<rule>\""
        )),
    }
}

fn check_task(
    t: &Task,
    repo: &Repo,
    statuses: &HashMap<EntityId, TaskStatus>,
    coord: &HashMap<EntityId, Record>,
    detached: &[claim::AttestedProof],
    cfg: &Config,
    store: &Store,
    default_branch: Option<&str>,
    report: &mut Report,
) {
    // Every proof against this task, from both sources. ADR-493471d64ba0 is
    // explicit that a signal counting proofs counts both, "or it fires on work
    // that is anchored" — and it would have: between a `done` landing and its
    // merge, the ref is the only place a CI reference exists, which is most of
    // a branch's life.
    let proofs: Vec<&ank_core::Proof> = t
        .proof
        .iter()
        .chain(detached.iter().map(|a| &a.proof))
        .collect();
    for b in &t.blocked_by {
        match statuses.get(b) {
            None => report.findings.push(Finding::fault(
                &t.id,
                format!("blocked_by names {b}, which does not exist"),
            )),
            // `closed` does not unblock: the work was not carried out, the
            // dependents stay blocked, and a human decides (§3).
            Some(TaskStatus::Closed) => report.findings.push(Finding::signal(
                &t.id,
                format!("blocked by {b}, which is closed: close down the chain or rewrite it"),
            )),
            _ => {}
        }
    }

    // The criterion set by whoever claimed the task rather than by whoever
    // created it. Not forbidden, and visible.
    if t.criteria_by == Some(CriteriaBy::Claimer) {
        report.findings.push(Finding::signal(
            &t.id,
            "done_criteria set by the claimer, not by the creator",
        ));
    }

    if t.status == TaskStatus::InProgress && t.done_criteria.is_none() {
        report
            .findings
            .push(Finding::fault(&t.id, "in progress with no done_criteria"));
    }
    if t.status == TaskStatus::Done && proofs.is_empty() {
        report
            .findings
            .push(Finding::fault(&t.id, "done with no proof"));
    }

    // A status that says held, with no record to hold it. The arm below already
    // notices the neighbouring case — a claim present but expired — and the two
    // are the same invariant read from opposite sides: only one of them was
    // implemented, so a claim that lapsed and had its ref removed left the file
    // asserting live work that nobody was doing. `context` lists such a task and
    // offers it to no one, which is the worst of both.
    //
    // A signal and never a fault. The corpus is intact; the record is stale.
    // Exiting 8 over it would teach a reader that 8 fires for things that do not
    // matter, and an exit code people learn to ignore is worse than none.
    if t.status == TaskStatus::InProgress && !coord.contains_key(&t.id) {
        report.findings.push(Finding::signal(
            &t.id,
            format!(
                "in_progress with no claim ref: nothing holds it (ank claim {})",
                t.id
            ),
        ));
    }

    // The freeze, checked at the point of use. The anchor is the claim record,
    // which the file's editor does not control (ADR-6b3f19e08a24).
    if let Some(Record::Claim(c)) = coord.get(&t.id) {
        if let Some(criteria) = &t.done_criteria {
            if !verify_frozen(criteria, &c.criteria) {
                report.findings.push(Finding::fault(
                    &t.id,
                    format!(
                        "done_criteria diverges from the claim (claimed {}, now {})",
                        c.criteria,
                        freeze::freeze_hash_short(criteria)
                    ),
                ));
            }
        }
        if claim::is_expired(c, claim::now_secs(), &t.id).unwrap_or(false) {
            report.findings.push(Finding::signal(
                &t.id,
                format!("claim by {} expired: the task is claimable again", c.holder),
            ));
        }
        // A constraint accepted while the work is in progress changes what
        // applies to it. `done` warns; so does this.
        if let Ok(applicable) = claim::applicable_constraints(store, repo, t) {
            if claim::constraints_hash(&applicable) != c.constraints {
                report.findings.push(Finding::signal(
                    &t.id,
                    "applicable constraints changed since the claim: re-read ank context",
                ));
            }
        }
    }

    // The statement this signal makes is about the task, not about the entry:
    // *its completion rests on nothing verifiable*. That is false the moment a
    // strong proof sits beside the weak one, so the condition belongs here and
    // not inside the loop.
    //
    // Read on the entry, it was a finding nobody could act on. §3 makes the
    // proof list append-only and ADR-85e6bbb195b8 forbids rewriting an entry to
    // make history look better, so a task closed before `ank done` existed could
    // never clear it: the assertion has to stay, and the assertion was what
    // fired. A line every reader learns to skip is worse than no line.
    if !proofs.is_empty() && proofs.iter().all(|p| p.proof_type.is_weak()) {
        // The first weak entry names the kind, which is what a reader acts on;
        // one finding per task, because the task is what is being judged.
        let kind = proofs[0].proof_type.as_str();
        report.findings.push(Finding::signal(
            &t.id,
            format!("weak proof '{kind}': it anchors nothing"),
        ));
    }

    // Attestation is the half of `done` nothing used to notice. `done` records
    // what ran on the machine that ran it; `attest` is what anchors the same
    // criterion to a run anybody can re-read. A task that never got the second
    // one rests entirely on a local claim, and the omission is invisible: the
    // task reads `done`, the proof list is non-empty, and no existing finding
    // fires — `commit` is not weak, so the check above stays silent by design.
    //
    // Gated on the default branch, and that gate is load-bearing rather than
    // decoration. On a feature branch straight after `done` the attestation
    // cannot exist yet — no merge run has happened — so reporting there would
    // name work the reader is unable to do, which is the failure the weak-proof
    // comment above was written about. The window where `main` carries the task
    // and the run is still going green is left to fire on purpose: the
    // statement is true when printed and clears when someone attests, and
    // buying that quiet would cost a grace constant §6 only justifies for the
    // flooding thresholds.
    if t.status == TaskStatus::Done
        && !proofs.is_empty()
        && !proofs.iter().any(|p| p.proof_type == ProofType::Test)
        && done_on(repo, default_branch, &t.id)
    {
        report.findings.push(Finding::signal(
            &t.id,
            format!(
                "done with no test proof: nothing external anchors it \
                 (ank attest {} --proof test:<run-id>)",
                t.id
            ),
        ));
    }

    for p in &proofs {
        // What ran is anchored in the proof, not in the current state of
        // config.yml. A verifier weakened in any commit shows up here.
        if let Some((name, hash)) = p.verifier.as_ref().and_then(|v| v.split_once('@')) {
            match cfg.verifier(name) {
                None => report.findings.push(Finding::signal(
                    &t.id,
                    format!("proof anchors verifier '{name}', absent from config.yml"),
                )),
                Some(def) if verify::definition_hash(def) != hash => {
                    report.findings.push(Finding::signal(
                        &t.id,
                        format!(
                            "verifier '{name}' changed since the proof ({hash} -> {})",
                            verify::definition_hash(def)
                        ),
                    ));
                }
                Some(_) => {}
            }
        }
    }

    // The field is declarative and git is the anchor. A creation in the future
    // is the half of that check reachable without porcelain (§4).
    if let Some(created) = claim::parse_utc(&t.created) {
        if created > claim::now_secs() + 86_400 {
            report.findings.push(Finding::signal(
                &t.id,
                format!("created in the future ({})", t.created),
            ));
        }
    }

    // Over-constrained (§5): constraints alone eating more than half the budget
    // in execution mode. A corpus problem, not a display problem.
    //
    // **One variable for the threshold tested and the threshold reported.** It
    // used to test `weight * 2 > context_budget` and report the budget, so the
    // message read `5527 characters of constraint against a budget of 8000` --
    // arithmetic no reader can believe. Every reader who checked it concluded
    // the tool was miscounting, and one went to the source to find out that the
    // limit is half of that (TASK-9ff86a0950bf). The two numbers cannot
    // disagree again while they come from the same binding, which is the fix;
    // rewording alone would have left the next edit free to separate them.
    //
    // **The total is the diagnosis and the breakdown is the treatment.** A
    // number alone names a problem with no path out of it: there is no verb
    // that splits a scope, and nothing in `14737 characters` says which of the
    // constraints to stop matching. The charge per constraint is the missing
    // fact, and it is also what turns §11 from an argument into a measurement —
    // "mechanise this one first" is a reading of the list, not a preference.
    //
    // Silent under the limit, breakdown and all. A per-constraint listing
    // printed on a healthy perimeter is the volume problem this project keeps
    // refusing, and the reader who wants it on a healthy scope has `ank
    // context`.
    if matches!(t.status, TaskStatus::Open | TaskStatus::InProgress) {
        if let Ok(applicable) = claim::applicable_constraints(store, repo, t) {
            let weight: usize = applicable.iter().map(|(_, c)| c.chars().count()).sum();
            let limit = cfg.context_budget / 2;
            if weight > limit {
                let mut charge: Vec<Charge> = applicable
                    .iter()
                    .map(|(id, c)| Charge {
                        id: id.clone(),
                        characters: c.chars().count(),
                    })
                    .collect();
                // Descending by cost, and by id under a tie so two runs on one
                // corpus print one order. `applicable_constraints` already
                // sorted by id, and a stable sort on the cost alone would have
                // kept that — but "already sorted upstream" is not a property
                // to depend on from here.
                charge.sort_by(|a, b| b.characters.cmp(&a.characters).then(a.id.cmp(&b.id)));
                report.findings.push(
                    Finding::signal(
                        &t.id,
                        format!(
                            "over-constrained scope: {weight} characters of constraint against a \
                             limit of {limit}, half of context_budget"
                        ),
                    )
                    .with_note(relief(store, t, charge.first()))
                    .with_charge(charge),
                );
            }
        }
    }
}

/// What the reader of an over-constrained scope can actually do, and what they
/// cannot.
///
/// Two lines at most, and the split matters: the first names an act on the
/// perimeter, which is the entity this finding is about and the only one still
/// open to an edit; the second names the heaviest constraint and says plainly
/// that it is not amendable, rather than printing a command that would exit 6.
/// Same doctrine as [`repair`] — naming a refusal is worse than naming nothing,
/// because a reader who runs it learns the tool is wrong rather than that the
/// entity is settled.
///
/// The constraint is accepted in every case that reaches here: `context` binds
/// accepted ADRs and nothing else, so a proposed one is charged against no
/// perimeter and cannot appear in the breakdown. The status is read back from
/// the store anyway rather than assumed, because the day that filter changes is
/// the day this would start naming a refusal in silence.
fn relief(store: &Store, t: &Task, heaviest: Option<&Charge>) -> Vec<String> {
    // Dropping the last glob is refused: an entity with no scope attaches to
    // nothing and nobody finds it again. So a one-glob perimeter is narrowed by
    // replacement, and the glob to replace is named because there is only one
    // it could be.
    let mut note = vec![match t.scope.as_slice() {
        [only] => format!(
            "narrow the perimeter, which is one glob and cannot empty: \
             ank amend {} --scope \"<narrower>\" --drop-scope \"{only}\"",
            t.id
        ),
        _ => format!(
            "narrow the perimeter: ank amend {} --drop-scope \"<glob>\"",
            t.id
        ),
    }];
    let Some(top) = heaviest else {
        return note;
    };
    let accepted = EntityId::parse(&top.id)
        .ok()
        .and_then(|id| store.load(&id).ok())
        .is_some_and(
            |loaded| matches!(loaded.entity, Entity::Adr(a) if a.status == AdrStatus::Accepted),
        );
    note.push(match accepted {
        true => format!(
            "{} costs the most, and is accepted: its constraint is anchored in the \
             ratification commit, so no amend reaches it — what lowers its charge is \
             mechanising it out of injected context",
            top.id
        ),
        false => format!(
            "{} costs the most: ank amend {} --drop-scope \"<glob>\" narrows what it binds",
            top.id, top.id
        ),
    });
    note
}

fn check_adr(
    a: &Adr,
    repo: &Repo,
    adr_ids: &HashSet<EntityId>,
    entities: &[(PathBuf, Entity)],
    report: &mut Report,
) {
    if let Some(target) = &a.supersedes {
        if !adr_ids.contains(target) {
            report.findings.push(Finding::fault(
                &a.id,
                format!("supersedes {target}, which does not exist"),
            ));
        } else {
            // A broken chain: the replacement claims the succession and the
            // replaced one never learned of it.
            //
            // Only once the replacement is real, though. `proposed` states an
            // intention, and the succession happens at `accept` — which is what
            // marks the target. Faulting before then would exit 8 over an
            // intention, and exit 8 fails the `check-repo` verifier nearly every
            // task in this corpus declares: writing `new adr --supersedes`, an
            // act the role table hands to the agent, would block every `done` in
            // the repository until a human ratified it. The consequence is out
            // of all proportion to the state it describes.
            let replaced = entities.iter().find_map(|(_, e)| match e {
                Entity::Adr(other) if &other.id == target => Some(other.status),
                _ => None,
            });
            if replaced != Some(AdrStatus::Superseded) {
                let message = format!("supersedes {target}, which is not marked superseded");
                report.findings.push(if a.status == AdrStatus::Proposed {
                    Finding::signal(&a.id, format!("{message} (proposed: not yet a succession)"))
                } else {
                    Finding::fault(&a.id, message)
                });
            }
        }
    }
    if a.status == AdrStatus::Superseded
        && !entities.iter().any(
            |(_, e)| matches!(e, Entity::Adr(other) if other.supersedes.as_ref() == Some(&a.id)),
        )
    {
        report.findings.push(Finding::fault(
            &a.id,
            "marked superseded but no ADR supersedes it",
        ));
    }
    if a.status == AdrStatus::Accepted && a.ratified.is_none() {
        // A signal and not a fault: the ADRs predating `accept` are ratified by
        // the repository's history, which allowed_signers records as the
        // bootstrap exception. Making it a violation would condemn a whole
        // corpus at once and block every `done` behind it.
        report.findings.push(Finding::signal(
            &a.id,
            "accepted with no ratification commit (bootstrap, or accepted by hand)",
        ));
    }
    match freeze_state(repo, a) {
        Freeze::Altered { ratified, now } => report.findings.push(Finding::fault(
            &a.id,
            format!(
                "altered since ratification (ratified {ratified}, now {now}): \
                 its constraint is no longer injected"
            ),
        )),

        // Not a fault, and the distinction is the point. An unreachable
        // ratification commit is a shallow clone or a rewritten history, not a
        // broken freeze — and a check that cries divergence over a shallow clone
        // is a check people learn to ignore.
        Freeze::Unverifiable => report.findings.push(Finding::signal(
            &a.id,
            "ratified, but no ratification commit is reachable: the freeze cannot be verified",
        )),

        Freeze::Unanchored | Freeze::Intact => {}
    }

    // The anchor is worth exactly what the signature on the commit carrying it
    // is worth (§8). An anchor read from a commit nobody signed anchors nothing
    // against the one case the whole mechanism exists for.
    match signature_state(repo, a) {
        Some(Signature::Absent) => report.findings.push(Finding::fault(
            &a.id,
            "its ratification commit is not signed: the anchor proves nothing (§8)",
        )),
        Some(Signature::Invalid { status }) => report.findings.push(Finding::fault(
            &a.id,
            format!("its ratification commit carries a signature git refuses ({status}): not a ratification"),
        )),
        Some(Signature::Undeclared { fingerprint }) => report.findings.push(Finding::fault(
            &a.id,
            format!(
                "ratified by {fingerprint}, which .ank/allowed_signers does not declare"
            ),
        )),

        // Counted, not reported here. A missing public key is a property of
        // the machine and not of this ADR, so it is one line for the corpus
        // rather than one per entity — the same rule §4 already applies to the
        // entities predating `author`, and for the same reason: a line per file
        // is the volume that teaches a reader to stop reading `check`. It is
        // still never silence, because a verification that degrades to success
        // is not a verification.
        Some(Signature::Unchecked) => report.unchecked_signatures += 1,

        // Counted for the same reason, and kept apart from `Unchecked` because
        // they say different things: one machine holds no key for a signature
        // that is there, the other could not look. Neither is a fault — a
        // broken environment is not a forged ratification, and reporting one as
        // the other is how a finding becomes noise. But it is never nothing.
        Some(Signature::Unreadable { reason }) => {
            report.unreadable_signatures += 1;
            report.signature_failure.get_or_insert(reason);
        }

        Some(Signature::Trusted) | None => {}
    }
    if a.constraint.trim().is_empty() {
        report
            .findings
            .push(Finding::fault(&a.id, "no constraint: it binds nothing"));
    }
}

/// More than this many entities by one `author` inside [`BURST_WINDOW`] is a
/// burst (§4).
///
/// High enough that a session filing the four tasks of a plan passes in
/// silence, low enough that a runaway loop is named within minutes. A constant
/// and not a config key: a repository able to raise its own flooding threshold
/// has a threshold that will be raised the first time it fires, and the signal
/// costs nothing to ignore — which is what makes it safe to leave unadjustable.
const BURST_COUNT: usize = 10;
const BURST_WINDOW: i64 = 3600;

/// The three signals that need `author` (§4).
///
/// Two of them were unreachable for want of the field, and the third is what
/// keeps their silence honest: an entity written before `author` existed is
/// skipped by both, so the corpus says how many such entities it holds — once,
/// never per file. One line each would add a line for every entity predating
/// the field, which is the volume that teaches a reader to stop reading
/// `check`.
fn check_authorship(
    entities: &[(PathBuf, Entity)],
    coord: &HashMap<EntityId, Record>,
    in_scope: &dyn Fn(&Entity) -> bool,
    report: &mut Report,
) {
    let considered: Vec<&Entity> = entities
        .iter()
        .map(|(_, e)| e)
        .filter(|e| in_scope(e))
        .collect();

    // Skipped, and said so once. `None` means the entity predates the field and
    // never that nobody wrote it: the author of a file that already exists
    // cannot be recovered, since git would name whoever committed it — a
    // different fact about a possibly different person — and ADR-b8884edcebe3
    // forbids the porcelain that would ask.
    let authorless = considered.iter().filter(|e| author_of(e).is_none()).count();
    if authorless > 0 {
        report.findings.push(Finding::signal(
            "corpus",
            format!(
                "{authorless} entities predate the author field: the burst and \
                 self-blocking signals skip them"
            ),
        ));
    }

    // The actor convention, reported **once for the corpus and never per file**
    // — the same choice as the line above and for the same reason: one line per
    // file adds a line for every entity written before the rule existed, which
    // is the volume that teaches a reader to stop reading `check`.
    //
    // A finding here and never a parse error (ADR-3877fef1d662). The corpus is
    // not migrated by a rule it predates, and refusing these files would lock
    // them out of their own format.
    let untyped = considered
        .iter()
        .filter_map(|e| author_of(e))
        .filter(|a| actor_kind(a).is_none())
        .count();
    if untyped > 0 {
        report.findings.push(Finding::signal(
            "corpus",
            format!(
                "{untyped} authors predate the actor convention: the convention \
                 binds new writes, and these values mean what they meant"
            ),
        ));
    }

    // An entity an agent wrote and no human has read. Derived from what the
    // fields state and nothing further: no score, no confidence, no ranking —
    // the signal is recorded and the reader judges.
    for e in &considered {
        let Some(author) = author_of(e) else { continue };
        if actor_kind(author) != Some(ActorKind::Agent) {
            continue;
        }
        if readings_of(e)
            .iter()
            .any(|v| actor_kind(&v.by) == Some(ActorKind::Human))
        {
            continue;
        }
        // The full id, like every other finding in this report. It used to be
        // a four-character prefix, which was the one subject in `check` a
        // reader could not paste back into `ank show`: four characters is not
        // a length §3 lets anything print, and this line had no corpus in hand
        // to measure a length against (TASK-c1f01f301d63).
        report.findings.push(Finding::signal(
            e.id(),
            "written by an agent and read by no human",
        ));
    }

    // Burst creation by a single identity (§3, §4). §3 accepts task flooding
    // without a quota, on the argument that the defence is visibility rather
    // than restriction. This is that visibility, and nothing more: a burst is
    // reported, never refused.
    let mut by_author: HashMap<&str, Vec<i64>> = HashMap::new();
    for e in &considered {
        if let (Some(author), Some(at)) = (author_of(e), claim::parse_utc(created_of(e))) {
            by_author.entry(author).or_default().push(at);
        }
    }
    let mut bursts: Vec<(&str, usize)> = Vec::new();
    for (author, mut times) in by_author {
        times.sort_unstable();
        // A sliding window over the sorted timestamps: for each entity, how
        // many fall within BURST_WINDOW before it. Reporting the widest run
        // rather than one finding per window keeps a long burst to one line.
        let mut widest = 0usize;
        let mut start = 0usize;
        for end in 0..times.len() {
            while times[end] - times[start] > BURST_WINDOW {
                start += 1;
            }
            widest = widest.max(end - start + 1);
        }
        if widest > BURST_COUNT {
            bursts.push((author, widest));
        }
    }
    bursts.sort_unstable();
    for (author, count) in bursts {
        report.findings.push(Finding::signal(
            "corpus",
            format!(
                "burst creation by {author}: {count} entities within an hour \
                 (over {BURST_COUNT})"
            ),
        ));
    }

    // A blocker written by the agent that holds the blocked task, after it took
    // it. That is the shape of an agent building itself an excuse — and equally
    // the shape of an agent doing what §3 asks, since a discovered subtask *is*
    // a new task with a blocked_by. Only a reader knows which, so it is
    // reported and never refused.
    let authors: HashMap<&EntityId, (&str, i64)> = considered
        .iter()
        .filter_map(|e| {
            let at = claim::parse_utc(created_of(e))?;
            Some((e.id(), (author_of(e)?, at)))
        })
        .collect();

    for e in &considered {
        let Entity::Task(t) = e else { continue };
        let Some(Record::Claim(c)) = coord.get(&t.id) else {
            continue;
        };
        let Some(claimed) = claim::parse_utc(&c.claimed) else {
            continue;
        };
        let mut own: Vec<String> = t
            .blocked_by
            .iter()
            .filter(|b| match authors.get(b) {
                Some((author, created)) => *author == c.holder && *created > claimed,
                None => false,
            })
            .map(|b| b.to_string())
            .collect();
        own.sort();
        if !own.is_empty() {
            report.findings.push(Finding::signal(
                &t.id,
                format!(
                    "blocked by {} created by the holder ({}) after claiming",
                    own.join(" "),
                    c.holder
                ),
            ));
        }
    }
}

fn author_of(e: &Entity) -> Option<&str> {
    match e {
        Entity::Task(t) => t.author.as_deref(),
        Entity::Adr(a) => a.author.as_deref(),
    }
}

fn readings_of(e: &Entity) -> &[Verified] {
    match e {
        Entity::Task(t) => &t.verified,
        Entity::Adr(a) => &a.verified,
    }
}

/// What kind of actor a written identity claims to be (§3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorKind {
    Human,
    Agent,
    Process,
}

/// Reads the actor convention off a value, and answers `None` when the value
/// does not follow it.
///
/// `None` is not a fault and not a guess. An identity written before the
/// convention states nothing about what kind of actor produced it, so every
/// signal that needs the distinction skips it — which is why
/// `claude-code@sean-laptop` is reported once as pre-convention and never
/// treated as an agent it never claimed to be.
///
/// The convention is a signal and not a wall: an agent can write `human:` in
/// front of its own name, exactly as it can set `$ANK_AGENT` to anything. What
/// it buys is that the ordinary case becomes legible.
fn actor_kind(value: &str) -> Option<ActorKind> {
    if let Some(rest) = value.strip_prefix("human:") {
        return (!rest.is_empty()).then_some(ActorKind::Human);
    }
    if let Some(rest) = value.strip_prefix("process:") {
        return (!rest.is_empty()).then_some(ActorKind::Process);
    }
    // `<producer>/<version>`: one slash, and neither half empty. An `@` rules
    // it out, since `agent@host` is exactly the pre-convention shape.
    if value.contains('@') {
        return None;
    }
    match value.split_once('/') {
        Some((producer, version)) if !producer.is_empty() && !version.is_empty() => {
            Some(ActorKind::Agent)
        }
        _ => None,
    }
}

fn created_of(e: &Entity) -> &str {
    match e {
        Entity::Task(t) => &t.created,
        Entity::Adr(a) => &a.created,
    }
}

/// `blocked_by` cycles, reported once per cycle and naming the whole ring: a
/// cycle read one node at a time is a puzzle.
fn check_cycles(entities: &[(PathBuf, Entity)], report: &mut Report) {
    let edges: HashMap<EntityId, Vec<EntityId>> = entities
        .iter()
        .filter_map(|(_, e)| match e {
            Entity::Task(t) => Some((t.id.clone(), t.blocked_by.clone())),
            _ => None,
        })
        .collect();

    fn visit(
        node: &EntityId,
        edges: &HashMap<EntityId, Vec<EntityId>>,
        path: &mut Vec<EntityId>,
        done: &mut HashSet<EntityId>,
        reported: &mut HashSet<String>,
        report: &mut Report,
    ) {
        if let Some(at) = path.iter().position(|n| n == node) {
            let ring: Vec<String> = path[at..].iter().map(|n| n.to_string()).collect();
            let mut key = ring.clone();
            key.sort();
            if reported.insert(key.join(",")) {
                report.findings.push(Finding::fault(
                    node,
                    format!("blocked_by cycle: {} -> {node}", ring.join(" -> ")),
                ));
            }
            return;
        }
        if done.contains(node) {
            return;
        }
        path.push(node.clone());
        for next in edges.get(node).map(|v| v.as_slice()).unwrap_or(&[]) {
            if edges.contains_key(next) {
                visit(next, edges, path, done, reported, report);
            }
        }
        path.pop();
        done.insert(node.clone());
    }

    let mut done = HashSet::new();
    let mut reported = HashSet::new();
    let mut keys: Vec<&EntityId> = edges.keys().collect();
    keys.sort_by_key(|k| k.to_string());
    for start in keys {
        visit(
            start,
            &edges,
            &mut Vec::new(),
            &mut done,
            &mut reported,
            report,
        );
    }
}

/// Whether `default_branch` carries this task as `done`.
///
/// The committed blob and not the working tree, for the same reason `maintain`
/// reads it there: the default branch is the one copy every checkout agrees on,
/// and a working tree can say `done` on a branch nobody has merged.
///
/// False whenever the question cannot be asked — no default branch resolved, a
/// branch with no commit yet, git refusing. Unable to ask is not permission to
/// accuse, and `inspect` already reports the unaskable case once, as a corpus
/// line rather than once per task.
fn done_on(repo: &Repo, default_branch: Option<&str>, id: &EntityId) -> bool {
    let Some(branch) = default_branch else {
        return false;
    };
    matches!(
        file_at_branch(repo, branch, id),
        Ok(Some(text))
            if matches!(parse_entity(&text), Ok(Entity::Task(t)) if t.status == TaskStatus::Done)
    )
}

/// How far this checkout's corpus is from the corpus the default branch carries
/// (§4, ADR-47e2ac102f58).
///
/// **Named, and never repaired.** Nothing here fetches and nothing merges: both
/// revisions are already in this clone, and a reader that rewrote the plane
/// underneath every other agent to answer a question has stopped being a reader.
///
/// **Once for the corpus, never per entity.** A corpus six entities behind would
/// otherwise print six lines saying one thing, which is the volume that teaches
/// a reader to stop reading `check`.
///
/// **The count is a comparison and not a history walk.** `rev-list` would answer
/// how many commits the branches differ by, which is a different question: a
/// branch can move ten times without touching `.ank/`, and a count in commits
/// would fire on every merge and mean nothing.
///
/// Infallible on purpose. Every way of failing to compare is a state to report
/// rather than a run to abort — `check` still owes its corpus findings when git
/// cannot answer this one.
fn corpus_drift(repo: &Repo, branch: &str, here: &BTreeMap<String, PathBuf>, report: &mut Report) {
    let there = match corpus_at(repo, branch) {
        Ok(map) => map,
        // The revision does not resolve here, which [`git::file_at`] tells apart
        // from an absent path — the distinction this whole comparison rests on.
        //
        // Silent only where there was never anything to compare against: a
        // repository with no commit at all is the nominal state of one freshly
        // `ank init`-ed. A `default_branch` naming no commit in a repository
        // that has some is a mistyped branch or one never fetched, and
        // rendering that as a corpus that has not moved is the single answer
        // this signal must never give.
        Err(_) => {
            if has_commit(&repo.root) {
                report.findings.push(Finding::signal(
                    "corpus",
                    format!(
                        "{branch} names no commit here, so this corpus was not compared \
                         against the default branch (git fetch origin {branch})"
                    ),
                ));
            }
            return;
        }
    };
    let mine = match blobs_here(repo, here) {
        Ok(map) => map,
        Err(e) => {
            report.findings.push(Finding::signal(
                "corpus",
                format!(
                    "this corpus was not compared against {branch}: {} \
                     (git status --short {})",
                    e.message,
                    ank_relative(repo)
                ),
            ));
            return;
        }
    };
    let ids: BTreeSet<&String> = mine.keys().chain(there.keys()).collect();
    let entities = ids
        .iter()
        .filter(|id| mine.get(**id) != there.get(**id))
        .count();
    report.drift = Some(Drift {
        branch: branch.to_string(),
        entities,
    });
    if entities > 0 {
        report.findings.push(Finding::signal(
            "corpus",
            format!(
                "{entities} entity file(s) differ from {branch}: this checkout does not \
                 carry the corpus the default branch does (git merge {branch})"
            ),
        ));
    }
}

/// The entity files `branch` carries, keyed by id, with the blob each name
/// points at.
///
/// One `cat-file` per layout directory and never one per entity: `file_at` on a
/// directory reads the tree at that revision, which already carries the content
/// hash of every file in it. Asking per entity would be one process per entity
/// on a corpus of two hundred, for an answer the tree states in one line each.
///
/// The canonical layout wins over the previous one, exactly as [`file_at_branch`]
/// resolves a single entity: the branch and the working tree need not agree on
/// where entities live (§6).
fn corpus_at(repo: &Repo, branch: &str) -> Result<BTreeMap<String, String>> {
    let rel = ank_relative(repo);
    let mut dirs = vec![format!("{rel}/{}", Store::ENTITIES_DIR)];
    for kind in [EntityKind::Task, EntityKind::Adr] {
        if let Some(sub) = Store::legacy_subdir(kind) {
            dirs.push(format!("{rel}/{sub}"));
        }
    }
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    for dir in dirs {
        let Some(listing) = git::file_at(&repo.root, branch, &dir)? else {
            continue;
        };
        // `<mode> SP <type> SP <object>TAB<name>`, git's tree format, which has
        // carried that shape since `cat-file -p` printed trees.
        for line in listing.lines() {
            let Some((meta, name)) = line.split_once('\t') else {
                continue;
            };
            let mut fields = meta.split_whitespace().skip(1);
            if fields.next() != Some("blob") {
                continue;
            }
            let Some(object) = fields.next() else {
                continue;
            };
            let Some(id) = name
                .strip_suffix(".md")
                .filter(|stem| EntityId::parse(stem).is_ok())
            else {
                continue;
            };
            found
                .entry(id.to_string())
                .or_insert_with(|| object.to_string());
        }
    }
    Ok(found)
}

/// The same hashes for the working tree, so the two sides are compared on git's
/// own terms.
///
/// `hash-object` and not a byte comparison of the two texts, for a reason that
/// only shows on one platform: a checkout with `core.autocrlf` on holds CRLF
/// where the blob holds LF, and a corpus read literally would differ from the
/// default branch in every single file on Windows and in none of them
/// elsewhere. `hash-object` applies the same conversion the commit did, which
/// makes the comparison mean what it says.
///
/// Batched, and bounded. One process for the whole corpus where the command line
/// allows it, chunked below a length every platform accepts — a corpus is not
/// bounded, and a command line is.
fn blobs_here(repo: &Repo, here: &BTreeMap<String, PathBuf>) -> Result<BTreeMap<String, String>> {
    /// Well under the shortest limit of the three platforms, and far above what
    /// a corpus of a few hundred entities needs.
    const BUDGET: usize = 6000;

    let ids: Vec<&String> = here.keys().collect();
    let paths: Vec<String> = here
        .values()
        .map(|p| {
            p.strip_prefix(&repo.root)
                .unwrap_or(p)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < paths.len() {
        let start = i;
        let mut args: Vec<&str> = vec!["hash-object", "--"];
        let mut budget = 0usize;
        while i < paths.len() && (i == start || budget + paths[i].len() < BUDGET) {
            budget += paths[i].len() + 1;
            args.push(&paths[i]);
            i += 1;
        }
        let text = git::run(&repo.root, &args)?;
        let objects: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();
        if objects.len() != i - start {
            return Err(CliError::new(
                9,
                format!(
                    "git hash-object answered {} object(s) for {} file(s)",
                    objects.len(),
                    i - start
                ),
            ));
        }
        for (n, object) in objects.into_iter().enumerate() {
            out.insert(ids[start + n].to_string(), object.to_string());
        }
    }
    Ok(out)
}

/// Whether this repository carries any commit at all.
///
/// The one thing that separates a freshly `ank init`-ed repository, where there
/// is nothing to compare a corpus against and silence is correct, from a
/// `default_branch` that names nothing in a repository full of history.
fn has_commit(root: &Path) -> bool {
    git::output(root, &["rev-parse", "--verify", "--quiet", "HEAD^{commit}"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Maintenance of the coordination plane (§7). The only place that prunes.
fn maintain(
    repo: &Repo,
    default_branch: &str,
    coord: &HashMap<EntityId, Record>,
    statuses: &HashMap<EntityId, TaskStatus>,
    prune: bool,
    report: &mut Report,
) -> Result<()> {
    // Sorted, so that two runs on the same repository report and prune in the
    // same order: a maintenance command whose output shuffles is one nobody
    // diffs.
    let mut unreachable_branch: Option<String> = None;
    let mut ids: Vec<&EntityId> = coord.keys().collect();
    ids.sort_by_key(|i| i.to_string());
    for id in ids {
        let record = &coord[id];
        // An orphan: a ref for a task that no longer exists anywhere.
        //
        // "Anywhere" is the load-bearing word, and asking the working tree
        // cannot answer it. `refs/ank/` is shared by every worktree of a
        // repository, so a checkout older than a task sees no such task and
        // used to delete a claim another worktree was holding at that moment
        // (TASK-52fbffbfdf65). The default branch is the one copy every
        // checkout agrees on, and the `settled` test below already reads it —
        // this asks the same question of the same source, one step earlier.
        if !statuses.contains_key(id) {
            match file_at_branch(repo, default_branch, id) {
                // Not an orphan: this checkout is simply older than the task,
                // or on a branch that never carried it. Silent on purpose. The
                // ref belongs to whoever holds it now, and a signal here would
                // fire on every check run from every branch predating the task.
                Ok(Some(_)) => {}
                Ok(None) => {
                    if prune {
                        claim::delete(&repo.root, id)?;
                        report.pruned.push(claim::claim_ref(id));
                    } else {
                        report
                            .findings
                            .push(Finding::signal(id, "orphan ref: no such task"));
                    }
                }
                // Unable to ask is not permission to delete: report once and
                // keep the ref, which is the reader's behaviour §2 asks for.
                Err(_) => {
                    if unreachable_branch.is_none() {
                        unreachable_branch = Some(default_branch.to_string());
                    }
                }
            }
            continue;
        }
        // A branch that names nothing yet is the nominal state of a repository
        // freshly `ank init`-ed: it has a default branch and no commit on it.
        // Reporting once and pruning nothing is the reader's behaviour (§2);
        // failing here would make `check` unusable on a new repository.
        let settled = match file_at_branch(repo, default_branch, id) {
            Ok(Some(text)) => matches!(
                parse_entity(&text),
                Ok(Entity::Task(t)) if matches!(t.status, TaskStatus::Done | TaskStatus::Closed)
            ),
            Ok(None) => false,
            Err(_) => {
                if unreachable_branch.is_none() {
                    unreachable_branch = Some(default_branch.to_string());
                }
                continue;
            }
        };
        if settled {
            // The information the ref carried is now where everybody reads it.
            if prune {
                claim::delete(&repo.root, id)?;
                report.pruned.push(claim::claim_ref(id));
            }
        } else if matches!(record, Record::Completed(_)) {
            // Not a corpus anomaly: a branch never merged. The answer is human.
            report.findings.push(Finding::signal(
                id,
                format!("finished on another branch, {default_branch} has not caught up"),
            ));
        }
    }
    if let Some(branch) = unreachable_branch {
        report.findings.push(Finding::signal(
            "coordination",
            format!("{branch} carries no commit yet: nothing pruned, nothing judged"),
        ));
    }
    Ok(())
}

/// The same maintenance for `refs/ank/proof/*` (ADR-493471d64ba0).
///
/// **The same predicate, and it has to be**: the ref lives exactly as long as
/// what it carries is not yet where everyone reads it. A proof ref pruned on
/// time would delete the record precisely during the window it exists to cover,
/// which is why it carries no TTL and nothing here consults a clock.
///
/// Deleted only when *every* attestation it holds has an equivalent in the file
/// on the default branch. A record carrying two runs of which one has landed is
/// left alone: pruning it would lose the other, and rewriting the record to drop
/// half of it would be this command editing an attestation, which it has no
/// standing to do.
fn maintain_proofs(
    repo: &Repo,
    default_branch: &str,
    detached: &HashMap<EntityId, Vec<claim::AttestedProof>>,
    statuses: &HashMap<EntityId, TaskStatus>,
    prune: bool,
    report: &mut Report,
) -> Result<()> {
    let mut ids: Vec<&EntityId> = detached.keys().collect();
    ids.sort_by_key(|i| i.to_string());
    for id in ids {
        let Ok(found) = file_at_branch(repo, default_branch, id) else {
            // `maintain` has already reported an unreadable default branch
            // once. A second line about the same cause would report the
            // consequence as if it were a separate finding.
            continue;
        };
        let Some(text) = found else {
            // No such task on the default branch. An orphan only if the corpus
            // does not carry it either — otherwise this is the ordinary case
            // the ref exists for: a task finished on a branch nobody merged.
            if !statuses.contains_key(id) {
                if prune {
                    claim::delete_at(&repo.root, &claim::proof_ref(id))?;
                    report.pruned.push(claim::proof_ref(id));
                } else {
                    report
                        .findings
                        .push(Finding::signal(id, "orphan proof ref: no such task"));
                }
            }
            continue;
        };
        let Ok(Entity::Task(landed)) = parse_entity(&text) else {
            continue;
        };
        let settled = detached[id].iter().all(|a| {
            landed.proof.iter().any(|p| {
                p.proof_type == a.proof.proof_type && p.reference.trim() == a.proof.reference.trim()
            })
        });
        if settled && prune {
            claim::delete_at(&repo.root, &claim::proof_ref(id))?;
            report.pruned.push(claim::proof_ref(id));
        }
    }
    Ok(())
}

/// The `.ank/` directory relative to the repository root, `/`-separated, as git
/// wants it. Usually `.ank`, but the tree need not be laid out that way.
fn ank_relative(repo: &Repo) -> String {
    repo.ank
        .strip_prefix(&repo.root)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".ank".to_string())
}

/// Every repository-relative path an entity may occupy, canonical first.
///
/// Two, for as long as the previous layout is read (§6). Asking git about a
/// path is not opening a file: a branch, a commit or an index can hold the
/// entity where the working tree no longer does, so the caller tries both and
/// takes the first answer. And when `accept` stages a commit it stages both,
/// because a write moves the file out of the previous layout and a commit that
/// staged only the destination would leave the removal uncommitted.
fn entity_rel_paths(repo: &Repo, id: &EntityId) -> Vec<String> {
    let rel = ank_relative(repo);
    let mut paths = vec![format!("{rel}/{}/{id}.md", Store::ENTITIES_DIR)];
    if let Some(sub) = Store::legacy_subdir(id.kind()) {
        paths.push(format!("{rel}/{sub}/{id}.md"));
    }
    paths
}

/// The paths to hand `git add` and `git commit` for an entity about to be
/// written. **Called before the write**, while the file is still where it is.
///
/// The canonical path always, since that is where the write lands. The previous
/// layout's only when a file is actually there, because a pathspec matching
/// neither the tree nor the index is a fatal error and not a no-op — `git add`
/// refuses rather than shrugging, and the whole ratification would fail over a
/// path that was never going to match.
/// The entity as a branch carries it, from whichever layout holds it **there**.
///
/// The working tree and the branch need not agree on the layout: a corpus moved
/// on a feature branch is still in the previous one on the default branch until
/// the merge lands, and the pruning predicate reads the default branch (§7). So
/// the candidates are tried against git and not against the disk.
///
/// An error from `file_at` is the revision being unresolvable, which is the same
/// answer for every path, so it is returned rather than swallowed: unable to ask
/// is not permission to delete.
fn file_at_branch(repo: &Repo, branch: &str, id: &EntityId) -> Result<Option<String>> {
    let mut failure = None;
    for path in entity_rel_paths(repo, id) {
        match git::file_at(&repo.root, branch, &path) {
            Ok(Some(text)) => return Ok(Some(text)),
            Ok(None) => {}
            Err(e) => failure = Some(e),
        }
    }
    match failure {
        Some(e) => Err(e),
        None => Ok(None),
    }
}

fn entity_rel_paths_to_stage(repo: &Repo, id: &EntityId) -> Vec<String> {
    let rel = ank_relative(repo);
    let mut paths = vec![format!("{rel}/{}/{id}.md", Store::ENTITIES_DIR)];
    if let Some(sub) = Store::legacy_subdir(id.kind()) {
        if repo.ank.join(sub).join(format!("{id}.md")).exists() {
            paths.push(format!("{rel}/{sub}/{id}.md"));
        }
    }
    paths
}

fn render(report: &Report, inv: &Invocation, out: &mut dyn Write) {
    if inv.json() {
        let items: Vec<String> = report
            .findings
            .iter()
            .map(|f| {
                let note: Vec<String> = f.note.iter().map(|l| json_str(l)).collect();
                // Numbers, not the lines a human is shown. A caller sorting
                // constraints by cost or summing them gets arithmetic it can do
                // directly; re-parsing "charges 2431 characters" would be the
                // sentence this field exists to avoid.
                let charge: Vec<String> = f
                    .charge
                    .iter()
                    .map(|c| format!("{{\"id\":\"{}\",\"characters\":{}}}", c.id, c.characters))
                    .collect();
                format!(
                    "{{\"level\":\"{}\",\"subject\":\"{}\",\"message\":{},\"note\":[{}],\
                     \"charge\":[{}]}}",
                    if f.level == Level::Fault {
                        "fault"
                    } else {
                        "signal"
                    },
                    f.subject,
                    json_str(&f.message),
                    note.join(","),
                    charge.join(",")
                )
            })
            .collect();
        let pruned: Vec<String> = report.pruned.iter().map(|p| json_str(p)).collect();
        let _ = writeln!(
            out,
            "{{\"faults\":{},\"signals\":{},\"tasks\":{},\"adr\":{},\"pruned\":[{}],\"findings\":[{}]}}",
            report.faults(),
            report.signals(),
            report.tasks,
            report.adrs,
            pruned.join(","),
            items.join(",")
        );
        return;
    }
    if inv.quiet() {
        return;
    }
    let style = inv.style();
    for f in &report.findings {
        let tag = if f.level == Level::Fault {
            style.red("error:")
        } else {
            style.yellow("signal:")
        };
        let _ = writeln!(out, "{tag} {}: {}", f.subject, f.message);
        // The structure alphabet of §4 and nothing outside it: a note is the
        // last child of its finding, so `LAST` opens it and `CLEAR` continues
        // it. Text and never colour — it carries the command to run next, and a
        // reader who piped this to a file must read the same bytes (ADR-0c8a).
        //
        // The breakdown comes first and the note after it, in one run of
        // children: the numbers are what the note's advice is about, and a
        // reader who met the advice first would have nothing to apply it to.
        let breakdown = f
            .charge
            .iter()
            .map(|c| format!("{} charges {} characters", c.id, c.characters));
        for (i, line) in breakdown.chain(f.note.iter().cloned()).enumerate() {
            let lead = if i == 0 {
                style::glyph::LAST
            } else {
                style::glyph::CLEAR
            };
            let _ = writeln!(out, "{lead}{line}");
        }
    }
    for p in &report.pruned {
        let _ = writeln!(out, "{} {}", style.retracted("pruned"), style.id(p));
    }
    let _ = writeln!(
        out,
        "check: {} — {} tasks, {} adr, {} signal(s)",
        if report.faults() == 0 {
            style.green("ok")
        } else {
            style.red(&format!("{} fault(s)", report.faults()))
        },
        report.tasks,
        report.adrs,
        report.signals()
    );
}

fn json_str(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ---------------------------------------------------------------------------
// review
// ---------------------------------------------------------------------------

/// The human read of a perimeter: what binds it, what has died, and what
/// `check` would say — without touching the coordination plane.
pub fn review(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<i32> {
    let path = crate::context::perimeter(inv, repo)?;
    let report = inspect(repo, cfg, path.as_deref(), false)?;
    let index = Index::open(&repo.ank)?;
    let files = tracked_files(&repo.root);

    // Filtered by live scopes: a decision matching no file is reviewed as dead,
    // not as binding (§11).
    let dead: HashSet<String> = report
        .findings
        .iter()
        .filter(|f| f.message.starts_with("dead scope"))
        .map(|f| f.subject.clone())
        .collect();

    let mut live = Vec::new();
    let mut proposed = Vec::new();
    for row in index.all()? {
        if row.kind != EntityKind::Adr {
            continue;
        }
        // The third copy of this matching, now gone the way of the other two:
        // `review` deciding for itself what a perimeter contains is how it came
        // to disagree with `context` about `docs\` (TASK-df4c39031583).
        if !crate::context::in_perimeter(&row.scope, path.as_deref()) {
            continue;
        }
        match row.status.as_str() {
            // The queue `accept` draws from, and the question this verb opens
            // with. Not filtered by `dead`, unlike the constraints below: a
            // proposal whose scope has died is still waiting for a human, and
            // dropping it from the queue would hide the one entry most in need
            // of the answer. Its dead scope is reported in its own section.
            "proposed" => proposed.push(row),
            "accepted" => {
                if dead.contains(&row.id.to_string()) {
                    continue;
                }
                let matched = ScopeSet::new(&row.scope)
                    .map(|s| files.iter().filter(|f| s.matches(f)).count())
                    .unwrap_or(0);
                live.push((row, matched));
            }
            // `superseded` binds nobody and is not waiting for anybody: it is
            // history, and history is not a review of the present.
            _ => {}
        }
    }
    live.sort_by_key(|(r, _)| r.id.to_string());
    proposed.sort_by_key(|r| r.id.to_string());

    let mut dead: Vec<String> = dead.into_iter().collect();
    dead.sort();

    if inv.json() {
        let items: Vec<String> = live
            .iter()
            .map(|(r, n)| {
                format!(
                    "{{\"id\":\"{}\",\"title\":{},\"files\":{n}}}",
                    r.id,
                    json_str(&r.title)
                )
            })
            .collect();
        // `files` is not carried here, and its absence is the point: it counts
        // what a constraint binds today, and a proposal binds nothing until it
        // is ratified. The queue's question is which decisions are waiting.
        let waiting: Vec<String> = proposed
            .iter()
            .map(|r| format!("{{\"id\":\"{}\",\"title\":{}}}", r.id, json_str(&r.title)))
            .collect();
        let _ = writeln!(
            out,
            "{{\"proposed\":[{}],\"live\":[{}],\"dead\":{},\"faults\":{},\"signals\":{}}}",
            waiting.join(","),
            items.join(","),
            dead.len(),
            report.faults(),
            report.signals()
        );
        return Ok(report.exit_code());
    }
    if !inv.quiet() {
        let style = inv.style();
        // First, because it is the question `review` exists to answer: §4 opens
        // its description with "ratification queue", and a maintainer runs this
        // before ratifying. It used to print nowhere at all, and an empty queue
        // and an unprinted queue read identically (TASK-e3d00a6e62bb).
        if proposed.is_empty() {
            // Said even when there is nothing to say, on the reasoning `status`
            // already applies to `elsewhere no claim by another agent`: silence
            // and "this verb does not answer that" are the same bytes, and this
            // verb is where the question has an answer.
            let _ = writeln!(out, "{}", style.key("nothing proposed for ratification"));
        } else {
            let _ = writeln!(
                out,
                "{}",
                style.header(&format!("PROPOSED ({})", proposed.len()))
            );
            for r in &proposed {
                let _ = writeln!(out, "  {}  {}", style.id(&r.id.to_string()), r.title);
            }
        }
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "{}",
            style.header(&format!("LIVE CONSTRAINTS ({})", live.len()))
        );
        for (r, n) in &live {
            let _ = writeln!(
                out,
                "  {}  {} ({n} files)",
                style.id(&r.id.to_string()),
                r.title
            );
        }
        if !dead.is_empty() {
            let _ = writeln!(
                out,
                "\n{}",
                style.header(&format!("DEAD SCOPES ({})", dead.len()))
            );
            for id in &dead {
                let _ = writeln!(out, "  {}", style.id(&id.to_string()));
            }
        }
        let _ = writeln!(
            out,
            "\n{} fault(s), {} signal(s)",
            if report.faults() == 0 {
                report.faults().to_string()
            } else {
                style.red(&report.faults().to_string())
            },
            report.signals()
        );
    }
    Ok(report.exit_code())
}

// ---------------------------------------------------------------------------
// accept
// ---------------------------------------------------------------------------

/// Promotes a `proposed` ADR to `accepted` and commits it, signed.
///
/// The only command writing into history rather than into the working tree, and
/// therefore the only one carrying a branch precondition (§12): a ratification
/// commit cannot wait for a merge to become authoritative — it is authoritative
/// as soon as it exists, on the branch where it exists.
pub fn accept(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "accept expects an id").with_hint("ank accept <id>"))?;

    // 9 and 7 are deliberately distinct. 9 says "I do not know where the right
    // place is", and the repository needs repairing; 7 says "you are not in the
    // right place", and the caller knows what to do. Conflating them would send
    // somebody switching branches over a configuration problem.
    let default_branch = git::resolve_default_branch(
        cfg.default_branch.as_deref(),
        git::origin_head(&repo.root)?.as_deref(),
    )?;
    let current = git::current_branch(&repo.root)?;
    if current.as_deref() != Some(default_branch.as_str()) {
        let here = current.as_deref().unwrap_or("a detached HEAD");
        return Err(CliError::new(
            7,
            format!(
                "accept requires the default branch (current: {here}, default: {default_branch})"
            ),
        )
        .with_hint(format!(
            "git switch {default_branch} && ank accept {prefix}"
        )));
    }

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Adr(mut adr) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not an ADR"))
            .with_hint(format!("ank show {prefix}")));
    };
    // Ratifying is not re-accepting. An `accepted` ADR carrying no anchor at
    // all was promoted by editing the file, or predates the tool — `check`
    // reports exactly that, as a signal — and this is the only door through
    // which a bootstrap corpus ever acquires the signed commits the authority
    // model rests on. `accept` was written for a corpus it could not reach.
    //
    // The refusal below is the half doing the work. `accept` over an existing
    // anchor is how a constraint edited in place would be re-anchored, which is
    // the one property ADR-6b3f19e08a24 exists to hold; changing a ratified
    // decision stays a succession, and the hint names it rather than showing the
    // file. Supplying a *first* anchor launders nothing: there is nothing for it
    // to diverge from.
    let ratifying_in_place = match (adr.status, adr.ratified.as_deref()) {
        (AdrStatus::Accepted, Some(anchor)) => {
            return Err(
                CliError::new(6, format!("{} is already ratified at {anchor}", adr.id))
                    .with_hint(format!("ank new adr --supersedes {}", adr.id)),
            );
        }
        (AdrStatus::Accepted, None) => true,
        _ => false,
    };
    if !ratifying_in_place {
        adr.status
            .check_transition(AdrStatus::Accepted)
            .map_err(|e| {
                CliError::new(6, e.to_string()).with_hint(format!("ank show {}", adr.id))
            })?;
    }
    let id = adr.id.clone();

    // Everything that can refuse, refused before anything is written. A
    // half-performed succession is a corpus the ratification commit would then
    // make authoritative, and `accept` has no second pass to repair it.
    let replaced = succession(&store, &adr)?;

    // The hash of what is being made binding, recorded before the commit that
    // makes it so: `constraint` and `scope` together are what is ratified (§8).
    //
    // The promotion itself is written here, and it was missing: the transition
    // was checked above and never performed, so `accept` wrote `ratified` onto
    // an ADR that stayed `proposed`. Neither test reached this line — both
    // assert refusals — which is the shape CLAUDE.md warns about, found by a
    // test that finally ran the commit.
    let anchor = ratification_anchor(&adr.constraint, &adr.scope);
    adr.status = AdrStatus::Accepted;
    adr.ratified = Some(anchor.clone());

    let mut paths = Vec::new();

    // The target first, and the order is the argument. Between the two writes
    // the corpus holds a target marked `superseded` whose superseder is still
    // `proposed` — which `check_adr` calls clean in both directions, because the
    // proposal does name it. The reverse order leaves an accepted superseder
    // over an unmarked target, which is precisely the fault.
    //
    // A `Recorded` succession skips this write and keeps the commit line: the
    // file already says what the commit is about to say, and rewriting it would
    // bump a version to record nothing.
    if let Succession::Pending(target) = &replaced {
        let mut target = target.clone();
        let target_base = target.version;
        target.status = AdrStatus::Superseded;
        paths.extend(entity_rel_paths_to_stage(repo, &target.id));
        store.write(&Entity::Adr(target), target_base)?;
    }
    paths.extend(entity_rel_paths_to_stage(repo, &id));
    store.write(&Entity::Adr(adr), base_version)?;

    // One commit for both writes. The succession is a single act, and two
    // commits would leave a window in which history says the constraint binds
    // while the one it replaced still binds too.
    let replaces = match replaced.target() {
        Some(t) => format!("supersedes: {}\n", t.id),
        None => String::new(),
    };
    let message = format!("ratify {id}\n\nconstraint+scope: {anchor}\n{replaces}by: {identity}\n");
    let commit = commit_signed(&repo.root, &paths, &message)?;

    if inv.json() {
        let superseded = match replaced.target() {
            Some(t) => format!("\"{}\"", t.id),
            None => "null".to_string(),
        };
        let _ = writeln!(
            out,
            "{{\"adr\":\"{id}\",\"status\":\"accepted\",\"superseded\":{superseded},\"commit\":\"{commit}\",\"anchor\":\"{anchor}\"}}"
        );
    } else if !inv.quiet() {
        // Two words for two acts. Reporting "accepted" over an ADR that was
        // already accepted would describe the wrong half of what just happened:
        // what the caller obtained is the anchor.
        let verb = if ratifying_in_place {
            "ratified"
        } else {
            "accepted"
        };
        let style = inv.style();
        let _ = writeln!(
            out,
            "{} {} -> {}",
            style.advanced(verb),
            style.id(&id.to_string()),
            &commit[..commit.len().min(7)]
        );
        if let Succession::Pending(t) = &replaced {
            let _ = writeln!(
                out,
                "{} {}",
                style.retracted("superseded"),
                style.id(&t.id.to_string())
            );
        }
    }
    Ok(0)
}

/// What `accept` has left to do about the ADR this one replaces.
enum Succession {
    /// It replaces nothing.
    None,
    /// The target is accepted, and this `accept` marks it superseded.
    Pending(Adr),
    /// The target already reads `superseded` and no other accepted ADR claims
    /// it, so the succession is on record and there is nothing left to write.
    Recorded(Adr),
}

impl Succession {
    /// The target, whether or not this `accept` is the one writing it — the
    /// commit message and the output name it either way, because the succession
    /// is what the ratification is about.
    fn target(&self) -> Option<&Adr> {
        match self {
            Succession::None => None,
            Succession::Pending(a) | Succession::Recorded(a) => Some(a),
        }
    }
}

/// The ADR this one replaces, loaded and checked.
///
/// `model.rs` states `Accepted -> Superseded` as the only legal write on an
/// accepted ADR, "performed by the `accept` of the ADR that replaces it". This
/// is that write's precondition, and it runs before `accept` touches anything:
/// a refusal must leave the corpus exactly as it found it.
fn succession(store: &Store, adr: &Adr) -> Result<Succession> {
    let Some(target_id) = adr.supersedes.clone() else {
        return Ok(Succession::None);
    };
    let Entity::Adr(target) = store.load(&target_id)?.entity else {
        return Err(CliError::new(1, format!("{target_id} is not an ADR"))
            .with_hint(format!("ank show {target_id}")));
    };

    // Already replaced by somebody else. Re-pointing the chain silently would
    // rewrite whose succession this was, and the corpus keeps no record of the
    // one it dropped.
    //
    // Scanned before the status is read, because it is what the status alone
    // cannot say: a target marked `superseded` is either this ADR's doing or
    // another's, and only the absence of another claimant makes it this one's.
    for other in store.list_ids()? {
        if other.kind() != EntityKind::Adr || other == adr.id {
            continue;
        }
        if let Entity::Adr(o) = store.load(&other)?.entity {
            if o.supersedes.as_ref() == Some(&target_id) && o.status == AdrStatus::Accepted {
                return Err(CliError::new(
                    7,
                    format!("{target_id} is already superseded by {other}"),
                )
                .with_hint(format!("ank show {other}")));
            }
        }
    }

    match target.status {
        AdrStatus::Accepted => Ok(Succession::Pending(target)),

        // Marked, unclaimed by anyone else, and named here: the succession
        // exists. Either it was performed by hand during bootstrap, or an
        // earlier `accept` wrote the target and did not reach its second write —
        // the same state, and the same thing to do about it, which is nothing.
        AdrStatus::Superseded => Ok(Succession::Recorded(target)),

        // Superseding a proposal is meaningless — nothing was ever binding — and
        // a caller who wrote one almost certainly meant a different identifier.
        // A prerequisite unmet, so 7 and not the 6 of an illegal transition.
        AdrStatus::Proposed => Err(CliError::new(
            7,
            format!("{target_id} is proposed, and only an accepted ADR can be superseded"),
        )
        .with_hint(format!("ank accept {target_id}"))),
    }
}

/// Whether an accepted ADR still says what was ratified (§3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Freeze {
    /// Nothing to compare: proposed, superseded, or accepted with no anchor at
    /// all — the bootstrap state, which `check` reports on its own.
    Unanchored,
    /// The commit and the file agree.
    Intact,
    /// The constraint or the scope moved after ratification.
    Altered { ratified: String, now: String },
    /// No ratification commit is reachable, so there is nothing to compare
    /// against. Distinct from `Altered` on purpose.
    Unverifiable,
}

/// Compares an accepted ADR against the anchor its ratification commit records.
///
/// The comparison deliberately ignores the `ratified` field's own value and
/// uses only the commit's. The field is written by whoever writes the file, so
/// an editor changing a constraint can change it in the same stroke; the
/// commit's copy is the one that costs a signature to replace. What `ratified`
/// still does is say the ADR claims to be ratified, which is what sends us
/// looking for the commit at all.
pub fn freeze_state(repo: &Repo, adr: &Adr) -> Freeze {
    if adr.status != AdrStatus::Accepted || adr.ratified.is_none() {
        return Freeze::Unanchored;
    }
    let recorded = match ratification_of(repo, adr) {
        Some(r) => r.anchor,
        // A git that cannot answer is not a divergence either. Reporting one
        // over a broken environment is how a finding becomes noise.
        None => return Freeze::Unverifiable,
    };
    let now = ratification_anchor(&adr.constraint, &adr.scope);
    if now == recorded {
        Freeze::Intact
    } else {
        Freeze::Altered {
            ratified: recorded,
            now,
        }
    }
}

/// The ratification commit for an accepted, anchored ADR.
fn ratification_of(repo: &Repo, adr: &Adr) -> Option<git::Ratification> {
    if adr.status != AdrStatus::Accepted || adr.ratified.is_none() {
        return None;
    }
    // Whichever layout the ADR sits in, and its own history is what carries the
    // ratification: an ADR ratified before the move has its commit on the path
    // it had then. The candidates go in together rather than one call each,
    // because the memo is keyed on the ADR and a first miss would be cached.
    let paths = entity_rel_paths(repo, &adr.id);
    git::ratification_at(&repo.root, &adr.id.to_string(), &paths).unwrap_or(None)
}

/// The signature on the commit the anchor was read from, or `None` when there
/// is no such commit to ask about — which `freeze_state` already reports as
/// unverifiable and which would only be said twice here.
///
/// This is the answer to the hole TASK-03eaa26bddd1 left open in writing: the
/// anchor was compared against the file without anyone asking who wrote it, so
/// an ordinary unsigned commit whose subject read `ratify <id>` was accepted as
/// a ratification.
pub fn signature_state(repo: &Repo, adr: &Adr) -> Option<Signature> {
    let declared = declared_signers(repo);
    let signers = repo.ank.join("allowed_signers");

    // Nothing declared is not "signed by nobody": it is the advisory mode §8
    // describes, already reported once by `check_signers`, and there is no
    // allowlist to judge against. Going further would also be unsafe — git
    // reports `N` for a perfectly signed commit when `gpg.format` is ssh and no
    // allowed-signers file is configured, so a corpus with no file would have
    // every ratification called unsigned.
    if declared.is_empty() || !signers.is_file() {
        return None;
    }

    let sha = ratification_of(repo, adr)?.sha;
    // `.ok()?` here used to fold every git failure into `None`, the one verdict
    // `check_adr` says nothing about — so a machine where the signature could
    // not be read looked exactly like a corpus that declared no key at all
    // (TASK-c92b7cc10f13). Failing to ask has to survive as an answer.
    match git::signature_of(&repo.root, &sha, Some(&signers)) {
        Ok(facts) => {
            // Asked only where the answer can change the verdict: every other
            // status already says whether a signature was there.
            let carries = facts.status == 'N' && commit_carries_signature(&repo.root, &sha);
            Some(classify_signature(&facts, &declared, carries))
        }
        Err(e) => Some(Signature::Unreadable {
            reason: e.to_string(),
        }),
    }
}

/// Whether the commit object holds a signature header at all, whatever git was
/// able to make of it.
///
/// `%G?` collapses two states into `N`: a commit nobody signed, and a signed
/// commit whose signature git could not even attempt to check. Measured rather
/// than inferred — with `gpg.program` pointing at a path that does not exist,
/// `rev-list --format=%G?` prints `N` for a commit signed by the maintainer and
/// puts `error: cannot spawn ...` on stderr, where a verdict has no business
/// living. Reporting that as `Absent` tells a contributor whose only fault is
/// not having GnuPG installed that the corpus is ratified by nobody
/// (TASK-f4ed2020c964).
///
/// The object itself is not ambiguous. A signed commit carries a `gpgsig`
/// header — `gpgsig-sha256` in a SHA-256 repository, and an SSH signature uses
/// the same header — and `cat-file` reads it without gpg, without a keyring and
/// without an opinion.
///
/// A failure to read is `false`: this only ever weakens `Unchecked` back to
/// `Absent`, which is the verdict the caller would have reached anyway, and
/// `Unreadable` already covers a git that cannot answer at all.
fn commit_carries_signature(root: &Path, sha: &str) -> bool {
    let Ok(object) = git::run(root, &["cat-file", "commit", sha]) else {
        return false;
    };
    // Headers stop at the first blank line. A message body quoting `gpgsig` is
    // not a signature, and a continuation line of the block starts with a space.
    object
        .lines()
        .take_while(|l| !l.is_empty())
        .any(|l| l.starts_with("gpgsig"))
}

/// Hash of `constraint` + `scope`, normalised. What the ratification commit
/// anchors, and what `check` compares the file against afterwards.
pub fn ratification_anchor(constraint: &str, scope: &[String]) -> String {
    let mut buf = freeze::normalize(constraint);
    buf.push('\n');
    for g in scope {
        buf.push_str(g.trim());
        buf.push('\n');
    }
    freeze::freeze_hash_short(&buf)
}

/// The one commit Ank produces. Signed, because the authority model rests on
/// the signature and on nothing else (§8).
///
/// Several paths, because a succession is two writes and one act: the ADR being
/// ratified and the one it replaces go into the same commit, or history holds a
/// moment where both constraints bind.
///
/// `add` and `commit` are porcelain, and this is the documented exception:
/// neither has a plumbing equivalent worth rewriting, and ADR-b8884edcebe3's
/// rule is about parsing output — nothing here is parsed but the resulting sha,
/// which `rev-parse` supplies.
fn commit_signed(cwd: &Path, paths: &[String], message: &str) -> Result<String> {
    use std::process::Command;
    let run = |args: &[&str]| -> Result<()> {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
            .map_err(|e| CliError::new(9, format!("git {}: {e}", args.join(" "))))?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            return Err(
                CliError::new(9, format!("git {} failed: {stderr}", args.join(" "))).with_hint(
                    "git config user.signingkey <key> && git config commit.gpgsign true",
                ),
            );
        }
        Ok(())
    };
    let refs: Vec<&str> = paths.iter().map(|p| p.as_str()).collect();

    // `-A`, and `--ignore-removal` would be exactly wrong: a path in this list
    // may name a file the write has just moved away from, and staging only what
    // still exists would commit the new file while leaving its removal behind.
    // `-A` also makes a path that never existed a no-op rather than a fatal
    // pathspec error, which is what the second candidate path usually is.
    let mut add = vec!["add", "-A", "--"];
    add.extend_from_slice(&refs);
    run(&add)?;

    let mut commit = vec!["commit", "-S", "-q", "-m", message, "--"];
    commit.extend_from_slice(&refs);
    run(&commit)?;

    git::run(cwd, &["rev-parse", "HEAD"])
}

// ---------------------------------------------------------------------------
// close and show
// ---------------------------------------------------------------------------

/// Ratified abandonment (§3). Terminal, reachable from `open` and
/// `in_progress`, and it revokes the active claim in the same operation — the
/// holding agent learns of it at its next `log`.
///
/// Never by deleting the file: that would break other tasks' `blocked_by`
/// references, where `closed` preserves them.
///
/// **It leaves nothing on the coordination plane, and that is the decision**
/// (TASK-78326e2e3e89). `done` turns the claim ref into a completion record so
/// that a task finished on an unmerged branch does not look free everywhere
/// else (ADR-bcf222a31525); `close` deletes the ref, so a task closed on a
/// branch is claimable elsewhere until the closure lands on the default branch.
/// The asymmetry is not an omission. A completion record refuses every other
/// `claim` with code 4, repository-wide once the ref is pushed, and `done`
/// earns that with a frozen criterion, declared verifiers and a proof where
/// this verb is gated by a reason alone. `close` also revokes somebody else's
/// live claim, so the symmetric version would let one agent take a task away
/// *and* stop anyone picking it up, on a branch nobody has reviewed.
///
/// The two errors differ in size the same way: a `done` invisible elsewhere
/// wastes work already performed, a `close` invisible elsewhere costs work on a
/// task somebody proposed to abandon — and that work, if it finishes, produces
/// a proof, which argues against the closure rather than being lost.
pub fn close(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(1, "close expects an id").with_hint("ank close <id> --reason \"<r>\"")
    })?;
    let reason = match inv.value("--reason") {
        Some(r) if !r.trim().is_empty() => r.trim().to_string(),
        _ => {
            return Err(
                CliError::new(7, "--reason is required to close a task").with_hint(format!(
                    "ank close {prefix} --reason \"superseded by the new pipeline\""
                )),
            )
        }
    };

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let home = store.log_home(&loaded);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not a task")));
    };
    let id = task.id.clone();
    task.status
        .check_transition(TaskStatus::Closed)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint(format!("ank show {id}")))?;
    task.status = TaskStatus::Closed;
    let entry = LogEntry {
        timestamp: claim::now_utc(),
        who: identity.to_string(),
        message: format!("closed: {reason}"),
    };
    store.write_with_log(home, &Entity::Task(task), &entry, base_version)?;

    let revoked = claim::delete(&repo.root, &id)?;
    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"status\":\"closed\",\"claim_revoked\":{revoked}}}"
        );
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} -> {}",
            inv.style().retracted("closed"),
            inv.style().id(&id.to_string()),
            inv.style().landed("closed")
        );
        if revoked {
            let _ = writeln!(out, "the active claim was revoked");
        }
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// attest
// ---------------------------------------------------------------------------

/// Adds a proof to a task already `done` (§3).
///
/// §3 allows exactly one write to a task after completion, and it is an
/// *addition* to the `proof` list. Until now nothing implemented it: `done`
/// wants a live claim and a task that is not finished, `claim` refuses one that
/// is, so the only permitted write on a finished entity had no command and was
/// performed by opening the file — twice, in 98dbf3b and in TASK-c2fae25adc66.
///
/// That is the argument for the verb. An append done by hand is
/// indistinguishable, in the resulting file, from a substitution done by hand:
/// the append-only rule was a sentence in the specification with nothing to
/// observe it. Here the entries already present are carried over untouched by
/// construction, and the version bump goes through the store's compare-and-swap.
///
/// **Outside the loop, and not as a consolation.** Attesting to a task somebody
/// else finished is not loop work, and SKILL.md does not teach it. That is a
/// fact about what an agent is taught, never a refusal: `attest` runs for
/// whoever types it, and what stops an agent grading itself twice is the state
/// the verb refuses on, not the identity of the caller.
///
/// `tree` and `verifier` stay empty: this records an attestation made
/// elsewhere, not a run Ank performed. Claiming either would be the overstated
/// proof that TASK-c2fae25adc66 existed to remove.
///
/// The `--proof` grammar is [`crate::done::submitted_proof`], shared with
/// `done` rather than copied beside it. Both verbs therefore accept the same
/// types, return the same codes and check a commit against git the same way, by
/// construction — while each still names itself in its own hints, because
/// `ank done --proof commit:<sha>` is not the command an `attest` caller needs.
pub fn attest(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(1, "attest expects an id")
            .with_hint("ank attest <id> --proof test:<ci-run-ref>")
    })?;

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let home = store.log_home(&loaded);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not a task")));
    };
    let id = task.id.clone();

    // A prerequisite, not a transition: nothing about the status changes here.
    // The verb that applies to an unfinished task is `done`, and the refusal
    // says so rather than leaving the reader to guess.
    if task.status != TaskStatus::Done {
        return Err(CliError::new(
            7,
            format!(
                "{id} is {}, and attest applies to a finished task",
                task.status.as_str()
            ),
        )
        .with_hint(format!("ank done {id}")));
    }

    let usage = crate::done::ProofUsage {
        command: format!("ank attest {id}"),
        purpose: format!("attest {id}"),
    };
    let proof =
        crate::done::submitted_proof(inv, &repo.root, &usage, task.done_criteria.as_deref())?;
    let kind = proof.proof_type.as_str().to_string();
    let reference = proof.reference.clone();

    if inv.has("--detached") {
        return detached(inv, repo, &id, &proof, identity, out);
    }

    // Appended. The entries already there are not read, rewritten or reordered
    // — they are simply still in the vector.
    task.proof.push(proof);
    let entry = LogEntry {
        timestamp: claim::now_utc(),
        who: identity.to_string(),
        message: format!("attested {kind}:{reference}"),
    };
    let entries = task.proof.len();
    store.write_with_log(home, &Entity::Task(task), &entry, base_version)?;

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"appended\":{{\"type\":\"{kind}\",\"ref\":{}}},\"proofs\":{entries}}}",
            json_str(&reference)
        );
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {kind}:{reference} ({entries} proofs)",
            inv.style().advanced("attested"),
            inv.style().id(&id.to_string())
        );
    }
    Ok(0)
}

/// `--detached`: the attestation goes to `refs/ank/proof/<id>` and no file is
/// touched (ADR-493471d64ba0).
///
/// **The point is what does not happen.** A green pipeline is a statement about
/// a tree, made by an environment nobody working on a branch controls, and §12
/// forbids ank from committing. Requiring that statement to arrive as a commit
/// put the most trustworthy proof behind the least trustworthy delivery, and in
/// practice meant it never arrived. So nothing here writes to `.ank/`, nothing
/// bumps a version, and `git status` after this command is what it was before.
///
/// No log entry either, for the same reason and not as an omission: the log is
/// a file. What a log line would have carried — who, when — is in the record.
///
/// **And that is why this is the one verb that fails when the push does not
/// land** (ADR-af533e7a3e03). Everything above is an argument that the ref is
/// the whole product: nothing was written to disk that still means something,
/// so a proof that reached no remote is readable by nobody and the attestation
/// did not happen. `claim` writes a ref too and stays on the degrade side, and
/// the difference is not what the verb touches but what survives the failure.
fn detached(
    inv: &Invocation,
    repo: &Repo,
    id: &EntityId,
    proof: &ank_core::Proof,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let kind = proof.proof_type.as_str();
    let reference = proof.reference.clone();
    let (written, entries) = claim::attach_proof(&repo.root, id, proof, identity)?;
    if written.cas == claim::Cas::Lost {
        // Two attestations racing on one task. ADR-493471d64ba0 argues this is
        // not a case anybody meets — one ref per task, and pipelines attest
        // different tasks — and "not met" is not "cannot happen", so it is an
        // ordinary lost swap naming the retry rather than a silent overwrite.
        return Err(
            CliError::new(4, format!("another attestation reached {id} first")).with_hint(format!(
                "ank attest {id} --proof {kind}:{reference} --detached"
            )),
        );
    }

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"attached\":{{\"type\":\"{kind}\",\"ref\":{}}},\
             \"detached_proofs\":{entries},\"pushed\":{}}}",
            json_str(&reference),
            written.sync == claim::Sync::Pushed
        );
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {kind}:{reference} ({entries} detached)",
            inv.style().advanced("attested"),
            inv.style().id(&id.to_string())
        );
    }
    // **The report is printed first, and then the verb fails**
    // (ADR-af533e7a3e03). A proof is a ref, so a push that never landed leaves
    // it readable by nobody and there is no degraded mode to fall back to: code
    // 9, an environment to repair rather than a failure of the work. What is
    // printed above is still true and still owed — `--json` carries
    // `"pushed":false`, which is the same fact the exit code carries, and an
    // integration reading the flag must not be able to disagree with one
    // reading the code.
    //
    // The hint is a push and not a re-run: the local swap succeeded, so the
    // record is already in this clone and one command finishes the job.
    if let Some(failure) = written.sync.proof_failure() {
        return Err(CliError::new(9, failure)
            .with_hint(format!("git push origin {}", claim::proof_ref(id))));
    }
    Ok(0)
}

// ---------------------------------------------------------------------------
// amend
// ---------------------------------------------------------------------------

/// Changes the two fields of an existing entity that no other command reaches:
/// `blocked_by` and `scope`.
///
/// Both were edited by hand during the session that produced this verb — a
/// blocker added to a task already filed, and a scope that omitted six files
/// without which its own task could not compile. Both edits were legitimate.
/// Neither was checkable by anything, which is the argument TASK-1f4f7b57039b
/// already made for `attest`: an edit performed by hand is indistinguishable,
/// in the resulting file, from any other edit performed by hand. A verb can
/// guarantee that the fields it did not name come back byte-identical.
///
/// **Outside the loop.** Adding a blocker to a task that already exists is a
/// change to a plan that is not yours, which is TASK-bc214fd815b2's reasoning
/// for keeping it out of what SKILL.md teaches. Keeping it out of the teaching
/// is the whole of it: the verb itself refuses nobody on identity.
///
/// **Add and remove, never replace.** A verb taking the whole list silently
/// drops whatever the caller forgot to repeat, and the round-trip guarantees
/// nothing about intent. Naming what enters and what leaves is also what makes
/// the log line worth reading afterwards.
pub fn amend(inv: &Invocation, repo: &Repo, identity: &str, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(1, "amend expects an id").with_hint("ank amend <id> --blocked-by <id>")
    })?;

    // Blank is refused rather than treated as a removal: a task with no
    // criterion cannot be claimed at all (§3), so emptying the field through a
    // flag that reads as an edit would be a state nobody asked for.
    let criteria = match inv.value("--criteria") {
        Some(c) if c.trim().is_empty() => {
            return Err(CliError::new(
                7,
                "--criteria is empty, and a task with no done_criteria cannot be claimed",
            )
            .with_hint(format!(
                "ank amend {prefix} --criteria \"<verifiable criterion>\""
            )))
        }
        Some(c) => Some(claim::ensure_trailing_newline(c.trim())),
        None => None,
    };

    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = version_of(&loaded.entity);
    let home = store.log_home(&loaded);
    let id = loaded.entity.id().clone();

    // Both normalised, and both for the same reason `new --scope` is: one is
    // written into the entity, and the other is compared against what is
    // already stored there, which is now always a normal form. A raw
    // `--drop-scope .\docs\**` would match no stored glob and be refused as
    // absent from a scope that carries it (TASK-8dd89053fa33).
    let add_scope = crate::context::normalised_globs(
        inv.values("--scope"),
        repo,
        &format!("ank amend {prefix} --scope"),
    )?;
    let drop_scope = crate::context::normalised_globs(
        inv.values("--drop-scope"),
        repo,
        &format!("ank amend {prefix} --drop-scope"),
    )?;
    let add_blocked = resolve_all(&store, inv.values("--blocked-by"))?;
    let drop_blocked = resolve_all(&store, inv.values("--drop-blocked-by"))?;

    if add_scope.is_empty()
        && drop_scope.is_empty()
        && add_blocked.is_empty()
        && drop_blocked.is_empty()
        && criteria.is_none()
    {
        return Err(
            CliError::new(7, format!("nothing to amend on {id}")).with_hint(format!(
                "ank amend {id} --blocked-by <id> | --drop-blocked-by <id> | \
                 --scope <glob> | --drop-scope <glob> | --criteria \"<c>\""
            )),
        );
    }

    let mut changes: Vec<String> = Vec::new();

    match loaded.entity {
        Entity::Task(mut task) => {
            // §3 allows exactly one write to a task after completion, and it is
            // an addition to the proof list — which is `attest`, not this.
            // Amending a finished task would produce the corpus fault `check`
            // reports as "done task modified beyond appending a proof".
            if matches!(task.status, TaskStatus::Done | TaskStatus::Closed) {
                return Err(CliError::new(
                    7,
                    format!("{id} is {}, and its plan is settled", task.status.as_str()),
                )
                .with_hint(format!("ank show {id}")));
            }

            for b in &drop_blocked {
                if !task.blocked_by.contains(b) {
                    return Err(CliError::new(7, format!("{b} does not block {id}"))
                        .with_hint(format!("ank show {id}")));
                }
            }
            task.blocked_by.retain(|b| !drop_blocked.contains(b));
            for b in add_blocked {
                if b == id {
                    return Err(CliError::new(
                        7,
                        format!("{id} cannot block itself: that is a cycle of one"),
                    )
                    .with_hint(format!("ank amend {id} --blocked-by <other-id>")));
                }
                if !task.blocked_by.contains(&b) {
                    changes.push(format!("+blocked_by {b}"));
                    task.blocked_by.push(b);
                }
            }
            for b in &drop_blocked {
                changes.push(format!("-blocked_by {b}"));
            }

            amend_scope(&mut task.scope, &add_scope, &drop_scope, &id, &mut changes)?;

            // The criterion, refused while a live claim anchors it and allowed
            // the rest of the time (§4). A criterion under no claim is anchored
            // by nothing: there is no freeze to respect, and `check` has nothing
            // to notice. Under one, the refusal is the freeze doing its work,
            // and what a criterion discovered wrong mid-work calls for is a
            // release.
            //
            // The case this route exists for is the criterion that turns out
            // *unmeasurable* rather than wrong (TASK-7c2fa14284ff). Before it,
            // the correction had two ways in and both were wrong: a hand edit,
            // which this verb exists to make unnecessary, or `claim --criteria`,
            // which recorded a creator's correction as the claimer's.
            if let Some(criteria) = criteria {
                if let Some(held) = claim::live(&repo.root, &id)? {
                    return Err(CliError::new(
                        6,
                        format!(
                            "done_criteria is frozen by the claim {} holds on {id}",
                            held.holder
                        ),
                    )
                    .with_hint("ank release --reason \"<why the criterion is wrong>\""));
                }
                if task.done_criteria.as_deref() != Some(criteria.as_str()) {
                    changes.push("done_criteria".to_string());
                    task.done_criteria = Some(criteria);
                    // `criteria_by` is deliberately left where it stands. It
                    // answers whether the criterion was set at claim time by the
                    // party the freeze constrains (§3), and an amend is not a
                    // claim: writing `claimer` would launder a correction into
                    // the shape the signal exists to expose, and writing
                    // `creator` would assert something about the caller, which
                    // nothing else on this surface does. The log entry below is
                    // what records the amend, as it does for the other fields.
                }
            }

            if changes.is_empty() {
                return Err(CliError::new(7, format!("{id} already reads that way"))
                    .with_hint(format!("ank show {id}")));
            }

            // A scope change moves which ADRs bear on the work, and the claim
            // record anchors the hash of exactly that set — `done` compares
            // against it and warns. Refusing would be wrong, since a scope
            // discovered false mid-task is what produced this verb; allowing it
            // silently would be worse.
            let touched_scope = !add_scope.is_empty() || !drop_scope.is_empty();
            let holder = match claim::read(&repo.root, &id)?.map(|h| h.record) {
                Some(Record::Claim(c)) => Some(c.holder),
                _ => None,
            };

            let entry = LogEntry {
                timestamp: claim::now_utc(),
                who: identity.to_string(),
                message: format!("amended: {}", changes.join(", ")),
            };
            store.write_with_log(home, &Entity::Task(task), &entry, base_version)?;

            report_amend(inv, &id, &changes, out);
            if touched_scope {
                if let Some(holder) = holder {
                    // Standard error: not the answer, and stdout under `--json`
                    // is a parser's input (§4, TASK-2eefcdd80124).
                    eprintln!(
                        "{} {id} is held by {holder}, and the scope change moves \
                         the constraints its claim anchors",
                        inv.style().on_stderr().yellow("warning:")
                    );
                }
            }
        }
        Entity::Adr(mut adr) => {
            if !add_blocked.is_empty() || !drop_blocked.is_empty() {
                return Err(CliError::new(
                    1,
                    "blocked_by applies to a task: an ADR blocks nothing",
                )
                .with_hint(format!("ank amend {id} --scope <glob>")));
            }
            // Refused rather than dropped, for the reason `new` refuses
            // `--verify` on an ADR: a flag silently ignored teaches the caller
            // it worked. An ADR is measured by nothing — what it carries is a
            // constraint, and changing that one is a succession.
            if criteria.is_some() {
                return Err(CliError::new(
                    1,
                    "done_criteria applies to a task: an ADR declares no criterion",
                )
                .with_hint(format!("ank amend {id} --scope <glob>")));
            }
            // `constraint` and `scope` are hashed into the ratification commit
            // (§8), so amending the scope of an accepted ADR would diverge from
            // the anchor and `check` would call it altered — while suspending
            // its injection into `context`. The succession is the way to change
            // a ratified decision, and it has its own verb.
            if adr.status != AdrStatus::Proposed {
                return Err(CliError::new(
                    6,
                    format!(
                        "{id} is {} and its scope is anchored in the ratification commit",
                        adr.status.as_str()
                    ),
                )
                .with_hint(
                    "ank new adr --supersedes <id> --title \"<t>\" --scope \"<glob>\" \
                     --constraint \"<rule>\"",
                ));
            }

            amend_scope(&mut adr.scope, &add_scope, &drop_scope, &id, &mut changes)?;
            if changes.is_empty() {
                return Err(CliError::new(7, format!("{id} already reads that way"))
                    .with_hint(format!("ank show {id}")));
            }
            // An ADR has no log section, so the change is recorded by `version`
            // and by the diff, which is what every other write to an ADR does.
            store.write(&Entity::Adr(adr), base_version)?;
            report_amend(inv, &id, &changes, out);
        }
    }

    Ok(0)
}

/// The scope half, shared by both kinds.
///
/// A scope that empties out is refused for the reason `new` refuses one at
/// creation: attachment happens through `scope` and nothing else, so an entity
/// without one appears in no `context` and nobody finds it again.
fn amend_scope(
    scope: &mut Vec<String>,
    add: &[String],
    drop: &[String],
    id: &EntityId,
    changes: &mut Vec<String>,
) -> Result<()> {
    for g in drop {
        if !scope.iter().any(|s| s == g) {
            return Err(
                CliError::new(7, format!("'{g}' is not in the scope of {id}"))
                    .with_hint(format!("ank show {id}")),
            );
        }
    }
    scope.retain(|s| !drop.iter().any(|d| d == s));
    for g in add {
        if !scope.iter().any(|s| s == g) {
            changes.push(format!("+scope {g}"));
            scope.push(g.clone());
        }
    }
    for g in drop {
        changes.push(format!("-scope {g}"));
    }
    if scope.is_empty() {
        return Err(CliError::new(
            7,
            format!("{id} would be left with no scope, and attach to nothing"),
        )
        .with_hint(format!("ank amend {id} --scope \"<glob>\"")));
    }
    // Validated here rather than trusted: a glob that does not compile would
    // otherwise surface in `check` as a corpus fault nobody can attribute.
    ank_core::scope::validate_globs(scope).map_err(|e| {
        CliError::new(7, format!("{e}")).with_hint("ank amend <id> --scope \"src/**\"")
    })
}

fn trimmed(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Every reference resolved at the point of the edit. An unknown one refused
/// here rather than in `check`, where nobody can attribute it to the act that
/// caused it — the same doctrine `new` applies to `--blocked-by`.
fn resolve_all(store: &Store, raw: &[String]) -> Result<Vec<EntityId>> {
    let mut out = Vec::new();
    for r in raw {
        let id = store.resolve(r.trim())?;
        if !out.contains(&id) {
            out.push(id);
        }
    }
    Ok(out)
}

fn report_amend(inv: &Invocation, id: &EntityId, changes: &[String], out: &mut dyn Write) {
    if inv.json() {
        let items: Vec<String> = changes.iter().map(|c| json_str(c)).collect();
        let _ = writeln!(
            out,
            "{{\"entity\":\"{id}\",\"amended\":[{}]}}",
            items.join(",")
        );
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "{} {} {}",
            inv.style().advanced("amended"),
            inv.style().id(&id.to_string()),
            changes.join(" ")
        );
    }
}

/// The whole entity, verbatim, and on a task the two directions of
/// `blocked_by` (§4).
///
/// Everything above the sections is byte for byte what is on disk; everything
/// else in the tool summarises, and this is still the one command that does
/// not. The sections under it are derived from the corpus at read time and
/// stored nowhere — a stored reverse edge is a second copy of `blocked_by` that
/// can disagree with the first, and the copy that disagrees is the one a reader
/// happens to open.
///
/// **Both headings print even at zero.** An absent heading and a heading with
/// nothing under it would be the same page, and only one of the two is an
/// answer to *what waits on this*.
pub fn show(inv: &Invocation, repo: &Repo, out: &mut dyn Write) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "show expects an id").with_hint("ank show <id>"))?;
    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let text = serialize_entity(&loaded.entity);
    // An ADR has no `blocked_by` to have two directions of, so it costs nothing
    // and the index is never opened for one.
    let edges = match &loaded.entity {
        Entity::Task(t) => Some(edges_of(repo, t)?),
        Entity::Adr(_) => None,
    };
    // The union of §3's proof list with what the proof ref carries
    // (ADR-493471d64ba0). Empty for an ADR, which is measured by nothing, and
    // empty for the great majority of tasks — one `rev-parse` that answers
    // "absent", which is what a task with no attestation costs.
    let detached = match &loaded.entity {
        Entity::Task(t) => claim::detached_proofs(&repo.root, &t.id),
        Entity::Adr(_) => Vec::new(),
    };
    // Only when the log is a file. A body still carrying its own `## Log`
    // section prints it above as part of the entity, and a second copy here
    // would be the same history said twice.
    let log = match store.log_home(&loaded) {
        LogHome::File => store.log_of(&loaded)?,
        LogHome::Body => Vec::new(),
    };

    if inv.json() {
        let state = match claim::read(&repo.root, loaded.entity.id())?.map(|h| h.record) {
            Some(Record::Claim(c)) => format!("\"claimed by {}\"", c.holder),
            Some(Record::Completed(c)) => {
                format!("\"finished at {}\"", &c.commit[..7.min(c.commit.len())])
            }
            // Reported and never coerced: the claim namespace carrying an
            // attestation is a damaged plane, and answering `null` would
            // present the task as free.
            Some(Record::Proof(_)) => "\"a proof record on the claim ref\"".to_string(),
            None => "null".to_string(),
        };
        let derived = match &edges {
            Some((blocked_by, unblocks)) => format!(
                ",\"blocked_by\":{},\"unblocks\":{}",
                edges_json(blocked_by),
                edges_json(unblocks)
            ),
            None => String::new(),
        };
        let _ = writeln!(
            out,
            "{{\"id\":\"{}\",\"coordination\":{state}{derived},\"detached_proofs\":{},\"log\":{},\"content\":{}}}",
            loaded.entity.id(),
            detached_json(&detached),
            log_json(&log),
            json_str(&text)
        );
    } else if !inv.quiet() {
        // Painted here and not above: `--json` carries the entity as data and
        // must keep receiving `text` itself. `cli::dispatch` already forces the
        // style to PLAIN under `--json`, so this is the second of two
        // independent guards rather than the only one.
        let _ = write!(out, "{}", crate::paint::entity(&text, inv.style()));
        // The log, from wherever it is. Since schema 3 it is a file of its own,
        // so an entity printed verbatim no longer carries it and `show` would
        // display an empty history for a task that has one — silently, which is
        // the failure the version bump exists to prevent (§3). Under the entity
        // and not inside it: what is above stays byte for byte the file.
        log_section(out, &log, inv.style());
        if let Some((blocked_by, unblocks)) = &edges {
            edge_section(out, "BLOCKED BY", blocked_by, inv.style());
            edge_section(out, "UNBLOCKS", unblocks, inv.style());
        }
        if let Entity::Task(t) = &loaded.entity {
            proof_section(out, t, &detached, inv.style());
        }
    }
    Ok(0)
}

/// The log as data. A list, empty when there is none, so a parser reads one
/// shape rather than two.
fn log_json(entries: &[LogEntry]) -> String {
    let items: Vec<String> = entries
        .iter()
        .map(|e| {
            format!(
                "{{\"timestamp\":{},\"who\":{},\"message\":{}}}",
                json_str(&e.timestamp),
                json_str(&e.who),
                json_str(&e.message)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// The entity's log, printed under it and never inside it.
///
/// **Silent when there is none**, which keeps `show` on an ADR adding nothing
/// and keeps a task nobody has logged against from growing an empty heading.
/// A body that still carries its own `## Log` section prints it above as part
/// of the entity and gets no second copy here: [`Store::log_of`] answers from
/// one place, never from both.
fn log_section(out: &mut dyn Write, entries: &[LogEntry], style: crate::style::Style) {
    if entries.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\n{}",
        style.header(&format!("LOG ({})", entries.len()))
    );
    for (i, e) in entries.iter().enumerate() {
        let connector = if i + 1 == entries.len() {
            crate::style::glyph::LAST
        } else {
            crate::style::glyph::BRANCH
        };
        let _ = writeln!(out, "{connector}{} {} — {}", e.timestamp, e.who, e.message);
    }
}

/// Every proof against the task, from both sources, each saying which it is
/// (ADR-493471d64ba0).
///
/// **Printed only when something is detached**, and the asymmetry is about
/// redundancy rather than about trust. The frontmatter above already lists the
/// file's proofs byte for byte, so a section repeating them under every `show`
/// of every finished task would be noise on the common path. What cannot be
/// seen anywhere else is the union, and the union only exists when a ref
/// carries something — at which point both halves are listed together, on the
/// same lines, because a reader comparing them is exactly who this is for.
fn proof_section(
    out: &mut dyn Write,
    task: &Task,
    detached: &[claim::AttestedProof],
    style: crate::style::Style,
) {
    if detached.is_empty() {
        return;
    }
    let total = task.proof.len() + detached.len();
    let _ = writeln!(out, "\n{}", style.header(&format!("PROOFS ({total})")));
    // `file` and `detached` and not a mark on one of them alone: naming only
    // the unusual case would make the display say which to prefer, and §3's
    // list is not more authoritative than the ref. Between a `done` landing and
    // its merge, the ref is the only place the reference exists at all.
    let rows = task
        .proof
        .iter()
        .map(|p| {
            (
                "file".to_string(),
                p.proof_type.as_str(),
                p.reference.clone(),
            )
        })
        .chain(detached.iter().map(|a| {
            (
                format!("detached, {} {}", a.identity, a.attested),
                a.proof.proof_type.as_str(),
                a.proof.reference.clone(),
            )
        }))
        .collect::<Vec<_>>();
    for (i, (origin, kind, reference)) in rows.iter().enumerate() {
        let connector = if i + 1 == rows.len() {
            crate::style::glyph::LAST
        } else {
            crate::style::glyph::BRANCH
        };
        let _ = writeln!(out, "{connector}{kind}:{reference}  ({origin})");
    }
}

fn detached_json(detached: &[claim::AttestedProof]) -> String {
    let items: Vec<String> = detached
        .iter()
        .map(|a| {
            format!(
                "{{\"type\":\"{}\",\"ref\":{},\"by\":{},\"at\":\"{}\"}}",
                a.proof.proof_type.as_str(),
                json_str(&a.proof.reference),
                json_str(&a.identity),
                a.attested
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

/// One end of a `blocked_by` edge, resolved against the corpus.
///
/// `status` is `None` when the reference resolves to nothing. A dangling
/// `blocked_by` is a fault `check` reports; `show` still prints the line it
/// could not resolve, because dropping it would produce a shorter list and a
/// shorter list is a wrong answer to a question about what blocks this.
struct Edge {
    id: EntityId,
    short: String,
    /// The stored status, and only ever that: it is what `--json` carries, and
    /// the machine surface does not move because a human listing learned to say
    /// something better.
    status: Option<String>,
    /// What the human line prints — the stored status seen through the
    /// coordination plane, brackets included.
    marker: Option<String>,
    title: Option<String>,
}

/// What blocks this task, and what this task directly unblocks.
///
/// The reverse direction is not filtered by status: `graph` draws the whole
/// edge set and this is the narrow view onto the same derivation, so a blocker
/// that is already `done` and a task that is already `done` both keep their
/// line and carry their status. The §5 ordering counts something else — how
/// many tasks are still *held up* — and a count is not a list.
fn edges_of(repo: &Repo, task: &Task) -> Result<(Vec<Edge>, Vec<Edge>)> {
    let index = Index::open(&repo.ank)?;
    let all = index.all()?;
    let shorts = crate::context::shorts_of(repo)?;
    let row_of: HashMap<&EntityId, &crate::index::Row> = all.iter().map(|r| (&r.id, r)).collect();
    // The same coordination every other listing reads, so a blocker that is
    // claimed says so here too instead of reading `[in_progress]` at a reader
    // who has just been told `[claimed:who]` by `context`.
    let coord = crate::context::coordination(&repo.root, &mut Vec::new())?;

    let edge = |id: &EntityId| -> Edge {
        let row = row_of.get(id);
        Edge {
            id: id.clone(),
            short: shorts.get(id).cloned().unwrap_or_else(|| id.to_string()),
            status: row.map(|r| r.status.clone()),
            // The whole marker, brackets included, because it is the marker
            // that varies and not just the word inside it.
            marker: row.map(|r| {
                crate::context::marker_for(&r.status, crate::context::coordination_of(&coord, id))
            }),
            title: row.map(|r| r.title.clone()),
        }
    };

    // Declared order for the blockers: the frontmatter printed just above says
    // the same thing in the same order, and two orders for one list is a
    // difference a reader has to account for before trusting either.
    let blocked_by: Vec<Edge> = task.blocked_by.iter().map(&edge).collect();

    // The reverse direction is derived and has no declared order, so it takes
    // the one every other listing uses.
    let mut waiting: Vec<&crate::index::Row> = all
        .iter()
        .filter(|r| r.kind == EntityKind::Task && r.blocked_by.contains(&task.id))
        .collect();
    waiting.sort_by_key(|r| r.id.to_string());
    let unblocks: Vec<Edge> = waiting.iter().map(|r| edge(&r.id)).collect();

    Ok((blocked_by, unblocks))
}

fn edge_section(out: &mut dyn Write, heading: &str, edges: &[Edge], style: crate::style::Style) {
    let _ = writeln!(
        out,
        "\n{}",
        style.header(&format!("{heading} ({})", edges.len()))
    );
    // One level deep by construction — these are the two directions of
    // `blocked_by` for a single task, not a walk — so the alphabet reduces to
    // its two connectors and no gutter is ever needed under them (§4).
    for (i, e) in edges.iter().enumerate() {
        let connector = if i + 1 == edges.len() {
            crate::style::glyph::LAST
        } else {
            crate::style::glyph::BRANCH
        };
        match (&e.marker, &e.title) {
            (Some(marker), Some(title)) => {
                let _ = writeln!(
                    out,
                    "{connector}{}  {} {title}",
                    style.id(&e.short),
                    style.status(marker)
                );
            }
            _ => {
                let _ = writeln!(out, "{connector}{}  (no such entity)", style.id(&e.short));
            }
        }
    }
}

fn edges_json(edges: &[Edge]) -> String {
    let items: Vec<String> = edges
        .iter()
        .map(|e| {
            let field = |v: &Option<String>| match v {
                Some(s) => json_str(s),
                None => "null".to_string(),
            };
            format!(
                "{{\"id\":\"{}\",\"short\":\"{}\",\"status\":{},\"title\":{}}}",
                e.id,
                e.short,
                field(&e.status),
                field(&e.title)
            )
        })
        .collect();
    format!("[{}]", items.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{Proof, ProofType};
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-human-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/entities")).unwrap();
            std::fs::create_dir_all(p.join("src")).unwrap();
            std::fs::write(p.join("src/a.rs"), "fn a() {}\n").unwrap();
            let t = Temp(p);
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "test@ank.local"],
                vec!["config", "user.name", "Test"],
                vec!["config", "core.autocrlf", "false"],
                // Signing off at creation, not at each commit
                // (TASK-40a972e98a9a). `accept` passes `-S`, which outranks
                // this, so the fixtures that sign for real still do.
                vec!["config", "commit.gpgsign", "false"],
                vec!["config", "tag.gpgsign", "false"],
            ] {
                assert!(Command::new("git")
                    .current_dir(&t.0)
                    .args(&args)
                    .status()
                    .unwrap()
                    .success());
            }
            std::fs::write(
                t.0.join(".ank/config.yml"),
                "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\ndefault_branch: main\n",
            )
            .unwrap();
            t
        }

        fn repo(&self) -> Repo {
            Repo {
                root: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }
        fn cfg(&self) -> Config {
            crate::config::load(&self.repo().config_path()).unwrap()
        }
        /// The log as a reader gets it, from wherever this entity keeps it.
        fn log(&self, id: &EntityId) -> Vec<LogEntry> {
            let store = self.store();
            let loaded = store.load(id).unwrap();
            store.log_of(&loaded).unwrap()
        }

        fn store(&self) -> Store {
            Store::new(self.0.join(".ank"))
        }
        fn write(&self, e: &Entity) {
            std::fs::write(self.store().path_of(e.id()), serialize_entity(e)).unwrap();
        }
        fn commit(&self, msg: &str) {
            for args in [vec!["add", "-A"], vec!["commit", "-qm", msg]] {
                Command::new("git")
                    .current_dir(&self.0)
                    .args(&args)
                    .status()
                    .unwrap();
            }
        }

        fn git_ok(&self, args: &[&str]) {
            let out = Command::new("git")
                .current_dir(&self.0)
                .args(args)
                .output()
                .expect("git must be installed: it is a hard dependency");
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }

        /// The same, keeping standard output. Trailing whitespace only is
        /// trimmed, so a commit object comes back byte for byte.
        fn git_out(&self, args: &[&str]) -> String {
            let out = Command::new("git")
                .current_dir(&self.0)
                .args(args)
                .output()
                .expect("git must be installed: it is a hard dependency");
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim_end().to_string()
        }

        /// A real signing key, because `accept` signs for real and the two
        /// tests that existed before this one both asserted refusals — they
        /// never reached the commit, so the commit was never exercised.
        ///
        /// SSH rather than GPG: `ssh-keygen` ships beside git on all three
        /// platforms and needs no agent, no keyring and no passphrase prompt,
        /// where a gpg fixture needs a home directory and a daemon.
        fn enable_signing(&self) {
            let key = self.0.join("signing-key");
            let out = Command::new("ssh-keygen")
                .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
                .arg(&key)
                .output()
                .expect("ssh-keygen ships with git on all three platforms");
            assert!(
                out.status.success(),
                "ssh-keygen: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            for args in [
                vec!["config", "gpg.format", "ssh"],
                vec![
                    "config",
                    "user.signingkey",
                    key.with_extension("pub").to_str().unwrap(),
                ],
            ] {
                assert!(Command::new("git")
                    .current_dir(&self.0)
                    .args(&args)
                    .status()
                    .unwrap()
                    .success());
            }
        }

        fn adr_at(&self, hex: &str) -> Adr {
            let id = EntityId::parse(&format!("ADR-{hex}")).unwrap();
            match self.store().load(&id).unwrap().entity {
                Entity::Adr(a) => a,
                _ => panic!("not an ADR"),
            }
        }
        fn report(&self) -> Report {
            inspect(&self.repo(), &self.cfg(), None, false).unwrap()
        }
        fn claim_as(&self, id: &EntityId, who: &str, criteria: &str) {
            let Entity::Task(task) = self.store().load(id).unwrap().entity else {
                panic!("not a task")
            };
            claim::acquire(
                &self.0,
                &task,
                who,
                std::time::Duration::from_secs(1800),
                &freeze::freeze_hash_short(criteria),
                &claim::constraints_hash(
                    &claim::applicable_constraints(&self.store(), &self.repo(), &task).unwrap(),
                ),
                None,
            )
            .unwrap();
        }
        fn call(&self, argv: &[&str], who: &str) -> Result<(i32, String)> {
            let argv: Vec<String> = argv.iter().map(|a| a.to_string()).collect();
            let inv = crate::cli::parse(&argv).unwrap();
            let mut out = Vec::new();
            let repo = self.repo();
            let cfg = self.cfg();
            let code = match inv.command {
                "check" => check(&inv, &repo, &cfg, &mut out)?,
                "review" => review(&inv, &repo, &cfg, &mut out)?,
                "accept" => accept(&inv, &repo, &cfg, who, &mut out)?,
                "close" => close(&inv, &repo, who, &mut out)?,
                "amend" => amend(&inv, &repo, who, &mut out)?,
                "show" => show(&inv, &repo, &mut out)?,
                other => panic!("not a human verb: {other}"),
            };
            Ok((code, String::from_utf8_lossy(&out).to_string()))
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn task(hex: &str, status: TaskStatus, blocked: &[&str]) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "Example task".into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status,
            scope: vec!["src/**".into()],
            blocked_by: blocked
                .iter()
                .map(|b| EntityId::parse(b).unwrap())
                .collect(),
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: if status == TaskStatus::Done {
                vec![Proof {
                    proof_type: ProofType::Commit,
                    reference: "0123456".into(),
                    tree: None,
                    criteria: None,
                    verifier: None,
                }]
            } else {
                vec![]
            },
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nBody.\n".into(),
        })
    }

    fn adr(hex: &str, status: AdrStatus, scope: &[&str]) -> Entity {
        Entity::Adr(Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "A decision".into(),
            created: "2026-07-20T00:00:00Z".into(),
            author: None,
            status,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            constraint: "A binding rule.\n".into(),
            see: None,
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
        })
    }

    fn has(report: &Report, level: Level, needle: &str) -> bool {
        report
            .findings
            .iter()
            .any(|f| f.level == level && f.message.contains(needle))
    }

    // -----------------------------------------------------------------------
    // check: faults
    // -----------------------------------------------------------------------

    #[test]
    fn a_healthy_corpus_exits_zero() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        let (code, out) = t.call(&["check"], "marie@laptop").unwrap();
        assert_eq!(code, 0, "{out}");
        assert_eq!(t.report().faults(), 0, "{:?}", t.report().findings);
        assert!(out.contains("check: ok"), "{out}");
    }

    #[test]
    fn a_non_canonical_file_and_a_conflict_marker_are_faults() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());
        // Doubled spacing: YAML reads it the same, the serializer writes it the
        // one way, and the round-trip is what notices. Trailing blank lines
        // would not have done — the body is verbatim, so they survive the
        // round-trip and are canonical.
        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replace("status: open", "status:  open")).unwrap();
        let r = t.report();
        assert!(has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);

        std::fs::write(
            t.0.join(".ank/entities/TASK-00000000ffff.md"),
            "---\n<<<<<<< HEAD\nid: TASK-00000000ffff\n=======\nid: other\n>>>>>>> branch\n",
        )
        .unwrap();
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "conflict markers"),
            "{:?}",
            r.findings
        );
        assert_eq!(t.call(&["check"], "m").unwrap().0, 8, "faults exit 8");
    }

    /// The same symptom as the test above -- the round-trip differs -- and the
    /// opposite verdict, because the content is canonical and only the line
    /// endings are not. Failing CI over a git checkout setting would report the
    /// wrong culprit to the wrong person.
    #[test]
    fn crlf_alone_is_a_signal_and_check_still_exits_zero() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());

        let text = std::fs::read_to_string(&p).unwrap();
        std::fs::write(&p, text.replace('\n', "\r\n")).unwrap();

        let r = t.report();
        // Named by its cause, and carrying the command: the file cannot be
        // repaired by editing it while git converts back on every checkout.
        assert!(has(&r, Level::Signal, "CRLF"), "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "git config core.autocrlf input"),
            "{:?}",
            r.findings
        );
        // And explicitly not the other diagnosis.
        assert!(!has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);
        assert!(
            !has(&r, Level::Fault, "missing frontmatter"),
            "{:?}",
            r.findings
        );
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert_eq!(
            t.call(&["check"], "m").unwrap().0,
            0,
            "CRLF alone must not fail the build"
        );
    }

    /// CRLF is the excuse, not the amnesty: a file that is also malformed is
    /// still a fault.
    #[test]
    fn crlf_does_not_excuse_a_genuinely_non_canonical_file() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let p = t
            .store()
            .path_of(&EntityId::parse("TASK-000000000001").unwrap());

        let text = std::fs::read_to_string(&p).unwrap();
        let both = text
            .replace("status: open", "status:  open")
            .replace('\n', "\r\n");
        std::fs::write(&p, both).unwrap();

        let r = t.report();
        assert!(has(&r, Level::Fault, "non-canonical"), "{:?}", r.findings);
        assert_eq!(t.call(&["check"], "m").unwrap().0, 8);
    }

    #[test]
    fn a_blocked_by_cycle_is_reported_once_and_names_the_ring() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            TaskStatus::Open,
            &["TASK-000000000002"],
        ));
        t.write(&task(
            "000000000002",
            TaskStatus::Open,
            &["TASK-000000000001"],
        ));
        let r = t.report();
        let cycles: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("cycle"))
            .collect();
        assert_eq!(
            cycles.len(),
            1,
            "once per cycle, not per member: {cycles:?}"
        );
        assert!(cycles[0].message.contains("TASK-000000000001"));
        assert!(cycles[0].message.contains("TASK-000000000002"));
    }

    #[test]
    fn an_unknown_blocker_and_a_dead_scope_are_faults() {
        let t = Temp::new();
        t.write(&task(
            "000000000001",
            TaskStatus::Open,
            &["TASK-00000000ffff"],
        ));
        assert!(has(&t.report(), Level::Fault, "does not exist"));

        // A task already done that names files which are not there: it claimed
        // to touch code that does not exist.
        let mut dead = task("000000000002", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut dead {
            x.scope = vec!["nowhere/**".into()];
        }
        t.write(&dead);
        let r = t.report();
        assert!(has(&r, Level::Fault, "dead scope"), "{:?}", r.findings);

        // The same scope on a task not yet started is a signal, not a fault:
        // scoping a file the work will create is the normal case, and this
        // repository's own release task does exactly that.
        let mut ahead = task("000000000003", TaskStatus::Open, &[]);
        if let Entity::Task(x) = &mut ahead {
            x.scope = vec!["not/written/yet.rs".into()];
        }
        t.write(&ahead);
        let r = t.report();
        assert!(
            has(&r, Level::Signal, "matches no file yet"),
            "{:?}",
            r.findings
        );
        assert!(
            !r.findings
                .iter()
                .any(|f| f.subject.contains("000000000003") && f.level == Level::Fault),
            "{:?}",
            r.findings
        );

        // An ADR binding nothing is a fault whatever its status: a constraint
        // that matches no file constrains nobody.
        t.write(&adr("00000000cccc", AdrStatus::Proposed, &["gone/**"]));
        let r = t.report();
        assert!(
            r.findings
                .iter()
                .any(|f| f.subject.contains("cccc") && f.level == Level::Fault),
            "{:?}",
            r.findings
        );
    }

    // -----------------------------------------------------------------------
    // check: the three signals that need `author`
    // -----------------------------------------------------------------------

    /// Counted once for the corpus, never per file. The fixtures carry no
    /// author, which is the state of every entity written before the field.
    #[test]
    fn entities_predating_the_author_field_are_counted_once() {
        let t = Temp::new();
        for hex in ["000000000001", "000000000002", "000000000003"] {
            t.write(&task(hex, TaskStatus::Open, &[]));
        }
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));

        let r = t.report();
        let lines: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("predate the author field"))
            .collect();
        assert_eq!(lines.len(), 1, "once for the corpus: {:?}", r.findings);
        assert_eq!(lines[0].level, Level::Signal);
        assert!(lines[0].message.starts_with("4 entities"), "{:?}", lines[0]);
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
    }

    /// §3 accepts flooding without a quota, on the argument that the defence is
    /// visibility rather than restriction. This is that visibility.
    #[test]
    fn a_burst_by_one_identity_is_reported_and_a_steady_pace_is_not() {
        let t = Temp::new();
        // Eleven inside the hour, one over the threshold.
        for i in 0..11 {
            let mut e = task(&format!("0000000000{i:02}"), TaskStatus::Open, &[]);
            if let Entity::Task(x) = &mut e {
                x.author = Some("codex@host-9".into());
                x.created = format!("2026-07-28T00:{:02}:00Z", i * 5);
            }
            t.write(&e);
        }
        // The same volume by another identity, spread across the day: the
        // signal is about one agent's rate, not the corpus filling up.
        for i in 0..11 {
            let mut e = task(&format!("0000000001{i:02}"), TaskStatus::Open, &[]);
            if let Entity::Task(x) = &mut e {
                x.author = Some("marie@laptop".into());
                x.created = format!("2026-07-28T{:02}:00:00Z", i + 1);
            }
            t.write(&e);
        }

        let r = t.report();
        let bursts: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("burst creation"))
            .collect();
        assert_eq!(bursts.len(), 1, "{:?}", r.findings);
        assert!(
            bursts[0].message.contains("codex@host-9"),
            "{:?}",
            bursts[0]
        );
        assert!(!bursts[0].message.contains("marie"), "{:?}", bursts[0]);
        assert_eq!(bursts[0].level, Level::Signal, "reported, never refused");
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
    }

    /// A blocker written by the holder after claiming: the shape of an agent
    /// building itself an excuse, and equally the shape of §3's discovered
    /// subtask. Only a reader knows which, so it is reported.
    #[test]
    fn a_blocker_the_holder_created_after_claiming_is_reported() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task(
            "000000000001",
            TaskStatus::InProgress,
            &["TASK-000000000002", "TASK-000000000003"],
        ));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");

        // Written by the holder, after the claim: the case.
        let mut own = task("000000000002", TaskStatus::Open, &[]);
        if let Entity::Task(x) = &mut own {
            x.author = Some("codex@host-9".into());
            x.created = claim::format_utc(claim::now_secs() + 60);
        }
        t.write(&own);

        // Written by somebody else, at the same moment: not the case, and the
        // distinction is the whole content of the signal.
        let mut other = task("000000000003", TaskStatus::Open, &[]);
        if let Entity::Task(x) = &mut other {
            x.author = Some("marie@laptop".into());
            x.created = claim::format_utc(claim::now_secs() + 60);
        }
        t.write(&other);

        let r = t.report();
        let found: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("created by the holder"))
            .collect();
        assert_eq!(found.len(), 1, "{:?}", r.findings);
        assert_eq!(found[0].level, Level::Signal);
        assert!(found[0].message.contains("TASK-000000000002"), "{found:?}");
        assert!(
            !found[0].message.contains("TASK-000000000003"),
            "another agent's blocker is not the signal: {found:?}"
        );
        assert!(found[0].message.contains("codex@host-9"), "{found:?}");
    }

    /// The blocker predates the claim, so it is not an excuse built after the
    /// fact -- it is a dependency that was already there.
    #[test]
    fn a_blocker_the_holder_created_before_claiming_is_silent() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task(
            "000000000001",
            TaskStatus::InProgress,
            &["TASK-000000000002"],
        ));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");

        let mut before = task("000000000002", TaskStatus::Open, &[]);
        if let Entity::Task(x) = &mut before {
            x.author = Some("codex@host-9".into());
            x.created = claim::format_utc(claim::now_secs() - 86_400);
        }
        t.write(&before);

        let r = t.report();
        assert!(
            !r.findings
                .iter()
                .any(|f| f.message.contains("created by the holder")),
            "{:?}",
            r.findings
        );
    }

    #[test]
    fn a_broken_supersede_chain_is_a_fault_in_both_directions() {
        let t = Temp::new();
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000eeee").unwrap());
        }
        t.write(&a);
        assert!(has(&t.report(), Level::Fault, "does not exist"));

        t.write(&adr("00000000bbbb", AdrStatus::Superseded, &["src/**"]));
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "no ADR supersedes it"),
            "{:?}",
            r.findings
        );
    }

    /// A dangling reference is wrong whoever wrote it, and `proposed` buys no
    /// amnesty for it: the target named here has never existed, so no future
    /// `accept` can repair the chain.
    #[test]
    fn a_proposed_adr_naming_a_target_that_does_not_exist_is_still_a_fault() {
        let t = Temp::new();
        let mut a = adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000eeee").unwrap());
        }
        t.write(&a);
        let r = t.report();
        assert!(has(&r, Level::Fault, "does not exist"), "{:?}", r.findings);
    }

    /// The proposed case, on its own: an announced succession is not a broken
    /// one. `proposed` states an intention, the succession happens at `accept`,
    /// and until then the target is not expected to be marked.
    #[test]
    fn a_proposed_superseder_over_an_unmarked_target_is_a_signal() {
        let t = Temp::new();
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);

        let r = t.report();
        assert!(
            has(&r, Level::Signal, "is not marked superseded"),
            "{:?}",
            r.findings
        );
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        // The consequence is the whole point: exit 8 fails `check-repo`, which
        // nearly every task in this corpus declares, so faulting here would
        // block every `done` in the repository behind one proposal.
        assert_eq!(
            t.call(&["check"], "marie@laptop").unwrap().0,
            0,
            "an intention must not block every done in the repository"
        );
    }

    /// The accepted case, on its own, and the fault is exactly the one that
    /// existed before: the succession is real now, and the target never learned
    /// of it.
    #[test]
    fn an_accepted_superseder_over_an_unmarked_target_is_still_a_fault() {
        let t = Temp::new();
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);

        let r = t.report();
        assert!(
            has(&r, Level::Fault, "is not marked superseded"),
            "{:?}",
            r.findings
        );
        assert_eq!(t.call(&["check"], "marie@laptop").unwrap().0, 8);
    }

    #[test]
    fn a_diverged_frozen_criterion_is_a_fault_against_the_claim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");
        assert_eq!(t.report().faults(), 0, "{:?}", t.report().findings);

        // The file is edited after the claim: the anchor is what makes it show.
        let Entity::Task(mut weakened) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        weakened.done_criteria = Some("Anything at all.\n".into());
        t.store().write(&Entity::Task(weakened), 1).unwrap();
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "diverges from the claim"),
            "{:?}",
            r.findings
        );
    }

    // -----------------------------------------------------------------------
    // check: signals
    // -----------------------------------------------------------------------

    #[test]
    fn signals_are_reported_without_making_the_corpus_fail() {
        let t = Temp::new();
        let mut weak = task("000000000001", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut weak {
            x.criteria_by = Some(CriteriaBy::Claimer);
            x.proof = vec![Proof {
                proof_type: ProofType::Assertion,
                reference: "it works".into(),
                tree: None,
                criteria: None,
                verifier: None,
            }];
        }
        t.write(&weak);
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));

        let r = t.report();
        assert_eq!(r.faults(), 0, "none of these is a fault: {:?}", r.findings);
        assert!(has(&r, Level::Signal, "weak proof"));
        assert!(has(&r, Level::Signal, "set by the claimer"));
        assert!(has(&r, Level::Signal, "no ratification commit"));
        assert!(has(&r, Level::Signal, "no ratification key"));
        assert_eq!(t.call(&["check"], "m").unwrap().0, 0, "signals exit 0");
    }

    /// The three shapes a proof list can take, because the interesting one is
    /// the mixture and it is the one that did not exist before: §3 makes the
    /// list append-only, so "weak and strong together" is the *only* shape a
    /// task closed before `ank done` can ever reach.
    #[test]
    fn a_weak_proof_signals_only_while_nothing_strong_sits_beside_it() {
        fn proof(proof_type: ProofType, reference: &str) -> Proof {
            Proof {
                proof_type,
                reference: reference.into(),
                tree: None,
                criteria: None,
                verifier: None,
            }
        }
        fn with_proofs(id: &str, proofs: Vec<Proof>) -> Entity {
            let mut e = task(id, TaskStatus::Done, &[]);
            if let Entity::Task(x) = &mut e {
                x.proof = proofs;
            }
            e
        }

        let t = Temp::new();
        // Weak alone: signalled, and the wording is the one readers know.
        t.write(&with_proofs(
            "000000000001",
            vec![proof(ProofType::Assertion, "it works")],
        ));
        // Strong alone: nothing to say.
        t.write(&with_proofs(
            "000000000002",
            vec![proof(ProofType::Test, "local/abcdef123456@0000000")],
        ));
        // Both, which is what an append produces. The assertion stays as the
        // record of how the task was actually closed; the task no longer rests
        // on it, so the task is clean.
        t.write(&with_proofs(
            "000000000003",
            vec![
                proof(ProofType::Assertion, "it works"),
                proof(ProofType::Test, "local/abcdef123456@0000000"),
            ],
        ));

        let r = t.report();
        assert_eq!(r.faults(), 0, "none of these is a fault: {:?}", r.findings);

        let weak: Vec<&Finding> = r
            .findings
            .iter()
            .filter(|f| f.message.contains("weak proof"))
            .collect();
        assert_eq!(
            weak.len(),
            1,
            "one task rests on nothing, and only one: {:?}",
            r.findings
        );
        assert!(weak[0].subject.contains("000000000001"), "{:?}", weak[0]);
        assert_eq!(
            weak[0].message,
            "weak proof 'assertion': it anchors nothing"
        );
        assert_eq!(weak[0].level, Level::Signal);

        // human-review is weak too, and a task carrying only weak entries of
        // more than one kind is still one finding, not two.
        let t = Temp::new();
        t.write(&with_proofs(
            "000000000004",
            vec![
                proof(ProofType::HumanReview, "reviewed by a human"),
                proof(ProofType::Assertion, "it works"),
            ],
        ));
        let r = t.report();
        assert_eq!(
            r.findings
                .iter()
                .filter(|f| f.message.contains("weak proof"))
                .count(),
            1,
            "the task is what is being judged, not the entry: {:?}",
            r.findings
        );
    }

    #[test]
    fn a_task_blocked_by_a_closed_one_is_a_signal_for_a_human() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Closed, &[]));
        t.write(&task(
            "000000000002",
            TaskStatus::Open,
            &["TASK-000000000001"],
        ));
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(has(&r, Level::Signal, "which is closed"));
    }

    #[test]
    fn a_verifier_that_moved_since_the_proof_is_a_signal() {
        let t = Temp::new();
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             verifiers:\n  cargo-test:\n    run: cargo test\n",
        )
        .unwrap();
        let mut done = task("000000000001", TaskStatus::Done, &[]);
        if let Entity::Task(x) = &mut done {
            x.proof = vec![Proof {
                proof_type: ProofType::Test,
                reference: "local/aaa@bbb".into(),
                tree: None,
                criteria: None,
                verifier: Some("cargo-test@000000000000".into()),
            }];
        }
        t.write(&done);
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "changed since the proof"),
            "{:?}",
            r.findings
        );
    }

    // -----------------------------------------------------------------------
    // check: pruning
    // -----------------------------------------------------------------------

    #[test]
    fn check_prunes_only_once_the_default_branch_agrees() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.commit("seed");
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");
        claim::complete(&t.0, &id, "codex@host-9").unwrap();

        // The branch has not caught up: kept, and reported.
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert!(r.pruned.is_empty(), "{:?}", r.pruned);
        assert!(
            has(&r, Level::Signal, "has not caught up"),
            "{:?}",
            r.findings
        );
        assert!(claim::read(&t.0, &id).unwrap().is_some());

        // Once it does, the ref has no further use.
        t.write(&task("000000000001", TaskStatus::Done, &[]));
        t.commit("done");
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned.len(), 1, "{:?}", r.pruned);
        assert!(claim::read(&t.0, &id).unwrap().is_none());
    }

    /// The predicate is the file **as the default branch carries it**, and the
    /// working tree is not the branch (ADR-bcf222a31525).
    ///
    /// The sharp half, and the one that decides whether the mechanism works at
    /// all: `done` writes to the working tree, so between the work and the
    /// merge the tree says `done` while the branch does not. Pruning there
    /// would reopen the exact window the completion ref exists to close, in the
    /// exact situation it exists for.
    ///
    /// Carried over from `claim::prune`'s own tests when that second copy of
    /// the predicate was deleted (TASK-4981a1370c0b). The rule had two
    /// implementations and this half was only ever asserted against the one
    /// nothing called.
    #[test]
    fn the_tree_saying_done_is_not_the_branch_saying_done() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.commit("seed");
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");
        claim::complete(&t.0, &id, "codex@host-9").unwrap();

        // The tree says done. The branch has not been told.
        t.write(&task("000000000001", TaskStatus::Done, &[]));
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert!(
            r.pruned.is_empty(),
            "the tree is not the branch: {:?}",
            r.pruned
        );
        assert!(claim::read(&t.0, &id).unwrap().is_some());

        // Committing is what makes it true for everybody else.
        t.commit("done");
        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned.len(), 1, "{:?}", r.pruned);
        assert!(claim::read(&t.0, &id).unwrap().is_none());
    }

    /// `closed` settles a ref exactly as `done` does, and everything else is
    /// left alone.
    ///
    /// Three cases in one fixture, all carried over from `claim::prune`'s tests
    /// (TASK-4981a1370c0b): a closed task prunes, a live claim on an open task
    /// is not maintenance's business, and a task the default branch has never
    /// carried is the unmerged case the ref exists for.
    #[test]
    fn closed_prunes_like_done_and_the_rest_is_left_alone() {
        let t = Temp::new();
        let closed = EntityId::parse("TASK-000000000001").unwrap();
        let live = EntityId::parse("TASK-00000000ffff").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.write(&task("00000000ffff", TaskStatus::InProgress, &[]));
        t.commit("seed");
        t.claim_as(&closed, "codex@host-9", "A verifiable criterion.\n");
        t.claim_as(&live, "claude-code@ank", "A verifiable criterion.\n");

        t.write(&task("000000000001", TaskStatus::Closed, &[]));
        t.commit("closed");

        // A task the branch has never seen: present in the tree, absent there.
        let unmerged = EntityId::parse("TASK-00000000aaaa").unwrap();
        t.write(&task("00000000aaaa", TaskStatus::InProgress, &[]));
        t.claim_as(&unmerged, "claude-code@ank", "A verifiable criterion.\n");

        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned, vec![claim::claim_ref(&closed)], "{:?}", r.pruned);
        assert!(
            claim::read(&t.0, &live).unwrap().is_some(),
            "an open task's claim is not maintenance's business"
        );
        assert!(
            claim::read(&t.0, &unmerged).unwrap().is_some(),
            "a task the default branch has never carried is the unmerged case"
        );
    }

    #[test]
    fn an_orphan_ref_is_pruned_and_review_never_prunes_anything() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        t.commit("seed");
        let ghost = EntityId::parse("TASK-00000000ffff").unwrap();
        let record = Record::Claim(claim::ClaimRecord {
            task: ghost.to_string(),
            holder: "codex@host-9".into(),
            claimed: claim::now_utc(),
            expires: claim::now_utc(),
            ttl: claim::DEFAULT_TTL.as_secs(),
            criteria: "aaaabbbbcccc".into(),
            constraints: "ddddeeeeffff".into(),
        });
        claim::put(&t.0, &ghost, &record, None).unwrap();

        // review inspects without touching the coordination plane.
        let r = inspect(&t.repo(), &t.cfg(), None, false).unwrap();
        assert!(r.pruned.is_empty());
        assert!(has(&r, Level::Signal, "orphan ref"));
        assert!(
            claim::read(&t.0, &ghost).unwrap().is_some(),
            "review kept it"
        );

        let r = inspect(&t.repo(), &t.cfg(), None, true).unwrap();
        assert_eq!(r.pruned.len(), 1);
        assert!(claim::read(&t.0, &ghost).unwrap().is_none());
    }

    // -----------------------------------------------------------------------
    // accept
    // -----------------------------------------------------------------------

    #[test]
    fn accept_refuses_outside_the_default_branch_with_seven() {
        let t = Temp::new();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        assert!(Command::new("git")
            .current_dir(&t.0)
            .args(["switch", "-q", "-c", "feat/opaque-sessions"])
            .status()
            .unwrap()
            .success());

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(
            err.message.contains("feat/opaque-sessions"),
            "{}",
            err.message
        );
        assert!(err.message.contains("main"), "{}", err.message);
        assert!(err.hint.unwrap().contains("git switch main"));

        // Nothing was written: a refusal is not a half transition.
        let Entity::Adr(a) = t
            .store()
            .load(&EntityId::parse("ADR-00000000aaaa").unwrap())
            .unwrap()
            .entity
        else {
            panic!()
        };
        assert_eq!(a.status, AdrStatus::Proposed);
    }

    #[test]
    fn an_indeterminable_default_branch_makes_accept_exit_nine_not_seven() {
        let t = Temp::new();
        std::fs::write(
            t.0.join(".ank/config.yml"),
            "schema: 1\ncontext_budget: 8000\nclaim_ttl_max: 2h\n",
        )
        .unwrap();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(
            err.code, 9,
            "9 is the repository to repair, 7 is the caller in the wrong place: {}",
            err.message
        );
    }

    #[test]
    fn accept_takes_no_bypass_flag() {
        // Tested rather than commented: a --force would become the default path
        // within two weeks, and the only way the constraint survives its own
        // convenience is for this to fail when somebody adds one.
        let spec = crate::cli::spec_of("accept").unwrap();
        for f in spec.flags {
            let n = f.name.to_ascii_lowercase();
            assert!(
                !n.contains("force") && !n.contains("no-verify") && !n.contains("bypass"),
                "accept must carry no bypass: {n}"
            );
        }
        assert!(
            spec.flags.is_empty(),
            "accept takes no flag at all: {:?}",
            spec.flags
        );
    }

    /// A superseder naming nothing: the path that existed before, unchanged,
    /// and now actually reaching the commit it always claimed to make.
    #[test]
    fn accept_ratifies_an_adr_that_replaces_nothing() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");

        let (code, out) = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("accepted ADR-00000000aaaa"), "{out}");
        assert!(!out.contains("superseded"), "nothing was replaced: {out}");

        let a = t.adr_at("00000000aaaa");
        assert_eq!(a.status, AdrStatus::Accepted);
        assert_eq!(
            a.ratified,
            Some(ratification_anchor(&a.constraint, &a.scope))
        );
    }

    /// The write `model.rs` declares legal and nothing performed: accepting the
    /// replacement is what marks the replaced.
    #[test]
    fn accept_marks_the_target_superseded_in_the_same_operation() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);
        t.commit("seed");

        let (code, out) = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("superseded ADR-00000000bbbb"), "{out}");

        assert_eq!(t.adr_at("00000000aaaa").status, AdrStatus::Accepted);
        assert!(t.adr_at("00000000aaaa").ratified.is_some());
        assert_eq!(
            t.adr_at("00000000bbbb").status,
            AdrStatus::Superseded,
            "the transition model.rs calls legal, performed by the accept of \
             the ADR that replaces it"
        );

        // The commit is authoritative as soon as it exists, so it must not
        // exist over a corpus `check` would call broken. Both directions of the
        // chain, and neither is a finding.
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(
            !r.findings
                .iter()
                .any(|f| f.message.contains("superseded") || f.message.contains("supersedes")),
            "{:?}",
            r.findings
        );

        // And both files are in the one commit: two would leave a window in
        // which history says both constraints bind.
        let files = String::from_utf8_lossy(
            &Command::new("git")
                .current_dir(&t.0)
                .args(["show", "--name-only", "--format=%B", "HEAD"])
                .output()
                .unwrap()
                .stdout,
        )
        .to_string();
        assert!(files.contains("ADR-00000000aaaa.md"), "{files}");
        assert!(files.contains("ADR-00000000bbbb.md"), "{files}");
        assert!(
            files.contains("supersedes: ADR-00000000bbbb"),
            "the message records the succession: {files}"
        );
    }

    /// Superseding a proposal is meaningless: nothing was ever binding, and the
    /// caller almost certainly meant a different identifier.
    #[test]
    fn accept_refuses_a_target_that_is_not_accepted_and_writes_nothing() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000bbbb", AdrStatus::Proposed, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);
        t.commit("seed");

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.message.contains("proposed"), "{}", err.message);
        assert_eq!(
            err.hint.as_deref(),
            Some("ank accept ADR-00000000bbbb"),
            "the refusal names the command that would make it acceptable"
        );

        // A refusal is not a half transition, in either file.
        assert_eq!(t.adr_at("00000000aaaa").status, AdrStatus::Proposed);
        assert!(t.adr_at("00000000aaaa").ratified.is_none());
        assert_eq!(t.adr_at("00000000bbbb").status, AdrStatus::Proposed);
    }

    /// Re-pointing the chain silently would rewrite whose succession it was,
    /// and the corpus would keep no record of the one it dropped.
    #[test]
    fn accept_refuses_a_target_another_adr_already_superseded() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["src/**"]));
        let mut first = adr("00000000cccc", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut first {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&first);
        let mut late = adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]);
        if let Entity::Adr(x) = &mut late {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&late);
        t.commit("seed");

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.message.contains("ADR-00000000cccc"), "{}", err.message);

        assert_eq!(t.adr_at("00000000aaaa").status, AdrStatus::Proposed);
        assert_eq!(t.adr_at("00000000bbbb").status, AdrStatus::Accepted);
    }

    /// The corpus `accept` was written for and could not reach: promoted by
    /// editing the file, so it never passed through `proposed` and the only
    /// legal promotion had no door to it.
    #[test]
    fn accept_ratifies_an_adr_accepted_by_hand_and_never_anchored() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        t.commit("seed");
        assert!(t.adr_at("00000000aaaa").ratified.is_none());

        let (code, out) = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(
            out.contains("ratified ADR-00000000aaaa"),
            "the act is the anchor, not a promotion that already happened: {out}"
        );

        let a = t.adr_at("00000000aaaa");
        assert_eq!(a.status, AdrStatus::Accepted);
        assert_eq!(
            a.ratified,
            Some(ratification_anchor(&a.constraint, &a.scope))
        );

        // And the signal that named this state is gone, which is the whole
        // point: it is what `check` reports over a bootstrap corpus.
        let r = t.report();
        assert!(
            !r.findings
                .iter()
                .any(|f| f.message.contains("no ratification commit")),
            "{:?}",
            r.findings
        );
    }

    /// The half doing the work. `accept` over an existing anchor is how a
    /// constraint edited in place would be re-anchored, and that is the one
    /// property ADR-6b3f19e08a24 exists to hold.
    #[test]
    fn accept_refuses_an_adr_that_already_carries_an_anchor() {
        let t = Temp::new();
        t.enable_signing();
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.ratified = Some("deadbeefcafe".into());
        }
        t.write(&a);
        t.commit("seed");
        let before = head(&t);

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert!(err.message.contains("deadbeefcafe"), "{}", err.message);
        assert_eq!(
            err.hint.as_deref(),
            Some("ank new adr --supersedes ADR-00000000aaaa"),
            "changing a ratified decision is a succession, and the hint says so \
             instead of offering the file"
        );

        // The anchor is untouched and no commit was made: a refusal that had
        // rewritten either would be the laundering it exists to prevent.
        assert_eq!(
            t.adr_at("00000000aaaa").ratified.as_deref(),
            Some("deadbeefcafe")
        );
        assert_eq!(t.adr_at("00000000aaaa").version, 1);
        assert_eq!(head(&t), before);
    }

    /// Bootstrap again, one level down: the succession was performed by hand
    /// too. Also the state an `accept` interrupted between its two writes leaves
    /// behind — the same corpus, and the same nothing to do about it.
    #[test]
    fn ratifying_a_succession_already_recorded_writes_only_the_superseder() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000bbbb", AdrStatus::Superseded, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);
        t.commit("seed");

        let (code, out) = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(t.adr_at("00000000aaaa").ratified.is_some());
        assert_eq!(
            t.adr_at("00000000bbbb").version,
            1,
            "the target already said it: rewriting it would bump a version to \
             record nothing"
        );

        // The commit still names the succession — it is what the ratification
        // is about — while carrying only the file that changed.
        let shown = String::from_utf8_lossy(
            &Command::new("git")
                .current_dir(&t.0)
                .args(["show", "--name-only", "--format=%B", "HEAD"])
                .output()
                .unwrap()
                .stdout,
        )
        .to_string();
        assert!(shown.contains("supersedes: ADR-00000000bbbb"), "{shown}");
        assert!(!shown.contains("ADR-00000000bbbb.md"), "{shown}");

        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
    }

    /// The exception is the anchor, not the chain: a target that was never
    /// binding is still not something to supersede, whichever door `accept`
    /// came through.
    #[test]
    fn ratifying_in_place_still_refuses_a_target_that_was_never_accepted() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000bbbb", AdrStatus::Proposed, &["src/**"]));
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.supersedes = Some(EntityId::parse("ADR-00000000bbbb").unwrap());
        }
        t.write(&a);
        t.commit("seed");

        let err = t
            .call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(t.adr_at("00000000aaaa").ratified.is_none());
    }

    /// The heart of it. Whoever edits the constraint also controls the copy of
    /// the anchor sitting beside it, so a comparison against the field would be
    /// the file agreeing with itself. The hash that decides is in the commit,
    /// and replacing that one costs a signature.
    #[test]
    fn the_freeze_compares_against_the_commit_and_not_against_the_field() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        t.call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();

        let repo = t.repo();
        assert_eq!(
            freeze_state(&repo, &t.adr_at("00000000aaaa")),
            Freeze::Intact
        );

        let mut altered = t.adr_at("00000000aaaa");
        let base = altered.version;
        altered.constraint = "A different rule.\n".into();
        altered.ratified = Some(ratification_anchor(&altered.constraint, &altered.scope));
        t.store().write(&Entity::Adr(altered), base).unwrap();

        assert!(
            matches!(
                freeze_state(&repo, &t.adr_at("00000000aaaa")),
                Freeze::Altered { .. }
            ),
            "the field was moved to match, and it changes nothing"
        );
    }

    // -----------------------------------------------------------------------
    // The signature on the ratification commit (TASK-d31af22248d9)
    // -----------------------------------------------------------------------

    /// The shape the file actually uses, which is git's: principal, then the
    /// key, with anything optional allowed to sit between them.
    #[test]
    fn allowed_signers_is_read_as_git_writes_it() {
        let parsed = parse_signers(
            "# a comment\n\
             \n\
             sean@example.com gpg 739A603FB05F9F2F7D3C8D50624FCFCC1482554A\n\
             marie@laptop namespaces=\"git\" ssh-ed25519 AAAAC3NzaC1lZDI1\n\
             nonsense\n",
        );
        assert_eq!(parsed.len(), 2, "{parsed:?}");
        assert_eq!(parsed[0].keytype, "gpg");
        assert_eq!(parsed[0].key, "739A603FB05F9F2F7D3C8D50624FCFCC1482554A");
        assert_eq!(
            parsed[1].keytype, "ssh-ed25519",
            "the option in the middle must not be mistaken for the key type"
        );
        assert_eq!(parsed[1].key, "AAAAC3NzaC1lZDI1");
    }

    fn gpg_signer(key: &str) -> Vec<Signer> {
        vec![Signer {
            principal: "sean@example.com".into(),
            keytype: "gpg".into(),
            key: key.into(),
        }]
    }

    fn facts(status: char, fingerprint: &str) -> git::SignatureFacts {
        git::SignatureFacts {
            status,
            fingerprint: fingerprint.into(),
        }
    }

    /// Every state git's facts can produce, and each one distinguishable from
    /// the others. Pure, so the five live here rather than behind five fixtures
    /// with five keyrings. The sixth, `Unreadable`, is the absence of those
    /// facts and so cannot appear here: it is covered through the binary, where
    /// a git that refuses to answer is something one can actually arrange.
    #[test]
    fn every_signature_state_is_distinguished() {
        let fp = "739A603FB05F9F2F7D3C8D50624FCFCC1482554A";
        let declared = gpg_signer(fp);

        assert_eq!(
            classify_signature(&facts('G', fp), &declared, false),
            Signature::Trusted
        );
        assert_eq!(
            classify_signature(&facts('N', ""), &declared, false),
            Signature::Absent,
            "no signature is the forgery this check exists for"
        );
        assert_eq!(
            classify_signature(&facts('N', ""), &declared, true),
            Signature::Unchecked,
            "the same letter over a commit that does carry a signature is a \
             machine that could not look, not a forgery"
        );
        assert_eq!(
            classify_signature(&facts('E', fp), &declared, false),
            Signature::Unchecked,
            "a missing public key must never read as a valid signature"
        );
        assert_eq!(
            classify_signature(&facts('B', fp), &declared, false),
            Signature::Invalid { status: 'B' }
        );
        assert_eq!(
            classify_signature(
                &facts('G', "0000000000000000000000000000000000000000"),
                &declared,
                false
            ),
            Signature::Undeclared {
                fingerprint: "0000000000000000000000000000000000000000".into()
            },
            "a good signature from a key nobody declared is not a ratification"
        );
    }

    /// The long key id is the tail of the fingerprint, and a file declaring
    /// either is declaring the same key. Case is not meaningful in hex.
    #[test]
    fn a_declared_long_key_id_matches_the_full_fingerprint() {
        let declared = gpg_signer("624fcfcc1482554a");
        assert_eq!(
            classify_signature(
                &facts('G', "739A603FB05F9F2F7D3C8D50624FCFCC1482554A"),
                &declared,
                false
            ),
            Signature::Trusted
        );
    }

    /// Under OpenPGP, `U` says the keyring does not ultimately trust the key —
    /// a fact about the operator's web of trust, not about this repository.
    /// The declaration is what decides, so it is judged like `G`.
    #[test]
    fn an_untrusted_openpgp_key_is_still_judged_by_the_declaration() {
        let fp = "739A603FB05F9F2F7D3C8D50624FCFCC1482554A";
        assert_eq!(
            classify_signature(&facts('U', fp), &gpg_signer(fp), false),
            Signature::Trusted
        );
    }

    /// Under SSH the letters mean something else, and this is measured against
    /// git rather than assumed: with an allowed-signers file, `G` is a matched
    /// principal and `U` is a good signature the file does not cover. Comparing
    /// the `SHA256:` fingerprint against the base64 key ourselves would call
    /// every correct signature undeclared.
    #[test]
    fn under_ssh_git_has_already_decided_the_allowlist() {
        let ssh = "SHA256:KjhREDTh0GcSpp8doZ/yhLAG62FZ7h26dQTVffo3JFE";
        let unrelated = gpg_signer("739A603FB05F9F2F7D3C8D50624FCFCC1482554A");
        assert_eq!(
            classify_signature(&facts('G', ssh), &unrelated, false),
            Signature::Trusted
        );
        assert!(matches!(
            classify_signature(&facts('U', ssh), &unrelated, false),
            Signature::Undeclared { .. }
        ));
    }

    /// Writes an allowed-signers file, since the fixture has none by default.
    fn declare(t: &Temp, line: &str) {
        std::fs::write(t.0.join(".ank").join("allowed_signers"), line).unwrap();
    }

    /// The public key `enable_signing` generated, in allowed-signers layout.
    fn signing_principal(t: &Temp) -> String {
        let pub_key = std::fs::read_to_string(t.0.join("signing-key.pub")).unwrap();
        format!("test@ank.local {}", pub_key.trim())
    }

    /// The hole this task was filed for, end to end and through `check`. An
    /// ordinary unsigned commit whose subject reads `ratify <id>` used to be
    /// accepted as a ratification: `rev-list` found it, the subject matched,
    /// the hashes agreed, and the freeze was reported intact.
    #[test]
    fn an_unsigned_ratification_commit_is_refused_as_a_ratification() {
        let t = Temp::new();
        t.enable_signing();
        declare(&t, &signing_principal(&t));

        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        t.call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();

        // The real ratification verifies, which is what makes the negative
        // below mean something.
        let repo = t.repo();
        assert_eq!(
            signature_state(&repo, &t.adr_at("00000000aaaa")),
            Some(Signature::Trusted)
        );
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);

        // Now forge one: same subject, same anchor line, no signature. `accept`
        // is not involved, and nothing but the signature tells the two apart.
        let forged = adr("00000000bbbb", AdrStatus::Accepted, &["src/**"]);
        let Entity::Adr(mut b) = forged else {
            unreachable!()
        };
        b.ratified = Some(ratification_anchor(&b.constraint, &b.scope));
        let anchor = ratification_anchor(&b.constraint, &b.scope);
        t.write(&Entity::Adr(b));
        for args in [
            vec!["add", "-A"],
            // Unsigned on purpose: the fixture turns signing off at creation,
            // and this commit asserting `Absent` is what depends on it.
            vec![
                "commit",
                "-qm",
                &format!("ratify ADR-00000000bbbb\n\nconstraint+scope: {anchor}"),
            ],
        ] {
            Command::new("git")
                .current_dir(&t.0)
                .args(&args)
                .status()
                .unwrap();
        }

        assert_eq!(
            signature_state(&t.repo(), &t.adr_at("00000000bbbb")),
            Some(Signature::Absent),
            "the anchor agrees; only the signature does not"
        );
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "not signed"),
            "an unsigned ratification must be a fault: {:?}",
            r.findings
        );
        assert!(r.faults() > 0);
    }

    /// Puts an OpenPGP signature header on `HEAD`, without gpg.
    ///
    /// The commit object is read back, `gpgsig` is inserted where git puts it,
    /// the result is stored with `hash-object` and `main` is moved onto it: same
    /// tree, same parent, same subject, one header more. The block itself is
    /// never parsed — nothing in this fixture can reach gpg to try — so what it
    /// says does not matter. That it is there is the whole point.
    fn sign_the_head_object(t: &Temp) {
        let object = t.git_out(&["cat-file", "commit", "HEAD"]);
        let (headers, message) = object
            .split_once("\n\n")
            .expect("a commit object separates headers from its message");
        let signed = format!(
            "{headers}\ngpgsig -----BEGIN PGP SIGNATURE-----\n \n \
             iQIzBAABCgAdFiEEc5pgP7BfnznX0jVUYk/PzBSCVUoFAmAAAAAACgkQYk/PzBSC\n \
             -----END PGP SIGNATURE-----\n\n{message}\n"
        );
        let path = t.0.join("signed-commit-object");
        std::fs::write(&path, &signed).unwrap();
        let sha = t.git_out(&["hash-object", "-t", "commit", "-w", path.to_str().unwrap()]);
        t.git_ok(&["update-ref", "refs/heads/main", &sha]);
        std::fs::remove_file(&path).unwrap();
    }

    /// A signature this machine cannot check is `Unchecked`, never `Absent`
    /// (TASK-f4ed2020c964).
    ///
    /// Through the OpenPGP path, which is where the ambiguity lives: git answers
    /// `N` — the same letter it uses for a commit nobody signed — when it cannot
    /// spawn `gpg.program` at all, and puts the reason on stderr where no
    /// verdict belongs. Read as `Absent`, that told a contributor on a full
    /// clone with no GnuPG installed that this repository's constraints are
    /// ratified by nobody.
    ///
    /// No gpg anywhere in the fixture, so this runs the same on a maintainer's
    /// machine and on a runner that has never held a key: the header is written
    /// onto the object directly, and `gpg.program` is pointed at a path that
    /// cannot exist. What is under test is how ank reads git's answer.
    /// A ratified ADR whose ratification commit carries a signature header or
    /// does not, in a repository where gpg cannot be reached at all.
    ///
    /// Two fixtures rather than one repository asked twice: `ratification_at`
    /// memoises its answer per repository and id for the life of the process,
    /// on the sound assumption that git history does not change under a running
    /// tool. Rewriting the commit after asking would be answered from that
    /// cache, and the test would measure the memo instead of the verdict.
    fn ratified_where_gpg_cannot_run(signed: bool) -> Temp {
        let t = Temp::new();
        // An OpenPGP declaration: `signature_state` answers `None` without one,
        // and under SSH git decides the allowlist itself.
        declare(
            &t,
            "sean@example.com gpg 739A603FB05F9F2F7D3C8D50624FCFCC1482554A",
        );
        t.commit("seed");

        let entity = adr("00000000cccc", AdrStatus::Accepted, &["src/**"]);
        let Entity::Adr(mut a) = entity else {
            unreachable!()
        };
        let anchor = ratification_anchor(&a.constraint, &a.scope);
        a.ratified = Some(anchor.clone());
        t.write(&Entity::Adr(a));
        t.commit(&format!(
            "ratify ADR-00000000cccc\n\nconstraint+scope: {anchor}"
        ));
        if signed {
            sign_the_head_object(&t);
        }

        // The machine without GnuPG, arranged rather than waited for.
        let nowhere = t.0.join("no-such-gpg");
        t.git_ok(&["config", "gpg.program", nowhere.to_str().unwrap()]);
        t
    }

    #[test]
    fn a_signature_that_cannot_be_checked_is_unchecked_and_never_absent() {
        // The control, in the same environment: no header, so `N` means what it
        // says. Without it the assertion below would pass on a fixture where
        // the header changed nothing.
        let unsigned = ratified_where_gpg_cannot_run(false);
        assert_eq!(
            signature_state(&unsigned.repo(), &unsigned.adr_at("00000000cccc")),
            Some(Signature::Absent),
            "a commit nobody signed stays Absent, unreachable gpg or not"
        );

        let signed = ratified_where_gpg_cannot_run(true);
        assert_eq!(
            signature_state(&signed.repo(), &signed.adr_at("00000000cccc")),
            Some(Signature::Unchecked),
            "git answers N because it could not run gpg, not because the commit \
             is unsigned: reporting that as Absent calls a ratified corpus forged"
        );
    }

    /// A good signature from a key the file does not declare is not a
    /// ratification either: the allowlist is what §8 says decides who may
    /// ratify, and a signature nobody vouched for vouches for nobody.
    #[test]
    fn a_signature_from_an_undeclared_key_is_refused() {
        let t = Temp::new();
        t.enable_signing();
        declare(
            &t,
            "someone@else ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB\n",
        );

        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        t.call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();

        assert!(
            matches!(
                signature_state(&t.repo(), &t.adr_at("00000000aaaa")),
                Some(Signature::Undeclared { .. })
            ),
            "signed for real, by a key the corpus never declared"
        );
        let r = t.report();
        assert!(
            has(&r, Level::Fault, "does not declare"),
            "{:?}",
            r.findings
        );
    }

    /// With nothing declared there is no allowlist to judge against, and §8
    /// already reports that once as advisory. Going further would be unsafe as
    /// well as noisy: under `gpg.format = ssh` git reports a perfectly signed
    /// commit as unsigned when no allowed-signers file is configured, so a
    /// corpus without the file would have every ratification called a forgery.
    #[test]
    fn with_no_declared_key_the_signature_is_not_judged_at_all() {
        let t = Temp::new();
        t.enable_signing();
        t.write(&adr("00000000aaaa", AdrStatus::Proposed, &["src/**"]));
        t.commit("seed");
        t.call(&["accept", "ADR-00000000aaaa"], "marie@laptop")
            .unwrap();

        assert_eq!(
            signature_state(&t.repo(), &t.adr_at("00000000aaaa")),
            None,
            "no file, no verdict"
        );
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "advisory"),
            "the advisory signal is what covers this case: {:?}",
            r.findings
        );
    }

    /// A machine without the public key is a correct repository on an
    /// incomplete machine, and it is reported once for the corpus rather than
    /// once per ADR — the rule §4 already applies to entities predating
    /// `author`, for the same reason.
    #[test]
    fn unchecked_signatures_are_counted_once_for_the_corpus() {
        let mut report = Report {
            unchecked_signatures: 3,
            ..Default::default()
        };
        // The rendering path is what the reader sees, so assert on the finding
        // the corpus pass emits rather than on the counter.
        if report.unchecked_signatures > 0 {
            let n = report.unchecked_signatures;
            report.findings.push(Finding::signal(
                "allowed_signers",
                format!("{n} ratification signature(s) could not be checked here"),
            ));
        }
        assert_eq!(report.faults(), 0, "never a fault: CI would go red");
        assert_eq!(
            report
                .findings
                .iter()
                .filter(|f| f.message.contains("could not be checked"))
                .count(),
            1,
            "one line for the machine, not one per ADR"
        );
    }

    /// Dogfooding, and the only test that exercises the OpenPGP branch: the
    /// fixtures above all sign with SSH, where git decides the allowlist, while
    /// this repository signs with GPG, where ank decides it. Without this, the
    /// branch that runs in production is the branch nothing runs.
    ///
    /// Two outcomes are correct and they are not the same: `Trusted` on a
    /// machine holding the public key, `Unchecked` on one that does not — CI,
    /// for instance. What must never appear is a verdict claiming the corpus is
    /// forged.
    #[test]
    fn this_repositorys_own_ratifications_are_signed() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let repo = Repo {
            ank: root.join(".ank"),
            root,
        };
        let store = Store::new(&repo.ank);
        let mut judged = 0;
        for id in store.list_ids().unwrap() {
            let Entity::Adr(a) = store.load(&id).unwrap().entity else {
                continue;
            };
            let Some(state) = signature_state(&repo, &a) else {
                continue;
            };
            judged += 1;
            assert!(
                matches!(state, Signature::Trusted | Signature::Unchecked),
                "{}: {state:?}",
                a.id
            );
        }
        // Nothing judged is a correct answer in exactly one situation, and it
        // is the situation CI runs in: `actions/checkout` clones shallow, so
        // `rev-list --full-history` reaches no ratification commit and every
        // ADR is the unverifiable case the freeze already reports. Asserting
        // `judged > 0` unconditionally asserted a property of the machine.
        //
        // Demanding the reason rather than skipping is what keeps this a test:
        // on any full clone, nothing judged means the wiring is broken.
        if judged == 0 {
            let shallow = Command::new("git")
                .current_dir(&repo.root)
                .args(["rev-parse", "--is-shallow-repository"])
                .output()
                .unwrap();
            assert_eq!(
                String::from_utf8_lossy(&shallow.stdout).trim(),
                "true",
                "no ratification was judged and the clone is complete: \
                 the check is wired to nothing"
            );
        }
    }

    /// A shallow clone, a rewritten history, a corpus moved between
    /// repositories. Saying "altered" here would be a lie, and a finding that
    /// lies is one people learn to skip.
    #[test]
    fn an_unreachable_ratification_commit_is_a_signal_and_never_a_divergence() {
        let t = Temp::new();
        let mut a = adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]);
        if let Entity::Adr(x) = &mut a {
            x.ratified = Some("deadbeefcafe".into());
        }
        t.write(&a);
        t.commit("seed");

        assert_eq!(
            freeze_state(&t.repo(), &t.adr_at("00000000aaaa")),
            Freeze::Unverifiable
        );
        let r = t.report();
        assert_eq!(r.faults(), 0, "{:?}", r.findings);
        assert!(
            has(&r, Level::Signal, "cannot be verified"),
            "{:?}",
            r.findings
        );
    }

    fn head(t: &Temp) -> String {
        String::from_utf8_lossy(
            &Command::new("git")
                .current_dir(&t.0)
                .args(["rev-parse", "HEAD"])
                .output()
                .unwrap()
                .stdout,
        )
        .trim()
        .to_string()
    }

    #[test]
    fn the_ratification_anchor_covers_the_constraint_and_the_scope() {
        let base = ratification_anchor("A rule.\n", &["src/**".into()]);
        assert_eq!(
            base,
            ratification_anchor("A rule.  \n\n", &["src/**".into()]),
            "editing noise does not move it"
        );
        assert_ne!(
            base,
            ratification_anchor("Another rule.\n", &["src/**".into()])
        );
        assert_ne!(
            base,
            ratification_anchor("A rule.\n", &["src/auth/**".into()]),
            "narrowing the scope changes what was ratified"
        );
    }

    // -----------------------------------------------------------------------
    // close, show, review
    // -----------------------------------------------------------------------

    // -----------------------------------------------------------------------
    // amend
    // -----------------------------------------------------------------------

    /// Allowed, because a scope discovered false mid-task is the situation the
    /// verb exists for. Warned about, because the claim record anchors the hash
    /// of the constraints that scope selects and the change moves them.
    #[test]
    fn amending_the_scope_under_a_live_claim_is_allowed_and_says_so() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");

        let (code, out) = t
            .call(&["amend", "0000", "--scope", "docs/**"], "marie@laptop")
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("amended"), "{out}");
        // The warning moved to standard error, which this harness does not
        // capture (TASK-2eefcdd80124): stdout under `--json` is a parser's
        // input, and a takeover notice is not the answer. That it is still said,
        // and that it still names the holder, is asserted through the binary in
        // `tests/cli.rs` — silence would be worse, and that is what checks it.
        assert!(
            !out.contains("warning"),
            "the warning reached stdout: {out}"
        );

        let Entity::Task(after) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        assert_eq!(after.scope, vec!["src/**", "docs/**"]);
        assert_eq!(after.version, 2);

        // A blocked_by change touches no constraint, so it says nothing. The
        // negative half matters as much as the warning: a verb that warned on
        // every amendment would be a verb whose warning nobody reads.
        t.write(&task("000000000002", TaskStatus::Open, &[]));
        let (code, out) = t
            .call(
                &[
                    "amend",
                    "TASK-000000000001",
                    "--blocked-by",
                    "TASK-000000000002",
                ],
                "marie@laptop",
            )
            .unwrap();
        assert_eq!(code, 0, "{out}");
        assert!(out.contains("+blocked_by"), "{out}");
        assert!(!out.contains("warning"), "{out}");
    }

    /// §3 allows exactly one write to a task after completion and it is
    /// `attest`'s. Amending a finished task would produce the corpus fault
    /// `check` reports as a done task modified beyond appending a proof.
    #[test]
    fn amend_refuses_a_finished_task_and_a_ratified_scope() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Done, &[]));
        let err = t
            .call(&["amend", "0000", "--scope", "docs/**"], "marie@laptop")
            .unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.message.contains("settled"), "{}", err.message);

        // An accepted ADR's scope is hashed into the ratification commit, so
        // amending it would diverge from the anchor and suspend its injection
        // into context. Changing a ratified decision is a succession.
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        let err = t
            .call(
                &["amend", "ADR-00000000aaaa", "--scope", "docs/**"],
                "marie@laptop",
            )
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert!(
            err.hint.unwrap().contains("--supersedes"),
            "the way through"
        );

        // A proposed one is free: nothing was ever binding.
        t.write(&adr("00000000bbbb", AdrStatus::Proposed, &["src/**"]));
        let (code, _) = t
            .call(
                &["amend", "ADR-00000000bbbb", "--scope", "docs/**"],
                "marie@laptop",
            )
            .unwrap();
        assert_eq!(code, 0);
        let Entity::Adr(after) = t
            .store()
            .load(&EntityId::parse("ADR-00000000bbbb").unwrap())
            .unwrap()
            .entity
        else {
            panic!()
        };
        assert_eq!(after.scope, vec!["src/**", "docs/**"]);
    }

    #[test]
    fn close_requires_a_reason_and_revokes_the_active_claim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::InProgress, &[]));
        t.claim_as(&id, "codex@host-9", "A verifiable criterion.\n");

        let err = t.call(&["close", "0000"], "marie@laptop").unwrap_err();
        assert_eq!(err.code, 7, "{}", err.message);
        assert!(err.hint.unwrap().contains("--reason"));

        let (code, out) = t
            .call(
                &[
                    "close",
                    "0000",
                    "--reason",
                    "superseded by the new pipeline",
                ],
                "marie@laptop",
            )
            .unwrap();
        assert_eq!(code, 0);
        assert!(out.contains("revoked"), "{out}");

        let Entity::Task(after) = t.store().load(&id).unwrap().entity else {
            panic!()
        };
        assert_eq!(after.status, TaskStatus::Closed);
        let entries = t.log(&id);
        assert!(entries[0]
            .message
            .contains("superseded by the new pipeline"));
        // The holder learns of it at its next log: the ref is gone.
        assert!(claim::read(&t.0, &id).unwrap().is_none());
        // And the file survives, because deleting it would break references.
        assert!(t.store().path_of(&id).exists());
    }

    #[test]
    fn show_prints_the_entity_verbatim() {
        let t = Temp::new();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        let (code, out) = t.call(&["show", "0000"], "marie@laptop").unwrap();
        assert_eq!(code, 0);
        // The entity comes first and comes whole: the derived sections are
        // appended under it, and nothing is allowed to reformat what is above.
        let file = std::fs::read_to_string(t.store().path_of(&id)).unwrap();
        assert!(
            out.starts_with(&file),
            "byte for byte, which is what makes it a reliable read: {out}"
        );
        assert_eq!(
            &out[file.len()..],
            "\nBLOCKED BY (0)\n\nUNBLOCKS (0)\n",
            "both headings print at zero: an absent heading is not an answer"
        );
    }

    /// An ADR has no `blocked_by`, so `show` on one is unchanged.
    #[test]
    fn show_on_an_adr_stays_verbatim_and_adds_nothing() {
        let t = Temp::new();
        let id = EntityId::parse("ADR-00000000aaaa").unwrap();
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        let (code, out) = t.call(&["show", "ADR-0000"], "marie@laptop").unwrap();
        assert_eq!(code, 0);
        assert_eq!(
            out,
            std::fs::read_to_string(t.store().path_of(&id)).unwrap()
        );
    }

    /// The end-to-end half of the painter's invariant. `paint`'s own tests
    /// prove the scan; this proves the wiring — that `show` calls it, that
    /// calling it moved nothing, and that `--json` was left out of it.
    #[test]
    fn show_paints_the_entity_and_moves_nothing() {
        let t = Temp::new();
        t.write(&task("000000000001", TaskStatus::Open, &[]));
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        let repo = t.repo();

        let render = |args: &[&str], style: crate::style::Style| {
            let argv: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            let mut inv = crate::cli::parse(&argv).unwrap();
            inv.style = style;
            let mut out = Vec::new();
            show(&inv, &repo, &mut out).unwrap();
            String::from_utf8(out).unwrap()
        };

        // Qualified: both entities live in this repo, and a bare `0000` is
        // ambiguous across the two kinds.
        for id in ["TASK-0000", "ADR-0000"] {
            let plain = render(&["show", id], crate::style::PLAIN);
            let painted = render(&["show", id], crate::style::COLOR);
            // Asserted before the comparison: `undo_sgr` strips both sides.
            assert!(
                !plain.contains('\x1b'),
                "the plain render carries an escape"
            );
            assert_ne!(painted, plain, "show did not paint {id}");
            assert_eq!(
                crate::style::undo_sgr(&painted),
                plain,
                "colour moved the content of {id}"
            );
        }

        // The style is forced to COLOR here rather than left to dispatch, which
        // would have set it to PLAIN: what is under test is that `show` itself
        // keeps the machine surface out of the painting, not that something
        // upstream happens to.
        let json = render(&["show", "TASK-0000", "--json"], crate::style::COLOR);
        assert!(!json.contains('\x1b'), "--json was painted: {json}");
    }

    #[test]
    fn review_lists_live_constraints_and_sets_dead_ones_apart() {
        let t = Temp::new();
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        t.write(&adr("00000000bbbb", AdrStatus::Accepted, &["nowhere/**"]));

        let (_, out) = t.call(&["review"], "marie@laptop").unwrap();
        assert!(out.contains("LIVE CONSTRAINTS (1)"), "{out}");
        assert!(out.contains("ADR-00000000aaaa"), "{out}");
        assert!(out.contains("DEAD SCOPES (1)"), "{out}");
        assert!(out.contains("ADR-00000000bbbb"), "{out}");
        // The queue is answered even when it is empty, because an empty queue
        // and an unprinted queue used to be the same bytes.
        assert!(out.contains("nothing proposed for ratification"), "{out}");
        assert!(!out.contains("PROPOSED"), "{out}");
    }

    /// The ratification queue `review` is described by and did not print.
    ///
    /// Three properties, and the third is the one the defect turned on: a
    /// proposal must not be counted as a live constraint, it must come before
    /// them, and a proposal whose scope has died stays in the queue — it is
    /// waiting for a human either way, and the reader most in need of the
    /// answer is the one whose entry would have been dropped.
    #[test]
    fn review_opens_with_what_is_waiting_for_ratification() {
        let t = Temp::new();
        t.write(&adr("00000000aaaa", AdrStatus::Accepted, &["src/**"]));
        t.write(&adr("00000000cccc", AdrStatus::Proposed, &["src/**"]));
        t.write(&adr("00000000dddd", AdrStatus::Proposed, &["nowhere/**"]));
        // History, and history is not a review of the present.
        t.write(&adr("00000000eeee", AdrStatus::Superseded, &["src/**"]));

        let (_, out) = t.call(&["review"], "marie@laptop").unwrap();
        assert!(out.contains("PROPOSED (2)"), "{out}");
        assert!(out.contains("LIVE CONSTRAINTS (1)"), "{out}");
        assert!(
            out.find("PROPOSED (2)") < out.find("LIVE CONSTRAINTS (1)"),
            "the queue is what a maintainer runs this for, and it comes first: {out}"
        );
        assert!(out.contains("ADR-00000000dddd"), "{out}");
        assert!(
            !out.contains("ADR-00000000eeee"),
            "a superseded decision is waiting for nobody: {out}"
        );

        let (_, json) = t.call(&["review", "--json"], "marie@laptop").unwrap();
        assert!(
            json.contains("\"proposed\":[{\"id\":\"ADR-00000000cccc\""),
            "{json}"
        );
        assert!(json.contains("\"live\":["), "{json}");
        assert!(json.contains("\"dead\":1"), "{json}");
    }
}
