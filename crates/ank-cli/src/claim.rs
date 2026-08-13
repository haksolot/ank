//! Claims on git refs: pickup, TTL, re-acquisition, completion refs (§7).
//!
//! A claim never touches a task file. It lives in `refs/ank/claims/<id>`, one
//! ref per task (ADR-4e7c25b1f639): writing it into the file would produce a
//! git diff on every pickup, which is exactly the noise separating the two
//! planes exists to avoid, and a single global ref would put two agents taking
//! two different tasks in contention over one address.
//!
//! **The ref has two states at one address**, and it is the record that says
//! which — never the address (ADR-bcf222a31525). `claim` writes a `claim`
//! record; `done` does not delete the ref, it replaces the record with
//! `completed`, with no TTL. That closes a real window: `status: done` lives on
//! the branch that produced it, so between the end of the work and the merge
//! the task would look free to every other agent. Two namespaces would let a
//! stale claim and a completion coexist for the same task — the very ambiguity
//! the ref exists to settle — and would split git's compare-and-swap across two
//! addresses where there is only one conflict.
//!
//! The compare-and-swap is git's own: `update-ref <ref> <new> <old>` fails if
//! `<old>` no longer matches. Nothing here builds a lock on top of it.
//!
//! `cli.rs::dispatch` routes `claim` here, and this was the first verb it
//! reached: until TASK-45d18f45de2c every verb but `init` answered
//! `not_implemented` while six module headers asserted the opposite.

use crate::cli::{CliError, Invocation, Result};
use crate::config::{self, Config};
use crate::git;
use crate::human::Freeze;
use crate::identity::ENV_AGENT;
use crate::repo::Repo;
use crate::store::Store;
use ank_core::{
    freeze, freeze_hash_short, Adr, AdrStatus, CriteriaBy, Entity, EntityId, Proof, ScopeSet, Task,
    TaskStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// One ref per task, and the address is the same whichever state the record
/// carries.
pub const CLAIMS_PREFIX: &str = "refs/ank/claims/";

/// Where an attestation lives when it has no tree to travel in
/// (ADR-493471d64ba0).
///
/// A separate namespace and not a third state on the claim ref: the two answer
/// different questions and have different lifetimes. A task can carry a
/// completion record and a detached proof at once, and collapsing them would
/// make one erase the other.
pub const PROOF_PREFIX: &str = "refs/ank/proof/";

/// Default TTL (§3). Short on purpose: it is renewed implicitly by `log`, so
/// working is enough to keep the lock.
pub const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);

/// Clock-drift tolerance on expiry (§7). At the scale of a 30-minute TTL, NTP
/// is more than enough, and two minutes cost far less than a claim wrongly
/// stolen from a machine whose clock runs fast.
pub const DRIFT_TOLERANCE: Duration = Duration::from_secs(2 * 60);

pub fn claim_ref(id: &EntityId) -> String {
    format!("{CLAIMS_PREFIX}{id}")
}

pub fn proof_ref(id: &EntityId) -> String {
    format!("{PROOF_PREFIX}{id}")
}

// ---------------------------------------------------------------------------
// UTC timestamps
// ---------------------------------------------------------------------------
//
// No date crate: the need is one format and one parse, and a dependency whose
// only job is to print twenty characters would not pay for itself.

/// `YYYY-MM-DDThh:mm:ssZ` for an instant given in seconds since the epoch.
pub fn format_utc(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs = epoch_secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

pub fn now_utc() -> String {
    format_utc(now_secs())
}

pub fn now_secs() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        // A clock set before 1970 is not a reason to refuse to work.
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// Reads `YYYY-MM-DDThh:mm[:ss]Z` back to seconds since the epoch. Strict: the
/// records are written by this module, so a timestamp that does not read back
/// is a corrupt record, not a format to guess at.
pub fn parse_utc(text: &str) -> Option<i64> {
    let t = text.trim();
    let (date, rest) = t.split_once('T')?;
    let time = rest.strip_suffix('Z')?;
    let mut dp = date.split('-');
    let y: i64 = dp.next()?.parse().ok()?;
    let m: u32 = dp.next()?.parse().ok()?;
    let d: u32 = dp.next()?.parse().ok()?;
    if dp.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let mut tp = time.split(':');
    let hh: i64 = tp.next()?.parse().ok()?;
    let mm: i64 = tp.next()?.parse().ok()?;
    let ss: i64 = match tp.next() {
        Some(s) => s.parse().ok()?,
        None => 0,
    };
    if tp.next().is_some()
        || !(0..24).contains(&hh)
        || !(0..60).contains(&mm)
        || !(0..61).contains(&ss)
    {
        return None;
    }
    Some(days_from_civil(y, m, d) * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Days from 1970-01-01 for a proleptic Gregorian date, and its inverse. The
/// pair is exact over the whole range we can represent, which a naive
/// "365.25 days" approximation is not.
fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 } as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ---------------------------------------------------------------------------
// The record, and its two states
// ---------------------------------------------------------------------------

pub const STATE_CLAIM: &str = "claim";
pub const STATE_COMPLETED: &str = "completed";
pub const STATE_PROOF: &str = "proof";

/// A claim in force: who holds the task and until when, plus the two hashes
/// that anchor what was frozen at pickup (§7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRecord {
    pub task: String,
    pub holder: String,
    pub claimed: String,
    pub expires: String,
    /// The granted lease, in seconds. **Recorded because it cannot be
    /// derived**: a renewal moves `expires` and leaves `claimed` where it is —
    /// `check` reads `claimed` to report blockers the holder created after
    /// taking the task — so `expires - claimed` stops being the lease the
    /// moment the first renewal lands (§3, §7).
    ///
    /// Absent on a record written before it existed, and then read as
    /// [`DEFAULT_TTL`]: the same promise the entity format makes about fields
    /// introduced later, and the reason this is `default` rather than required.
    #[serde(default)]
    pub ttl: u64,
    /// Hash of the frozen `done_criteria`. `done` compares the current field
    /// against it and fails with code 6 if it diverged — immutability is
    /// verifiable, never defended (ADR-6b3f19e08a24).
    pub criteria: String,
    /// Hash of the constraints applicable to the task's scope at pickup. A
    /// constraint accepted mid-work changes it, and `done` warns.
    pub constraints: String,
}

/// A task finished on some branch. No TTL: it stays until durable state has
/// caught up on the default branch, which is what `prune` decides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletedRecord {
    pub task: String,
    pub commit: String,
    /// Absent on a detached HEAD: working detached is legitimate, and the
    /// record then simply carries no branch rather than a made-up one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub identity: String,
    pub completed: String,
}

/// One attestation, and who stood behind it when.
///
/// The proof itself is [`ank_core::Proof`] verbatim rather than a parallel
/// shape: a reader unions these with the task file's list, and two structures
/// for one thing is how the two sources start disagreeing about what a proof
/// is. What is added is the pair a file cannot carry — a proof in a file is
/// dated and attributed by the commit that put it there, and a detached one is
/// authored by an actor with no branch (ADR-493471d64ba0).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttestedProof {
    pub identity: String,
    pub attested: String,
    pub proof: Proof,
}

/// Every attestation recorded against one task, outside its file.
///
/// A list and not a single entry, so that a second attestation appends where
/// the file's `proof` list would have appended. One ref holding one record that
/// carries many is what keeps the append-only rule true on both sides; a record
/// per attestation would need a ref per attestation, and the address is the
/// task.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProofRecord {
    pub task: String,
    pub proofs: Vec<AttestedProof>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    Claim(ClaimRecord),
    Completed(CompletedRecord),
    Proof(ProofRecord),
}

impl Record {
    pub fn task(&self) -> &str {
        match self {
            Record::Claim(c) => &c.task,
            Record::Completed(c) => &c.task,
            Record::Proof(p) => &p.task,
        }
    }
}

/// A record read off a ref, together with the object it came from. The object
/// name is the witness for the next compare-and-swap: acting on a record read
/// a moment ago is only safe if the ref has not moved since.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    pub object: String,
    pub record: Record,
}

/// `name` is the ref, not the task: the two namespaces are repaired by
/// different commands, and a hint naming the claim ref for a damaged proof
/// record would send the reader to delete the wrong thing.
fn corrupt(name: &str, detail: impl std::fmt::Display) -> CliError {
    // Code 9: a coordination ref nobody can read is an environment to repair,
    // not a failure of the agent's work (§4). Never a silent fallback to the
    // other state — that is what would let a completion read as a free task.
    CliError::new(9, format!("unreadable record on {name}: {detail}"))
        .with_hint(format!("git update-ref -d {name}"))
}

/// Serialises a record. `state` is written first and every value goes through
/// the YAML emitter, which quotes what would otherwise read back as another
/// type — a short hash that happens to be all digits is the case that matters,
/// and it has its own test.
pub fn serialize_record(record: &Record) -> String {
    use serde_yaml::{Mapping, Value};
    let (state, body) = match record {
        Record::Claim(c) => (STATE_CLAIM, serde_yaml::to_value(c)),
        Record::Completed(c) => (STATE_COMPLETED, serde_yaml::to_value(c)),
        Record::Proof(p) => (STATE_PROOF, serde_yaml::to_value(p)),
    };
    let mut map = Mapping::new();
    map.insert(Value::from("state"), Value::from(state));
    if let Ok(Value::Mapping(fields)) = body {
        for (k, v) in fields {
            map.insert(k, v);
        }
    }
    serde_yaml::to_string(&Value::Mapping(map)).unwrap_or_default()
}

/// Reads a record. Two stages on purpose: the discriminator is read first, so
/// that an unknown state is named as such instead of surfacing as a serde
/// error about a missing field of whichever variant we tried first.
pub fn parse_record(text: &str, name: &str) -> Result<Record> {
    use serde_yaml::Value;
    let value: Value = serde_yaml::from_str(text).map_err(|e| corrupt(name, e))?;
    let Value::Mapping(mut map) = value else {
        return Err(corrupt(name, "not a YAML mapping"));
    };
    let state = map
        .remove(Value::from("state"))
        .ok_or_else(|| corrupt(name, "no state field"))?;
    let state = state
        .as_str()
        .ok_or_else(|| corrupt(name, "state is not a string"))?
        .to_string();
    let rest = Value::Mapping(map);
    match state.as_str() {
        STATE_CLAIM => Ok(Record::Claim(
            serde_yaml::from_value(rest).map_err(|e| corrupt(name, e))?,
        )),
        STATE_COMPLETED => Ok(Record::Completed(
            serde_yaml::from_value(rest).map_err(|e| corrupt(name, e))?,
        )),
        STATE_PROOF => Ok(Record::Proof(
            serde_yaml::from_value(rest).map_err(|e| corrupt(name, e))?,
        )),
        other => Err(corrupt(name, format!("unknown state '{other}'"))),
    }
}

// ---------------------------------------------------------------------------
// Ref operations. The compare-and-swap is git's.
// ---------------------------------------------------------------------------

/// Every task this identity holds a live claim on, `except` aside, ordered by
/// id.
///
/// One claim at a time is the convention, and nothing enforces it. It is worth
/// saying because the default identity is `<user>@<hostname>` (§8): two
/// terminals on one machine are the same agent as far as the refs can tell, so
/// they share and renew each other's claims in silence. Binding identity to the
/// session instead — a PID, a TTY — would break resuming a claim after a
/// restart, and identity is declared, never proof. What is fixable is the
/// silence.
///
/// `now` is a parameter for the same reason `is_expired` takes one: the drift
/// tolerance is two minutes, so a test that waited on the clock for a lapse
/// would wait two minutes.
///
/// A damaged or unreadable record is skipped rather than raised: this feeds a
/// warning, and a reader of other people's refs does not get to fail the write
/// it accompanies. `acquire` is the one that reads the ref it is about to
/// touch, and it is right to call the same damage a hard error.
/// The one sentence that says how to stop sharing an identity.
///
/// **Written once and said by both places that raise the case.** `claim` says it
/// at acquisition and `status` says it whenever the state persists
/// (TASK-38b384543551); two copies of a way out are two chances to name a
/// different variable, and the variable is the whole content of the advice.
pub fn way_out() -> String {
    format!("a second session on this machine sets its own {ENV_AGENT}")
}

pub fn live_claims_of(
    cwd: &Path,
    identity: &str,
    except: &EntityId,
    now: i64,
) -> Result<Vec<(EntityId, ClaimRecord)>> {
    let mut held = Vec::new();
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        if &id == except {
            continue;
        }
        let out = git::output(cwd, &["cat-file", "-p", r.object.as_str()])?;
        if !out.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let Ok(Record::Claim(c)) = parse_record(&text, &r.name) else {
            continue;
        };
        if c.holder != identity || is_expired(&c, now, &id).unwrap_or(true) {
            continue;
        }
        held.push((id, c));
    }
    held.sort_by(|a, b| a.0.to_string().cmp(&b.0.to_string()));
    Ok(held)
}

/// One live claim at a time per identity, refused rather than warned about
/// (§4).
///
/// §4 said "assumes and **enforces** one active claim at a time per agent" for
/// as long as this code only warned. The gap is not academic: the default
/// identity is `<user>@<hostname>` (§8), so two sessions in one working tree
/// are one agent as far as the refs can tell, and their claims do not
/// collide — they accumulate. `on_task` then returns the first live record and
/// `ank_refs` sorts by refname, so HEAD becomes the lowest of the two, picked
/// in silence by `log`, `release` and `done` (TASK-97d8747416ea). This closes
/// the door that state comes through, and TASK-a548c95261a5 records why the
/// previous answer — name it, never refuse (TASK-d79dc424c63d) — was measured
/// to be not enough: the warning is printed once, at acquisition, and a
/// convention that announces itself only when it is taken fades exactly as a
/// session lengthens.
///
/// **Code 7, not 4.** 4 means "take something else" (§4), and the task asked
/// for is available — it is the caller that is not. What is missing is a
/// prerequisite, and it is exact: finish or hand back what is already held.
///
/// **The message names the identity, never the caller.** Under a shared
/// identity the session being refused may have claimed nothing at all; it is
/// being answered about somebody else's claim, and "you already hold" would
/// simply be false. The hint carries both ways through, the second in a
/// parenthetical, exactly as the "another ready task" hints do — and the
/// sentence comes from [`way_out`] rather than a second copy of it.
///
/// **The task being claimed is excluded.** Re-claiming what one already holds
/// is a different question, answered by the transition check and by `acquire`,
/// and this refusal has no business intercepting it.
///
/// Refusing on state and never on identity (ADR-c656cbcc33a9): what is read is
/// the coordination plane — which refs exist and who holds them — and the
/// answer is the same for every caller. A lapsed claim is not a live one, so
/// pickup after expiry (§3) passes through untouched.
fn already_holding(cwd: &Path, identity: &str, target: &EntityId, now: i64) -> Result<()> {
    let held = live_claims_of(cwd, identity, target, now)?;
    // The lowest id, which is the one HEAD resolves to and therefore the one
    // `release` in the hint would hand back. Naming all of them would be a
    // longer message about a state this refusal exists to prevent.
    let Some((id, record)) = held.first() else {
        return Ok(());
    };
    Err(CliError::new(
        7,
        format!(
            "{identity} holds a live claim on {id} ({})",
            remaining_text(record, now)
        ),
    )
    .with_hint(format!("ank release --reason \"<why>\"   ({})", way_out())))
}

/// The record a task's ref carries, if it carries one. An absent ref is
/// `None`, never an error: that is the nominal state of a free task.
pub fn read(cwd: &Path, id: &EntityId) -> Result<Option<Held>> {
    read_at(cwd, &claim_ref(id))
}

/// The same read, addressed by ref rather than by task, so that the proof
/// namespace goes through this code and not beside it (ADR-493471d64ba0).
pub fn read_at(cwd: &Path, name: &str) -> Result<Option<Held>> {
    let args = ["rev-parse", "--verify", "--quiet", name];
    let out = git::output(cwd, &args)?;
    if !out.status.success() {
        return Ok(None);
    }
    let object = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if object.is_empty() {
        return Ok(None);
    }
    let args = ["cat-file", "-p", object.as_str()];
    let out = git::output(cwd, &args)?;
    if !out.status.success() {
        return Err(git::failed(&args, &out));
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    Ok(Some(Held {
        object,
        record: parse_record(&text, name)?,
    }))
}

/// Writes the record as a blob and returns its object name. A ref may point at
/// any object type outside `refs/heads/*`, and a blob is what a record is: no
/// tree, no commit, nothing to walk.
///
/// The one place that spawns git without going through [`git::output`], because
/// the record is fed on stdin and that runner does not pipe one. `hash-object`
/// is in the plumbing ADR-b8884edcebe3 allows; what is lost here is the debug
/// assertion that guards the list, not the rule it enforces.
fn write_blob(cwd: &Path, record: &Record) -> Result<String> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    let text = serialize_record(record);
    let mut child = Command::new("git")
        .current_dir(cwd)
        .args(["hash-object", "-w", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| CliError::new(9, format!("git hash-object: {e}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| CliError::new(9, "git hash-object: no stdin"))?
        .write_all(text.as_bytes())
        .map_err(|e| CliError::new(9, format!("git hash-object: {e}")))?;
    let out = child
        .wait_with_output()
        .map_err(|e| CliError::new(9, format!("git hash-object: {e}")))?;
    if !out.status.success() {
        return Err(git::failed(&["hash-object", "-w", "--stdin"], &out));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Outcome of a compare-and-swap. Losing is an ordinary answer here — it is
/// what "somebody else got there first" looks like — so it is a value and not
/// an error; only the caller knows which message the loss deserves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cas {
    Won,
    Lost,
}

/// `update-ref <ref> <new> <old>`, with `<old>` empty meaning the ref must not
/// exist. A non-zero exit is the CAS saying no; we distinguish it from a
/// broken git by re-reading the ref, which the caller needs anyway to name the
/// winner.
fn update(cwd: &Path, name: &str, new: &str, old: Option<&str>) -> Result<Cas> {
    let args = ["update-ref", name, new, old.unwrap_or("")];
    let out = git::output(cwd, &args)?;
    if out.status.success() {
        return Ok(Cas::Won);
    }
    match current_object(cwd, name)? {
        // The ref is where we left it: nothing moved, so the refusal did not
        // come from contention. Reporting a lost race here would send an agent
        // to take another task when the real problem is the environment.
        Some(o) if Some(o.as_str()) == old => Err(git::failed(&args, &out)),
        None if old.is_none() => Err(git::failed(&args, &out)),
        _ => Ok(Cas::Lost),
    }
}

fn current_object(cwd: &Path, name: &str) -> Result<Option<String>> {
    let out = git::output(cwd, &["rev-parse", "--verify", "--quiet", name])?;
    if !out.status.success() {
        return Ok(None);
    }
    let o = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if o.is_empty() { None } else { Some(o) })
}

/// How far a write of the coordination plane got (§7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sync {
    /// No remote: level 0, the default mode and the only one that shipped
    /// before level 1. Silent by construction — a warning here would fire on
    /// every claim of every solo repository.
    Local,
    /// The remote took it, so the claim holds repository-wide and not merely
    /// in this clone.
    Pushed,
    /// A remote exists and could not be reached. The claim stands locally and
    /// the caller says so: degrade, do not fail (§2), and the risk of a
    /// concurrent claim is displayed rather than hidden.
    Unsynchronised(String),
}

impl Sync {
    /// The sentence a verb prints when the plane did not reach the remote, or
    /// `None` when there is nothing to report.
    ///
    /// Written once because three verbs say it and a warning restated three
    /// times is three chances to describe a different state.
    pub fn warning(&self) -> Option<String> {
        match self {
            Sync::Local | Sync::Pushed => None,
            Sync::Unsynchronised(_) => Some(
                "claim not pushed: it holds in this clone only, and another clone \
                 can take the same task"
                    .to_string(),
            ),
        }
    }

    /// The same sentence for an attestation, and it is a different sentence
    /// because it is a different risk. A claim not pushed can be taken twice;
    /// a proof not pushed is simply invisible to everyone else, which is the
    /// whole thing a detached proof exists to avoid (ADR-493471d64ba0).
    pub fn proof_warning(&self) -> Option<String> {
        match self {
            Sync::Local | Sync::Pushed => None,
            Sync::Unsynchronised(_) => Some(
                "proof not pushed: it is recorded in this clone only, and no other \
                 clone can read it"
                    .to_string(),
            ),
        }
    }
}

/// The outcome of a write: whether it took, and how far it reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    pub cas: Cas,
    pub sync: Sync,
}

/// Puts `record` on the ref. `witness` is the object the caller read: `None`
/// requires the ref to be absent, `Some` requires it to be exactly there.
///
/// **The one place the two planes meet** (§7). Every write of a claim goes
/// through here — acquisition, the renewal `log` performs, the retake of a
/// lapsed claim, the completion record `done` writes — so pushing here covers
/// all four without any of them remembering to. The lease handed to the remote
/// is the same `witness` the local swap used, which is what makes the two
/// checks one rule rather than two that agree today.
///
/// **Local first, then the remote.** The local swap is what this clone's own
/// verbs read next; a push landing over a local write that lost would leave
/// this clone believing something the refs deny.
///
/// A remote that refuses the swap means another clone holds the task, and the
/// local ref is then corrected to the winner's record rather than rolled back
/// to nothing: the caller's next question is *who holds it*, and the answer has
/// to be readable where it looks.
pub fn put(cwd: &Path, id: &EntityId, record: &Record, witness: Option<&str>) -> Result<Written> {
    put_at(cwd, &claim_ref(id), record, witness)
}

/// The same write, addressed by ref. Every property of [`put`] holds here
/// because this is where they are implemented; `put` only knows the address.
pub fn put_at(cwd: &Path, name: &str, record: &Record, witness: Option<&str>) -> Result<Written> {
    let blob = write_blob(cwd, record)?;
    if update(cwd, name, &blob, witness)? == Cas::Lost {
        return Ok(Written {
            cas: Cas::Lost,
            sync: Sync::Local,
        });
    }
    push(cwd, name, Some(&blob), witness)
}

/// The remote half of a write, shared by [`put_at`] and [`delete_at`].
fn push(cwd: &Path, name: &str, new: Option<&str>, witness: Option<&str>) -> Result<Written> {
    if git::remote(cwd)?.is_none() {
        return Ok(Written {
            cas: Cas::Won,
            sync: Sync::Local,
        });
    }
    match git::push_ref(cwd, name, new, witness)? {
        git::Pushed::Ok => Ok(Written {
            cas: Cas::Won,
            sync: Sync::Pushed,
        }),
        git::Pushed::Refused { .. } => {
            // The winner's record, brought here so that whoever asks next reads
            // the truth rather than this clone's rejected guess. If even that
            // fails, the local ref keeps what we wrote and the caller still
            // learns it lost — a wrong holder named is worse than none.
            let _ = git::fetch_ref(cwd, name);
            Ok(Written {
                cas: Cas::Lost,
                sync: Sync::Pushed,
            })
        }
        git::Pushed::Unreachable { reason } => Ok(Written {
            cas: Cas::Won,
            sync: Sync::Unsynchronised(reason),
        }),
    }
}

/// The remote's view of one claim, brought into this clone before a decision
/// rests on it (§7).
///
/// `claim` runs this first so that a task held in another clone is refused with
/// its holder named, rather than taken locally and then unwound by a rejected
/// push. The push remains what arbitrates; this only makes the common case
/// answer politely.
///
/// Silent on every failure. An unreachable remote means the local view is the
/// only one available, and saying so belongs to the write that follows, which
/// is where the risk actually lands.
pub fn sync_from_remote(cwd: &Path, id: &EntityId) -> Result<()> {
    sync_ref_from_remote(cwd, &claim_ref(id))
}

/// The same fetch, addressed by ref.
///
/// A detached proof needs it for a reason a claim never has: the record was
/// written by a pipeline, which is in no clone at all, so a reader that never
/// fetched would report a task as unanchored while its attestation sits on the
/// remote (ADR-493471d64ba0).
pub fn sync_ref_from_remote(cwd: &Path, name: &str) -> Result<()> {
    if git::remote(cwd)?.is_none() {
        return Ok(());
    }
    let Ok(Some(theirs)) = git::ls_remote(cwd, name) else {
        return Ok(());
    };
    if current_object(cwd, name)?.as_deref() == Some(theirs.as_str()) {
        return Ok(());
    }
    let _ = git::fetch_ref(cwd, name);
    Ok(())
}

/// Deletes the ref. This is what `release` and `close` do — and only they:
/// `done` replaces the record, it does not delete (ADR-bcf222a31525). Returns
/// whether there was anything to delete.
///
/// Unconditional, with no witness, because `close` is defined that way: it
/// revokes the active claim whoever holds it, and the holder finds out at its
/// next `log` (§3). `release` therefore has to establish that it holds the
/// claim before calling this — [`read`] answers that — and the check belongs to
/// the verb, not here, where it would make `close` impossible to express.
///
/// **A deletion and not a completion record, for `close` as much as for
/// `release`** (TASK-78326e2e3e89). The asymmetry with `done` is decided rather
/// than inherited: a completion record refuses every other `claim` with code 4,
/// and `done` earns that with a frozen criterion, its verifiers and a proof
/// where the other two are gated by a reason. The reasoning is at `close` in
/// `human.rs`, where the act is.
/// **The deletion travels too** (§7). Level 1 makes a claim visible to every
/// clone, so a release that stayed local would leave the task looking held
/// everywhere else until its TTL ran out — a state this change creates and
/// therefore owes an answer to. The remote failing to take it is not worth
/// failing the release over: the handback is durable state and already written,
/// and the stale ref expires on its own.
pub fn delete(cwd: &Path, id: &EntityId) -> Result<bool> {
    delete_at(cwd, &claim_ref(id))
}

/// The same deletion, addressed by ref. `check` prunes a proof ref through it
/// once the file on the default branch carries the same attestation.
pub fn delete_at(cwd: &Path, name: &str) -> Result<bool> {
    let Some(witness) = current_object(cwd, name)? else {
        return Ok(false);
    };
    let args = ["update-ref", "-d", name];
    let out = git::output(cwd, &args)?;
    if !out.status.success() {
        return Err(git::failed(&args, &out));
    }
    let _ = push(cwd, name, None, Some(&witness));
    Ok(true)
}

// ---------------------------------------------------------------------------
// Detached proofs (ADR-493471d64ba0)
// ---------------------------------------------------------------------------

/// Every attestation `refs/ank/proof/<id>` carries, or an empty list.
///
/// **An absent ref is an empty list and never an error**, exactly as an absent
/// claim ref is a free task: most tasks carry no attestation, and a reader that
/// failed on the ordinary case would be unusable.
///
/// **A damaged record is an empty list too, and that is the harder call.** The
/// alternative is to fail every `show` and every `check` of a corpus one ref of
/// which is corrupt — and the union is additive, so losing it understates what
/// is anchored rather than inventing what is not. `check` is where a damaged
/// coordination ref is reported, and it reports this one through the same walk
/// as the rest.
pub fn detached_proofs(cwd: &Path, id: &EntityId) -> Vec<AttestedProof> {
    match read_at(cwd, &proof_ref(id)) {
        Ok(Some(Held {
            record: Record::Proof(p),
            ..
        })) => p.proofs,
        _ => Vec::new(),
    }
}

/// Appends one attestation to the task's proof ref.
///
/// Read, append, compare-and-swap on the object just read — the same three
/// steps every other write of this plane takes, and for the same reason: two
/// pipelines attesting the same task must not silently overwrite each other.
/// A lost swap is returned as [`Cas::Lost`] and the caller names the retry;
/// ADR-493471d64ba0 argues it is not a case anybody meets, and "not met" is not
/// "cannot happen".
pub fn attach_proof(
    cwd: &Path,
    id: &EntityId,
    proof: &Proof,
    identity: &str,
) -> Result<(Written, usize)> {
    let name = proof_ref(id);
    // The remote first: a pipeline wrote there, in no clone, so a local-only
    // read would append to a record that is already out of date and lose the
    // run before it.
    let _ = sync_ref_from_remote(cwd, &name);
    let held = read_at(cwd, &name)?;
    let (witness, mut proofs) = match held {
        Some(Held {
            object,
            record: Record::Proof(p),
        }) => (Some(object), p.proofs),
        // A ref in this namespace carrying another state is not something to
        // overwrite silently: it is a corrupt plane, and `corrupt` names the
        // command that clears it.
        Some(Held { .. }) => return Err(corrupt(&name, "not a proof record")),
        None => (None, Vec::new()),
    };
    proofs.push(AttestedProof {
        identity: identity.to_string(),
        attested: now_utc(),
        proof: proof.clone(),
    });
    let count = proofs.len();
    let record = Record::Proof(ProofRecord {
        task: id.to_string(),
        proofs,
    });
    let written = put_at(cwd, &name, &record, witness.as_deref())?;
    Ok((written, count))
}

// ---------------------------------------------------------------------------
// Expiry
// ---------------------------------------------------------------------------

/// Whether a claim has lapsed, judged on the timestamp the record carries plus
/// the drift tolerance (§7). A record whose timestamp does not read back is
/// corrupt, and saying "expired" about it would be a silent fallback.
pub fn is_expired(claim: &ClaimRecord, now: i64, id: &EntityId) -> Result<bool> {
    let expires = parse_utc(&claim.expires).ok_or_else(|| {
        corrupt(
            &claim_ref(id),
            format!("unreadable expiry '{}'", claim.expires),
        )
    })?;
    Ok(now > expires + DRIFT_TOLERANCE.as_secs() as i64)
}

/// Where an identity stands on the coordination plane: the task whose ref names
/// it, and whether the claim is still in force.
#[derive(Debug, Clone)]
pub struct Standing {
    pub id: EntityId,
    /// The object the record was read from, for the compare-and-swap.
    pub object: String,
    pub record: ClaimRecord,
    /// The TTL ran out and nobody took the task over. §3 calls this normal, not
    /// a fault: a build longer than the lease expires the claim, and the holder
    /// coming back re-acquires and carries on.
    pub lapsed: bool,
}

/// The task this identity is on — a live claim, or a lapsed one nobody has
/// taken over.
///
/// **One lookup for every verb that acts on "my task"** (§3). `done` and `log`
/// each scanned the refs themselves and each dropped a lapsed claim on the
/// floor, so the return-after-expiry §3 describes as nominal answered
/// `no task in progress for this agent` in both — measured on a claim that
/// lapsed during a CI wait, which is the case the paragraph was written for
/// (TASK-5bd23835d5a0). Two copies of a rule are two chances to get it wrong,
/// and this one got it wrong twice.
///
/// A live claim wins over a lapsed one. Holding two at once is a convention
/// nothing enforces, so the tie is broken rather than left to ref order.
pub fn on_task(cwd: &Path, identity: &str) -> Result<Option<Standing>> {
    let mut lapsed_one: Option<Standing> = None;
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        let Some(held) = read(cwd, &id)? else {
            continue;
        };
        let Record::Claim(c) = held.record else {
            continue;
        };
        if c.holder != identity {
            continue;
        }
        let lapsed = is_expired(&c, now_secs(), &id)?;
        let standing = Standing {
            id,
            object: held.object,
            record: c,
            lapsed,
        };
        if !lapsed {
            return Ok(Some(standing));
        }
        lapsed_one.get_or_insert(standing);
    }
    Ok(lapsed_one)
}

/// The standing of one named task, if this identity is the one on it.
///
/// [`on_task`] asked at a task instead of at the refs. It is what the explicit
/// id on `log`, `release` and `done` resolves through: §4 makes that id "a task
/// this agent holds a live claim on" rather than "HEAD spelled out", so naming
/// a task and deriving HEAD have to produce the same kind of answer or the two
/// paths drift (TASK-97d8747416ea).
///
/// A lapsed claim answers here exactly as it does in `on_task`, and for the
/// same reason: it is still this agent's task, and `log` and `done` retake it
/// (§3).
pub fn standing_on(cwd: &Path, identity: &str, id: &EntityId) -> Result<Option<Standing>> {
    let Some(held) = read(cwd, id)? else {
        return Ok(None);
    };
    let Record::Claim(c) = held.record else {
        return Ok(None);
    };
    if c.holder != identity {
        return Ok(None);
    }
    let lapsed = is_expired(&c, now_secs(), id)?;
    Ok(Some(Standing {
        id: id.clone(),
        object: held.object,
        record: c,
        lapsed,
    }))
}

/// What a verb says when it derived HEAD out of more than one live claim.
///
/// The choice itself stays deterministic — the lowest id, because the refs are
/// enumerated in refname order — and §3 asks that it never be silent. That is
/// the whole defect this answers: `log` wrote its entry, `release` handed a
/// task back and `done` verified one, none of them saying which of the two it
/// had picked or that there had been a choice (TASK-97d8747416ea).
///
/// Written once and said by every caller, like [`way_out`] which it ends with:
/// the sentences name a task id and a variable, and two copies are two chances
/// to name the wrong one.
///
/// Empty when there is nothing to report, so a caller can print it
/// unconditionally and the nominal case stays silent.
/// `tail` is what the verb needs after the id to stay a command somebody can
/// run: a message for `log`, a reason for `release`. §4 makes a hint the exact
/// command to run next, and `ank log TASK-8f3a` on its own is a different verb
/// — it reads.
pub fn sharing_warnings(
    verb: &str,
    tail: &str,
    acting_on: &EntityId,
    also: &[(EntityId, ClaimRecord)],
) -> Vec<String> {
    if also.is_empty() {
        return Vec::new();
    }
    let mut said = vec![format!(
        "{} live claims on this identity, acting on {acting_on}",
        also.len() + 1
    )];
    said.extend(
        also.iter()
            .map(|(id, c)| format!("also holding {id} until {}", c.expires)),
    );
    // The command, not the advice: naming another task is what the explicit id
    // is for, and the agent that meant the other one needs to be able to copy
    // the line rather than derive it (§4).
    said.push(format!("ank {verb} {}{tail} acts on that one", also[0].0));
    said.push(way_out());
    said
}

/// Takes a lapsed claim back, in the same agent's name (§3).
///
/// **The anchors are carried over, never recomputed.** Re-freezing the
/// criterion here would erase a divergence introduced while the claim was down,
/// which is the one thing `done` checks the hash to catch — the point of
/// re-acquisition is to restore the lock, not to re-bless the work. The expiry
/// moves, and the lease it moves by is the one the claim was granted.
///
/// The compare-and-swap is on the object the record was read from, so an agent
/// that took the task over between the read and this write keeps it.
pub fn retake(cwd: &Path, standing: &Standing, cap: Duration) -> Result<Written> {
    let ttl = renewal_ttl(&standing.record, cap);
    let record = Record::Claim(ClaimRecord {
        expires: format_utc(now_secs() + ttl.as_secs() as i64),
        ttl: ttl.as_secs(),
        ..standing.record.clone()
    });
    put(cwd, &standing.id, &record, Some(&standing.object))
}

/// The claim in force on a task, if there is one.
///
/// **The one place the question "is this criterion frozen right now" is
/// answered.** `edit` asked it first, and `amend --criteria` asks exactly the
/// same thing (§4): the two must agree, and a rule restated in two verbs is a
/// rule that will eventually be restated differently. A completion ref is not a
/// claim and neither is a lapsed one — an expired claim is not in force and
/// freezes nothing, which is the reading `log` and `done` already apply.
///
/// Any live claim, not this agent's: refusals are on state and never on identity
/// (ADR-c656cbcc33a9).
pub fn live(cwd: &Path, id: &EntityId) -> Result<Option<ClaimRecord>> {
    let Some(Record::Claim(c)) = read(cwd, id)?.map(|h| h.record) else {
        return Ok(None);
    };
    if is_expired(&c, now_secs(), id)? {
        return Ok(None);
    }
    Ok(Some(c))
}

fn remaining_text(claim: &ClaimRecord, now: i64) -> String {
    match parse_utc(&claim.expires) {
        Some(e) if e > now => {
            let mins = (e - now + 59) / 60;
            format!("expires in {mins}m")
        }
        Some(_) => "expired".to_string(),
        None => format!("expires {}", claim.expires),
    }
}

// ---------------------------------------------------------------------------
// The two hashes anchored at pickup
// ---------------------------------------------------------------------------

/// The accepted constraints bearing on a task's scope, sorted by id. Only
/// `accepted` counts: a proposal binds nobody, so including it would make the
/// hash move on a decision that has not been taken.
///
/// Scope intersection is glob against glob, and it is tested in both
/// directions — the ADR may be the wider of the two (`crates/ank-cli/**`
/// against one file) or the narrower (one file against `crates/**`). The rule
/// is approximate by nature, and it is deliberately in one named place:
/// `context` needs the same one (TASK-d4e5f6a7b8c9), and two implementations
/// drifting apart would show up as a constraint hash that moves for no reason.
pub fn applicable_constraints(
    store: &Store,
    repo: &Repo,
    task: &Task,
) -> Result<Vec<(String, String)>> {
    let mut found = Vec::new();
    for adr in bearing_on(store, repo, task)? {
        // Suspended, not merely reported. Injecting a constraint that no longer
        // matches what was ratified would let whoever edited the file rewrite
        // the rule every agent afterwards works under, which is the one thing
        // the freeze exists to prevent (§3).
        if matches!(
            crate::human::freeze_state(repo, &adr),
            Freeze::Altered { .. }
        ) {
            continue;
        }
        found.push((adr.id.to_string(), adr.constraint.clone()));
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(found)
}

/// The ids `applicable_constraints` withheld for having diverged from their
/// ratification.
///
/// `context` names them, and that is not decoration: a constraint that vanishes
/// in silence is worse than one that binds wrongly, because an absence is the
/// one thing a reader cannot notice.
pub fn suspended_constraints(store: &Store, repo: &Repo, task: &Task) -> Result<Vec<String>> {
    let mut found: Vec<String> = bearing_on(store, repo, task)?
        .into_iter()
        .filter(|adr| {
            matches!(
                crate::human::freeze_state(repo, adr),
                Freeze::Altered { .. }
            )
        })
        .map(|adr| adr.id.to_string())
        .collect();
    found.sort();
    Ok(found)
}

/// Every accepted ADR whose scope meets the task's, before any question of
/// whether it still says what was ratified.
fn bearing_on(store: &Store, _repo: &Repo, task: &Task) -> Result<Vec<Adr>> {
    let mut found = Vec::new();
    for id in store.list_ids()? {
        if id.kind() != ank_core::EntityKind::Adr {
            continue;
        }
        let loaded = store.load(&id)?;
        let Entity::Adr(adr) = loaded.entity else {
            continue;
        };
        if adr.status != AdrStatus::Accepted {
            continue;
        }
        if scopes_intersect(&task.scope, &adr.scope)? {
            found.push(adr);
        }
    }
    Ok(found)
}

fn scopes_intersect(a: &[String], b: &[String]) -> Result<bool> {
    let invalid = |e: ank_core::Error| CliError::new(1, format!("invalid scope: {e}"));
    let set_a = ScopeSet::new(a).map_err(invalid)?;
    let set_b = ScopeSet::new(b).map_err(invalid)?;
    Ok(a.iter().any(|g| set_b.overlaps_dir(g, b)) || b.iter().any(|g| set_a.overlaps_dir(g, a)))
}

/// Hash of a constraint set. Stable under reordering (the entries are sorted)
/// and under editing noise (each constraint is normalised the way every other
/// freeze is), so it moves only when a constraint really enters, leaves or
/// changes.
pub fn constraints_hash(constraints: &[(String, String)]) -> String {
    let mut buf = String::new();
    for (id, text) in constraints {
        buf.push_str(id);
        buf.push('\n');
        buf.push_str(&freeze::normalize(text));
        buf.push('\n');
    }
    freeze_hash_short(&buf)
}

// ---------------------------------------------------------------------------
// Taking a task
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Acquired {
    pub id: EntityId,
    pub holder: String,
    pub expires: String,
    /// True when the ref already carried a claim we replaced — the original
    /// holder returning after expiry, or a takeover from a lapsed one.
    pub taken_over: bool,
    /// How far the claim reached: this clone, or the whole repository (§7).
    pub sync: Sync,
}

/// Takes the ref for `identity`, or says precisely why it cannot be taken.
///
/// Every branch of §7 and §3 lives here, and none of them falls through to
/// another: an absent ref is created under a CAS that requires absence, a live
/// claim held by somebody else is code 4, a lapsed one is taken over under a
/// CAS on the object we read, a completion is code 4 with its own message, and
/// an unreadable record is named rather than guessed at.
///
/// This function touches no file. The `open -> in_progress` move belongs to
/// the verb, which does it after the ref is held.
pub fn acquire(
    cwd: &Path,
    task: &Task,
    identity: &str,
    ttl: Duration,
    criteria_hash: &str,
    constraints_hash: &str,
    other_ready: Option<&str>,
) -> Result<Acquired> {
    let id = &task.id;
    let now = now_secs();
    let held = read(cwd, id)?;

    let witness = match &held {
        None => None,
        Some(h) => match &h.record {
            Record::Completed(c) => return Err(finished_elsewhere(id, c, other_ready)),
            // An attestation on the claim ref is a corrupt plane and never a
            // free task: overwriting it here would destroy a record while
            // taking a claim, and reading it as absence would hand out a task
            // whose real state nobody can see.
            Record::Proof(_) => {
                return Err(corrupt(&claim_ref(id), "a proof record on the claim ref"))
            }
            Record::Claim(c) => {
                // The holder returning to a claim of its own that is still
                // live is a renewal, and it is silent: working is what keeps
                // the lock, there is no heartbeat verb to memorise.
                if !is_expired(c, now, id)? && c.holder != identity {
                    return Err(held_by_other(id, c, now, other_ready));
                }
                Some(h.object.as_str())
            }
        },
    };

    let record = Record::Claim(ClaimRecord {
        task: id.to_string(),
        holder: identity.to_string(),
        claimed: format_utc(now),
        expires: format_utc(now + ttl.as_secs() as i64),
        ttl: ttl.as_secs(),
        criteria: criteria_hash.to_string(),
        constraints: constraints_hash.to_string(),
    });

    let written = put(cwd, id, &record, witness)?;
    match written.cas {
        Cas::Won => {}
        // Somebody landed between our read and our write — in this clone, or in
        // another one whose push got there first (§7). Re-read to name them:
        // the winner is the one whose message the agent needs, and `put` has
        // already brought a remote winner's record here for that.
        Cas::Lost => return Err(lost_the_race(cwd, id, now, other_ready)?),
    }

    let Record::Claim(taken) = record else {
        unreachable!("just built as a claim")
    };
    Ok(Acquired {
        id: id.clone(),
        holder: taken.holder,
        expires: taken.expires,
        taken_over: witness.is_some(),
        sync: written.sync,
    })
}

/// Replaces whatever the ref carries with a completion record, keeping the
/// address. Called by `done` (TASK-e5f6a7b8c9d0); no TTL is written, because
/// what ends the completion ref is durable state catching up, not time.
///
/// The completion is pushed like every other write of the plane, and that is
/// what makes it useful across clones (§7): a completion ref that stayed local
/// would tell the other clones nothing, which is exactly the window it exists
/// to close.
pub fn complete(cwd: &Path, id: &EntityId, identity: &str) -> Result<(CompletedRecord, Sync)> {
    let commit = git::run(cwd, &["rev-parse", "HEAD"])?;
    let branch = git::current_branch(cwd)?;
    let witness = current_object(cwd, &claim_ref(id))?;
    let record = CompletedRecord {
        task: id.to_string(),
        commit,
        branch,
        identity: identity.to_string(),
        completed: now_utc(),
    };
    match put(
        cwd,
        id,
        &Record::Completed(record.clone()),
        witness.as_deref(),
    )? {
        Written {
            cas: Cas::Won,
            sync,
        } => Ok((record, sync)),
        Written { cas: Cas::Lost, .. } => Err(CliError::new(
            4,
            format!("{id} moved while it was being completed"),
        )
        .with_hint(format!("ank claim {id}"))),
    }
}

// ---------------------------------------------------------------------------
// The two refusals of code 4, which must never read alike
// ---------------------------------------------------------------------------

fn other_task_hint(other_ready: Option<&str>) -> String {
    match other_ready {
        Some(id) => format!("ank claim {id}   (another ready task in this scope)"),
        None => "ank context".to_string(),
    }
}

fn held_by_other(
    id: &EntityId,
    claim: &ClaimRecord,
    now: i64,
    other_ready: Option<&str>,
) -> CliError {
    CliError::new(
        4,
        format!(
            "{id} held by {} ({})",
            claim.holder,
            remaining_text(claim, now)
        ),
    )
    .with_hint(other_task_hint(other_ready))
}

fn finished_elsewhere(
    id: &EntityId,
    done: &CompletedRecord,
    other_ready: Option<&str>,
) -> CliError {
    let commit: String = done.commit.chars().take(7).collect();
    let where_ = match &done.branch {
        Some(b) => format!("commit {commit}, branch {b}"),
        None => format!("commit {commit}, detached HEAD"),
    };
    CliError::new(
        4,
        format!("{id} finished on another branch ({where_}), not merged here yet"),
    )
    .with_hint(other_task_hint(other_ready))
}

/// The same fact as `finished_elsewhere`, about a blocker rather than about the
/// task being claimed: code 7, because what refuses here is the prerequisite,
/// not the ref.
fn blocker_finished_elsewhere(
    task: &EntityId,
    blocker: &EntityId,
    done: &CompletedRecord,
    other_ready: Option<&str>,
) -> CliError {
    let commit: String = done.commit.chars().take(7).collect();
    let where_ = match &done.branch {
        Some(b) => format!("finished on {b} (commit {commit})"),
        None => format!("finished on a detached HEAD (commit {commit})"),
    };
    CliError::new(
        7,
        format!("{task} is blocked by {blocker}, {where_}, not merged here yet"),
    )
    .with_hint(other_task_hint(other_ready))
}

/// Who holds it now, for a caller whose compare-and-swap lost.
///
/// Shared with `done`'s retake of a lapsed claim (§3): "if another agent took
/// it over in the meantime, they fail with code 4 and the name of the new
/// holder" is the same answer this already produces for a lost claim, and
/// producing it twice would be two chances to word it differently.
pub fn taken_over_since(cwd: &Path, id: &EntityId) -> Result<CliError> {
    lost_the_race(cwd, id, now_secs(), None)
}

fn lost_the_race(
    cwd: &Path,
    id: &EntityId,
    now: i64,
    other_ready: Option<&str>,
) -> Result<CliError> {
    Ok(match read(cwd, id)? {
        Some(Held {
            record: Record::Claim(c),
            ..
        }) => held_by_other(id, &c, now, other_ready),
        Some(Held {
            record: Record::Completed(c),
            ..
        }) => finished_elsewhere(id, &c, other_ready),
        Some(Held {
            record: Record::Proof(_),
            ..
        }) => corrupt(&claim_ref(id), "a proof record on the claim ref"),
        None => CliError::new(4, format!("{id} was taken and released while claiming"))
            .with_hint(format!("ank claim {id}")),
    })
}

// ---------------------------------------------------------------------------
// Pruning — exposed here, called by `check`
// ---------------------------------------------------------------------------

/// Deletes the refs of tasks that appear `done` or `closed` **on the default
/// branch**: the information the ref carried is then present where everybody
/// reads it, and the ref has no further use.
///
/// The predicate is the file as the branch carries it, never the reachability
/// of the recorded commit (ADR-bcf222a31525): `done` writes to the working
/// tree, so the commit it records is frequently already an ancestor, and an
/// agent that branched and finished before its first commit would see its ref
/// vanish within the second — reopening the exact window the mechanism closes.
///
/// `tasks_prefix` is repository-relative and `/`-separated, git's own syntax on
/// the three platforms — `.ank/tasks` in the ordinary layout.
///
/// **Nothing in this module calls it.** `check` does (TASK-a7b8c9d0e1f2): a
/// reader does not sanitise the coordination plane underneath everyone else,
/// and concentrating maintenance in one command is what makes its timing
/// predictable.
// The predicate itself lives in `human::maintain`, which is what `check` runs.
// A second implementation stood here for a long time, documented as "called by
// check" and called by nothing outside its own tests (TASK-4981a1370c0b): two
// copies of the rule that decides when a coordination ref disappears, and the
// unexercised one free to drift from the one that runs. It was already the
// weaker copy — it had never learned, as `maintain` did from
// TASK-52fbffbfdf65, to ask the default branch whether a task exists before
// treating its ref as an orphan, because a checkout older than a task sees no
// such task and used to delete a claim another worktree was holding.
//
// What a wrong pruning decision costs is a lost claim, which is a task two
// agents can hold at once. That is why the duplication was worth removing
// rather than tidying.

// ---------------------------------------------------------------------------
// The verb
// ---------------------------------------------------------------------------

/// `ank claim <id> [--criteria <c>] [--ttl 30m]`.
///
/// Called by `cli.rs::dispatch`, which resolves the repository, the config and
/// the identity once and hands them to every verb: establishing them is the
/// foundation's job, and a verb doing it again would be free to do it
/// differently.
pub fn run(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let prefix = inv
        .positionals
        .first()
        .ok_or_else(|| CliError::new(1, "claim expects a task id").with_hint("ank claim <id>"))?;
    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = crate::store::version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{prefix} is not a task"))
            .with_hint(format!("ank show {prefix}")));
    };

    let ttl = resolve_ttl(inv.value("--ttl"), cfg)?;
    let ready = other_ready_task(&repo.root, &store, &task).map(|id| id.to_string());

    // Preconditions first, in the order of §3: no ref is touched by a claim
    // that was never going to be legal.
    //
    // **`--criteria` sets an absent criterion and never replaces one** (§4).
    // §3 gives the flag exactly one job: a task cannot be claimed without a
    // criterion, and the refusal below names the command that sets it and
    // claims in the same call. Overwriting silently was a second job nobody
    // specified, and it was the wrong one — it recorded the correction of a
    // criterion that had turned out unmeasurable as the claimer's, which is the
    // shape `criteria_by` exists to expose (TASK-7c2fa14284ff). Correcting an
    // existing criterion is `amend --criteria`, which the freeze governs on
    // state: refused while a claim is live, allowed the rest of the time.
    if let Some(c) = inv.value("--criteria") {
        if !c.trim().is_empty() {
            let carried = task
                .done_criteria
                .as_deref()
                .is_some_and(|existing| !existing.trim().is_empty());
            if carried {
                return Err(CliError::new(
                    6,
                    format!("{} already carries a done_criteria", task.id),
                )
                .with_hint(format!(
                    "ank amend {} --criteria \"<verifiable criterion>\"",
                    task.id
                )));
            }
            task.done_criteria = Some(ensure_trailing_newline(c));
            task.criteria_by = Some(CriteriaBy::Claimer);
        }
    }
    let criteria = match task.done_criteria.as_deref() {
        Some(c) if !c.trim().is_empty() => c.to_string(),
        _ => {
            return Err(
                CliError::new(7, format!("{} has no done_criteria", task.id)).with_hint(format!(
                    "ank claim {} --criteria \"<verifiable criterion>\"",
                    task.id
                )),
            )
        }
    };
    // The remote's view first, so a task held in another clone is refused with
    // its holder named rather than taken here and unwound by a rejected push
    // (§7). Silent when there is no remote, and silent when there is one and it
    // cannot be reached — that is the write's news to break, not the read's.
    sync_from_remote(&repo.root, &task.id)?;

    check_blockers(&repo.root, &store, &task, ready.as_deref())?;
    task.status
        .check_transition(TaskStatus::InProgress)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint(format!("ank show {}", task.id)))?;

    // Last of the preconditions, and last on purpose. Everything above refuses
    // on the task asked for; this one refuses on what the caller already holds,
    // and an agent told to hand back its work for a task that would have
    // refused it anyway has paid for nothing.
    already_holding(&repo.root, identity, &task.id, now_secs())?;

    let constraints = constraints_hash(&applicable_constraints(&store, repo, &task)?);
    let acquired = acquire(
        &repo.root,
        &task,
        identity,
        ttl,
        &freeze_hash_short(&criteria),
        &constraints,
        ready.as_deref(),
    )?;

    // Durable state last, and it carries the transition alone: no holder, no
    // expiry, no TTL ever reaches a task file (ADR-4e7c25b1f639).
    task.status = TaskStatus::InProgress;
    store.write(&Entity::Task(task.clone()), base_version)?;

    // Read a second time, after the ref is taken, and what it can still find is
    // the race `already_holding` cannot close: two sessions of one identity
    // that both passed the check before either took its ref. The window is
    // narrow and it is real, so it is named rather than assumed away — the
    // refusal above is what makes this the exception it now reads as.
    //
    // It still warns and never refuses, and here that is the only option left:
    // the ref is taken and the transition is written, so a refusal at this
    // point would refuse a claim the agent holds. `status` says the same thing
    // for as long as the state lasts (TASK-38b384543551).
    let also_held = live_claims_of(&repo.root, identity, &acquired.id, now_secs())?;
    let warnings: Vec<String> = also_held
        .iter()
        .map(|(id, c)| format!("{identity} already holds {id} until {}", c.expires))
        .chain((!also_held.is_empty()).then(way_out))
        // A claim that did not reach the remote holds in this clone alone, and
        // §7 is explicit that the risk is displayed rather than hidden.
        .chain(acquired.sync.warning())
        .collect();

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{}\",\"holder\":\"{}\",\"expires\":\"{}\",\"warnings\":[{}]}}",
            acquired.id,
            acquired.holder,
            acquired.expires,
            warnings
                .iter()
                .map(|w| crate::commands::json_string(w))
                .collect::<Vec<_>>()
                .join(",")
        );
    } else {
        // A warning survives `--quiet`: what it reports is not the confirmation
        // the flag is there to silence.
        for w in &warnings {
            let _ = writeln!(out, "{} {w}", inv.style().yellow("warning:"));
        }
        if !inv.quiet() {
            let slug = task.slug.as_deref().unwrap_or("");
            let _ = writeln!(
                out,
                "{} {} {slug} -> HEAD",
                inv.style().advanced("claimed"),
                inv.style().id(&acquired.id.to_string())
            );
        }
    }
    Ok(0)
}

/// A criterion, normalised for storage. Shared with `amend --criteria`: the two
/// routes write the same field and have to write it identically, or the corpus
/// would record which command was used.
pub fn ensure_trailing_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    }
}

/// `--ttl`, defaulted and then capped by `claim_ttl_max`: an agent cannot grant
/// itself twenty-four hours and hoard (§3).
fn resolve_ttl(flag: Option<&str>, cfg: &Config) -> Result<Duration> {
    let asked = match flag {
        Some(v) => config::parse_duration(v)
            .map_err(|e| CliError::new(1, e).with_hint("ank claim <id> --ttl 30m"))?,
        None => DEFAULT_TTL,
    };
    Ok(asked.min(cfg.claim_ttl_max))
}

/// The lease a renewal recomputes the expiry from (§3).
///
/// **Re-capped here and not only at the claim**, so that lowering
/// `claim_ttl_max` takes effect on the next `log` rather than waiting for the
/// next claim. A record from before the field existed carries `0`, which is not
/// a lease anybody granted: it reads as the default, the same way an absent
/// field does everywhere else in the format.
pub fn renewal_ttl(record: &ClaimRecord, cap: Duration) -> Duration {
    let granted = match record.ttl {
        0 => DEFAULT_TTL,
        secs => Duration::from_secs(secs),
    };
    granted.min(cap)
}

fn status_map(store: &Store) -> Result<HashMap<EntityId, TaskStatus>> {
    let mut map = HashMap::new();
    for id in store.list_ids()? {
        if id.kind() != ank_core::EntityKind::Task {
            continue;
        }
        if let Entity::Task(t) = store.load(&id)?.entity {
            map.insert(id, t.status);
        }
    }
    Ok(map)
}

/// A blocker that is not `done` refuses the claim with code 7 and names it.
/// `closed` does not unblock either — the work was not carried out (§3).
///
/// A blocker finished on another branch reads, in the working tree, exactly
/// like one nobody has started: `status_map` sees the file this branch carries
/// and nothing else. The completion ref is the only witness of that window
/// (ADR-bcf222a31525), so it is consulted here. The refusal itself does not
/// move — claiming on top of unmerged work is the real risk — only what the
/// agent is told about it, which is the answer `acquire` already gives for the
/// claimed task itself.
fn check_blockers(cwd: &Path, store: &Store, task: &Task, other_ready: Option<&str>) -> Result<()> {
    let map = status_map(store)?;
    let blockers = task
        .active_blockers(|id| map.get(id).copied())
        .map_err(|e| CliError::new(7, e.to_string()).with_hint("ank check"))?;

    // A blocker carrying a completion ref is named ahead of the first one,
    // wherever it sits in the list: it is the one fact the plain message hides,
    // and the order of `blocked_by` says nothing about which blocker matters.
    for id in &blockers {
        if let Some(Held {
            record: Record::Completed(c),
            ..
        }) = read(cwd, id)?
        {
            return Err(blocker_finished_elsewhere(&task.id, id, &c, other_ready));
        }
    }

    if let Some(first) = blockers.first() {
        let closed = map.get(*first) == Some(&TaskStatus::Closed);
        let why = if closed { " (closed)" } else { "" };
        return Err(
            CliError::new(7, format!("{} is blocked by {first}{why}", task.id))
                .with_hint(format!("ank show {first}")),
        );
    }
    Ok(())
}

/// Another task the agent could take instead, for the hint on a refusal (§4).
/// Same scope first, since that is what the message claims; any ready task
/// otherwise is not offered, because pointing somewhere else entirely would be
/// the generic help the style forbids.
///
/// A candidate carrying a completion ref is skipped: its file says `open` on
/// this branch, and offering it would print an exact command that refuses the
/// moment it is run — which is the generic help by another route.
fn other_ready_task(cwd: &Path, store: &Store, task: &Task) -> Option<EntityId> {
    let map = status_map(store).ok()?;
    let mut candidates: Vec<&EntityId> = map
        .iter()
        .filter(|(id, st)| **st == TaskStatus::Open && **id != task.id)
        .map(|(id, _)| id)
        .collect();
    candidates.sort_by_key(|id| id.to_string());
    for id in candidates {
        let Ok(Entity::Task(t)) = store.load(id).map(|l| l.entity) else {
            continue;
        };
        if t.active_blockers(|b| map.get(b).copied())
            .map(|v| !v.is_empty())
            .unwrap_or(true)
        {
            continue;
        }
        if !scopes_intersect(&task.scope, &t.scope).unwrap_or(false) {
            continue;
        }
        if matches!(
            read(cwd, id),
            Ok(Some(Held {
                record: Record::Completed(_),
                ..
            }))
        ) {
            continue;
        }
        return Some(id.clone());
    }
    None
}

/// Anchors the ADRs whose constraint applies here, so that a reader of this
/// module lands on them: claims in refs (ADR-4e7c25b1f639), the ref's second
/// state (ADR-bcf222a31525), plumbing by criterion (ADR-b8884edcebe3), freeze
/// by hash (ADR-6b3f19e08a24), one surface (ADR-c656cbcc33a9).
#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{serialize_entity, Adr};
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    // -----------------------------------------------------------------------
    // Harness: a real repository, because refs are what is under test
    // -----------------------------------------------------------------------

    struct Temp(PathBuf);

    impl Temp {
        /// Same shape as `git.rs::tests::Temp`: a known branch name, a local
        /// identity the CI runners lack, and autocrlf pinned so that what is
        /// read back is what was written.
        fn new_repo() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-claim-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
            std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
            let t = Temp(p);
            t.porcelain(&["init", "-q", "-b", "main"]);
            t.porcelain(&["config", "user.email", "test@ank.local"]);
            t.porcelain(&["config", "user.name", "Test"]);
            t.porcelain(&["config", "core.autocrlf", "false"]);
            // Signing off at creation, not at each commit (TASK-40a972e98a9a).
            t.porcelain(&["config", "commit.gpgsign", "false"]);
            t.porcelain(&["config", "tag.gpgsign", "false"]);
            t
        }

        /// Porcelain is forbidden to the tool (ADR-b8884edcebe3), not to the
        /// harness that builds the fixture.
        fn porcelain(&self, args: &[&str]) {
            let st = Command::new("git")
                .current_dir(&self.0)
                .args(args)
                .status()
                .expect("git must be installed: it is a hard dependency");
            assert!(st.success(), "git {args:?}");
        }

        fn commit_all(&self, message: &str) -> String {
            self.porcelain(&["add", "-A"]);
            self.porcelain(&["commit", "-qm", message]);
            git::run(&self.0, &["rev-parse", "HEAD"]).unwrap()
        }

        fn store(&self) -> Store {
            Store::new(self.0.join(".ank"))
        }

        fn repo(&self) -> Repo {
            Repo {
                root: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }

        fn seed(&self, task: &Task) {
            let e = Entity::Task(task.clone());
            std::fs::write(
                self.0.join(".ank/tasks").join(format!("{}.md", task.id)),
                serialize_entity(&e),
            )
            .unwrap();
        }

        fn task_text(&self, id: &EntityId) -> String {
            std::fs::read_to_string(self.0.join(".ank/tasks").join(format!("{id}.md"))).unwrap()
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn open_task(hex: &str) -> Task {
        Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "Example task".into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status: TaskStatus::Open,
            scope: vec!["src/**".into()],
            blocked_by: vec![],
            done_criteria: Some("A verifiable criterion.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nFree body.\n".into(),
        }
    }

    /// The lapsed half of the warning, which the integration test cannot reach:
    /// the drift tolerance is two minutes, so a claim expiring through the clock
    /// costs two minutes of wall time. Here `now` is a parameter and the lapse
    /// is free.
    #[test]
    fn a_lapsed_claim_is_not_something_its_holder_still_holds() {
        let t = Temp::new_repo();
        let a = open_task("00000000ee01");
        let b = open_task("00000000ee02");
        t.seed(&a);
        t.seed(&b);
        t.commit_all("seed");

        let ttl = Duration::from_secs(30 * 60);
        acquire(&t.0, &a, "mia@laptop", ttl, "h", "c", None).unwrap();

        let now = now_secs();
        let held = live_claims_of(&t.0, "mia@laptop", &b.id, now).unwrap();
        assert_eq!(held.len(), 1, "held while live: {held:?}");
        assert_eq!(held[0].0, a.id);

        // Past the expiry and past the tolerance, the ref is still there and
        // nobody holds it: that is exactly the state a takeover reads.
        let later = now + ttl.as_secs() as i64 + DRIFT_TOLERANCE.as_secs() as i64 + 1;
        let held = live_claims_of(&t.0, "mia@laptop", &b.id, later).unwrap();
        assert!(held.is_empty(), "a lapsed claim is not held: {held:?}");

        // And it was never anybody else's to begin with.
        let held = live_claims_of(&t.0, "codex@ank", &b.id, now).unwrap();
        assert!(held.is_empty(), "another identity holds nothing: {held:?}");
    }

    fn adr(hex: &str, scope: &[&str], constraint: &str, status: AdrStatus) -> Entity {
        Entity::Adr(Adr {
            id: EntityId::parse(&format!("ADR-{hex}")).unwrap(),
            slug: Some("example".into()),
            title: "An example decision".into(),
            created: "2026-07-28T00:00:00Z".into(),
            author: None,
            status,
            scope: scope.iter().map(|s| s.to_string()).collect(),
            constraint: constraint.into(),
            see: None,
            supersedes: None,
            ratified: None,
            verified: Vec::new(),
            schema: 1,
            version: 1,
            body: "\nWhy.\n".into(),
        })
    }

    fn take(t: &Temp, task: &Task, who: &str, ttl: Duration) -> Result<Acquired> {
        acquire(&t.0, task, who, ttl, "aaaabbbbcccc", "ddddeeeeffff", None)
    }

    // -----------------------------------------------------------------------
    // The assumption everything rests on
    // -----------------------------------------------------------------------

    #[test]
    fn a_ref_can_point_at_a_blob_which_is_what_a_record_is() {
        // If this ever fails, nothing below is worth reading: the record would
        // have to become a tag object, and `mktag` is not in the plumbing list
        // that ADR-b8884edcebe3 allows.
        let t = Temp::new_repo();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        let record = Record::Claim(ClaimRecord {
            task: id.to_string(),
            holder: "claude-code@ank".into(),
            claimed: "2026-07-31T02:00:00Z".into(),
            expires: "2026-07-31T02:30:00Z".into(),
            ttl: DEFAULT_TTL.as_secs(),
            criteria: "aaaabbbbcccc".into(),
            constraints: "ddddeeeeffff".into(),
        });
        assert_eq!(put(&t.0, &id, &record, None).unwrap().cas, Cas::Won);

        let refs = git::ank_refs(&t.0).unwrap();
        assert_eq!(refs.len(), 1, "{refs:?}");
        assert_eq!(refs[0].name, claim_ref(&id));
        assert_eq!(read(&t.0, &id).unwrap().unwrap().record, record);
    }

    // -----------------------------------------------------------------------
    // The record and its two states
    // -----------------------------------------------------------------------

    /// A record written before the lease was recorded still reads, and renews
    /// at the default.
    ///
    /// The promise §7 makes about the coordination plane, and the same one the
    /// entity format makes about fields introduced later: absence means
    /// "written before this existed", never "invalid". Written out as bytes
    /// rather than built from the struct, because what has to keep parsing is
    /// what is already sitting in somebody's `refs/ank/claims/`.
    #[test]
    fn a_record_from_before_the_lease_reads_and_renews_at_the_default() {
        let id = EntityId::parse("TASK-000000000001").unwrap();
        let text = "state: claim\ntask: TASK-000000000001\nholder: claude-code@ank\n\
                    claimed: 2026-07-31T02:00:00Z\nexpires: 2026-07-31T02:30:00Z\n\
                    criteria: '123456789012'\nconstraints: '000000000000'\n";
        let Record::Claim(c) = parse_record(text, &claim_ref(&id)).unwrap() else {
            panic!("a claim record");
        };
        assert_eq!(c.ttl, 0, "an absent lease is not a lease anybody granted");

        assert_eq!(renewal_ttl(&c, Duration::from_secs(2 * 3600)), DEFAULT_TTL);

        // And the cap still binds a record that carries no lease of its own.
        assert_eq!(
            renewal_ttl(&c, Duration::from_secs(300)),
            Duration::from_secs(300)
        );
    }

    /// The lease is re-capped at renewal, so lowering `claim_ttl_max` takes
    /// effect on the next `log` rather than waiting for the next claim.
    #[test]
    fn a_lowered_cap_binds_the_renewal_and_not_only_the_claim() {
        let granted = ClaimRecord {
            task: "TASK-000000000001".into(),
            holder: "claude-code@ank".into(),
            claimed: "2026-07-31T02:00:00Z".into(),
            expires: "2026-07-31T04:00:00Z".into(),
            ttl: 2 * 3600,
            criteria: "123456789012".into(),
            constraints: "000000000000".into(),
        };
        assert_eq!(
            renewal_ttl(&granted, Duration::from_secs(2 * 3600)),
            Duration::from_secs(2 * 3600)
        );
        assert_eq!(
            renewal_ttl(&granted, Duration::from_secs(45 * 60)),
            Duration::from_secs(45 * 60)
        );
    }

    #[test]
    fn an_all_digit_hash_survives_the_round_trip_as_a_string() {
        // A twelve-character hash can come out all decimal digits. Unquoted,
        // YAML reads it back as a number and the record stops parsing --
        // exactly the trap `ank check` catches on proof entries.
        let id = EntityId::parse("TASK-000000000001").unwrap();
        let record = Record::Claim(ClaimRecord {
            task: id.to_string(),
            holder: "claude-code@ank".into(),
            claimed: "2026-07-31T02:00:00Z".into(),
            expires: "2026-07-31T02:30:00Z".into(),
            ttl: DEFAULT_TTL.as_secs(),
            criteria: "123456789012".into(),
            constraints: "000000000000".into(),
        });
        let text = serialize_record(&record);
        assert_eq!(
            parse_record(&text, &claim_ref(&id)).unwrap(),
            record,
            "{text}"
        );

        let done = Record::Completed(CompletedRecord {
            task: id.to_string(),
            commit: "1234567890123456789012345678901234567890".into(),
            branch: Some("main".into()),
            identity: "claude-code@ank".into(),
            completed: "2026-07-31T02:30:00Z".into(),
        });
        let text = serialize_record(&done);
        assert_eq!(
            parse_record(&text, &claim_ref(&id)).unwrap(),
            done,
            "{text}"
        );
    }

    #[test]
    fn an_unknown_state_is_named_never_read_as_the_other_one() {
        let id = EntityId::parse("TASK-000000000001").unwrap();

        let err = parse_record(
            "state: abandoned\ntask: TASK-000000000001\n",
            &claim_ref(&id),
        )
        .unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains("abandoned"), "{}", err.message);
        assert!(err.hint.unwrap().contains("update-ref -d"));

        // No state at all, a state of the wrong type, and a blob that is not a
        // mapping: three ways to be unreadable, none of them a claim and none
        // of them a completion.
        for text in [
            "task: TASK-000000000001\nholder: x\n",
            "state: [claim]\n",
            "not a mapping at all\n",
            "state: claim\ntask: TASK-000000000001\n",
        ] {
            let err = parse_record(text, &claim_ref(&id)).unwrap_err();
            assert_eq!(err.code, 9, "{text}");
            assert!(err.message.contains("unreadable record"), "{text}");
        }
    }

    #[test]
    fn a_corrupt_blob_on_the_ref_is_an_error_not_a_free_task() {
        let t = Temp::new_repo();
        let id = EntityId::parse("TASK-000000000001").unwrap();
        t.porcelain(&["config", "core.autocrlf", "false"]);
        let blob = {
            use std::io::Write as _;
            use std::process::Stdio;
            let mut c = Command::new("git")
                .current_dir(&t.0)
                .args(["hash-object", "-w", "--stdin"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();
            c.stdin
                .take()
                .unwrap()
                .write_all(b"state: something-else\n")
                .unwrap();
            let o = c.wait_with_output().unwrap();
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        };
        git::run(&t.0, &["update-ref", &claim_ref(&id), &blob]).unwrap();

        let err = read(&t.0, &id).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains(&claim_ref(&id)), "{}", err.message);
    }

    // -----------------------------------------------------------------------
    // Taking, refusing, expiring, returning
    // -----------------------------------------------------------------------

    #[test]
    fn a_claim_lands_on_the_ref_and_the_file_learns_only_in_progress() {
        // Through the verb, not through `acquire`: what the criterion says
        // must not appear in the file is what the whole command writes, and
        // moving the file by hand here would assert nothing about that.
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);
        let before = t.task_text(&task.id);

        let inv = invocation(&["claim", &task.id.to_string()]);
        assert_eq!(run_verb(&t, &inv).unwrap(), 0);

        let text = t.task_text(&task.id);
        assert!(text.contains("status: in_progress"), "{text}");
        assert_eq!(
            before
                .replace("status: open", "status: in_progress")
                .replace("version: 1", "version: 2"),
            text,
            "the transition and the version are the only things the file learns"
        );
        for forbidden in ["claude-code@ank", "expires", "holder", "ttl", "claim"] {
            assert!(
                !text.contains(forbidden),
                "'{forbidden}' reached the file, which is what ADR-4e7c25b1f639 forbids:\n{text}"
            );
        }

        // And the claim really is on the ref, with both anchoring hashes.
        match read(&t.0, &task.id).unwrap().unwrap().record {
            Record::Claim(c) => {
                assert_eq!(c.holder, "claude-code@ank");
                assert_eq!(c.task, task.id.to_string());
                assert!(parse_utc(&c.expires).unwrap() > parse_utc(&c.claimed).unwrap());
            }
            other => panic!("expected a claim, got {other:?}"),
        }
    }

    #[test]
    fn two_concurrent_claims_leave_exactly_one_winner_and_the_losers_get_4() {
        // Real threads on one real repository. A lock whose release failed
        // under concurrency once passed green unit tests here; simulating the
        // race would prove nothing about git's compare-and-swap.
        let t = Temp::new_repo();
        let task = Arc::new(open_task("000000000001"));
        t.seed(&task);
        let root = Arc::new(t.0.clone());
        let n = 12;

        let mut handles = Vec::new();
        for i in 0..n {
            let root = Arc::clone(&root);
            let task = Arc::clone(&task);
            handles.push(std::thread::spawn(move || {
                acquire(
                    &root,
                    &task,
                    &format!("agent-{i}@host"),
                    DEFAULT_TTL,
                    "aaaabbbbcccc",
                    "ddddeeeeffff",
                    None,
                )
            }));
        }
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let winners = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(winners, 1, "exactly one winner: {results:?}");
        for r in results.iter().filter(|r| r.is_err()) {
            let err = r.as_ref().unwrap_err();
            assert_eq!(err.code, 4, "the losers exit with 4: {}", err.render());
            assert!(err.hint.is_some(), "a refusal always says what to do next");
        }

        // One ref, one holder, and it is the winner.
        let refs = git::ank_refs(&t.0).unwrap();
        assert_eq!(refs.len(), 1, "{refs:?}");
        let winner = results.iter().find_map(|r| r.as_ref().ok()).unwrap();
        match read(&t.0, &task.id).unwrap().unwrap().record {
            Record::Claim(c) => assert_eq!(c.holder, winner.holder),
            other => panic!("expected a claim, got {other:?}"),
        }
    }

    #[test]
    fn a_live_claim_refuses_another_agent_and_names_the_holder() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);
        take(&t, &task, "codex@host-9", DEFAULT_TTL).unwrap();

        let err = take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap_err();
        assert_eq!(err.code, 4);
        assert!(err.message.contains("codex@host-9"), "{}", err.message);
        assert!(err.message.contains("expires in"), "{}", err.message);
    }

    #[test]
    fn an_expired_claim_makes_the_task_claimable_again() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);

        // A TTL of one second, then past the drift tolerance: expiry is read
        // off the timestamp the record carries, so no waiting is required.
        take(&t, &task, "codex@host-9", Duration::from_secs(1)).unwrap();
        let held = read(&t.0, &task.id).unwrap().unwrap();
        let Record::Claim(c) = held.record else {
            panic!("expected a claim")
        };
        let expiry = parse_utc(&c.expires).unwrap();
        assert!(
            !is_expired(&c, expiry + 60, &task.id).unwrap(),
            "inside the two-minute drift tolerance, the claim still stands"
        );
        assert!(is_expired(&c, expiry + 121, &task.id).unwrap());

        // Rewrite the record with an expiry well in the past, which is what a
        // 40-minute silent build produces.
        let stale = Record::Claim(ClaimRecord {
            expires: format_utc(now_secs() - 3600),
            ..c
        });
        put(&t.0, &task.id, &stale, Some(&held.object)).unwrap();

        let taken = take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert_eq!(taken.holder, "claude-code@ank");
        assert!(taken.taken_over, "the ref carried a lapsed claim");
        assert_eq!(git::ank_refs(&t.0).unwrap().len(), 1, "still one ref");
    }

    #[test]
    fn the_original_holder_re_acquires_silently_when_nobody_took_over() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);

        let first = take(&t, &task, "claude-code@ank", Duration::from_secs(1)).unwrap();
        let held = read(&t.0, &task.id).unwrap().unwrap();
        let Record::Claim(c) = held.record else {
            panic!("expected a claim")
        };
        let stale = Record::Claim(ClaimRecord {
            expires: format_utc(now_secs() - 3600),
            ..c
        });
        put(&t.0, &task.id, &stale, Some(&held.object)).unwrap();

        let again = take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert_eq!(again.holder, "claude-code@ank");
        assert!(
            parse_utc(&again.expires).unwrap() > parse_utc(&first.expires).unwrap(),
            "re-acquisition moves the expiry forward"
        );

        // And a live claim of one's own renews just as silently: working is
        // what keeps the lock, there is no heartbeat verb.
        let renewed = take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert!(renewed.taken_over);
    }

    #[test]
    fn a_completion_ref_refuses_the_claim_with_its_own_message() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);
        let commit = t.commit_all("seed");
        take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();

        let (done, _) = complete(&t.0, &task.id, "claude-code@ank").unwrap();
        assert_eq!(done.commit, commit);
        assert_eq!(done.branch.as_deref(), Some("main"));

        // The move to completed neither deletes the ref nor carries a TTL.
        let held = read(&t.0, &task.id).unwrap().unwrap();
        match &held.record {
            Record::Completed(c) => assert_eq!(c.branch.as_deref(), Some("main")),
            other => panic!("expected a completion, got {other:?}"),
        }
        let text = serialize_record(&held.record);
        assert!(!text.contains("expires"), "a completion has no TTL: {text}");

        let err = take(&t, &task, "codex@host-9", DEFAULT_TTL).unwrap_err();
        assert_eq!(err.code, 4);
        assert!(
            err.message.contains("finished on another branch"),
            "{}",
            err.message
        );
        assert!(err.message.contains(&commit[..7]), "{}", err.message);
        assert!(err.message.contains("branch main"), "{}", err.message);

        // Distinct from the other code 4, which is the whole point of having
        // two: one says take something else, the other says this is done.
        let other = open_task("00000000ffff");
        t.seed(&other);
        take(&t, &other, "codex@host-9", DEFAULT_TTL).unwrap();
        let held_err = take(&t, &other, "claude-code@ank", DEFAULT_TTL).unwrap_err();
        assert_eq!(held_err.code, 4);
        assert_ne!(held_err.message, err.message);
        assert!(!held_err.message.contains("finished on another branch"));
    }

    #[test]
    fn release_deletes_the_ref_and_says_whether_there_was_one() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);

        assert!(!delete(&t.0, &task.id).unwrap(), "nothing to delete yet");
        take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert!(delete(&t.0, &task.id).unwrap());
        assert!(read(&t.0, &task.id).unwrap().is_none());
        assert!(git::ank_refs(&t.0).unwrap().is_empty());

        // And the task is free again.
        take(&t, &task, "codex@host-9", DEFAULT_TTL).unwrap();
    }

    // -----------------------------------------------------------------------
    // Pruning
    // -----------------------------------------------------------------------

    // The two tests that stood here exercised the second copy of the pruning
    // predicate, and went with it (TASK-4981a1370c0b). Every case they covered
    // is asserted against the predicate that actually runs, in `human`'s tests:
    // `the_tree_saying_done_is_not_the_branch_saying_done` for the branch
    // against the working tree, and
    // `closed_prunes_like_done_and_the_rest_is_left_alone` for `closed`, for an
    // open task's live claim, and for a task the default branch has never
    // carried. The one below stays: it is about `claim`, not about pruning.

    #[test]
    fn claiming_never_prunes() {
        // A reader does not sanitise the coordination plane underneath
        // everyone else: `check` prunes, and nothing here does.
        let t = Temp::new_repo();
        let mut prunable = open_task("000000000001");
        let fresh = open_task("00000000ffff");
        t.seed(&prunable);
        t.seed(&fresh);
        t.commit_all("seed");
        take(&t, &prunable, "codex@host-9", DEFAULT_TTL).unwrap();
        complete(&t.0, &prunable.id, "codex@host-9").unwrap();
        prunable.status = TaskStatus::Done;
        t.seed(&prunable);
        t.commit_all("done");

        take(&t, &fresh, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert!(
            read(&t.0, &prunable.id).unwrap().is_some(),
            "a claim elsewhere must leave a prunable ref exactly where it was"
        );
        assert_eq!(git::ank_refs(&t.0).unwrap().len(), 2);
    }

    // -----------------------------------------------------------------------
    // Preconditions, hashes, TTL
    // -----------------------------------------------------------------------

    #[test]
    fn a_missing_criterion_refuses_with_7_and_the_command_that_sets_it() {
        let t = Temp::new_repo();
        let mut task = open_task("000000000001");
        task.done_criteria = None;
        task.criteria_by = None;
        t.seed(&task);

        let inv = invocation(&["claim", &task.id.to_string()]);
        let err = run_verb(&t, &inv).unwrap_err();
        assert_eq!(err.code, 7);
        assert!(err.message.contains("no done_criteria"), "{}", err.message);
        let hint = err.hint.unwrap();
        assert!(hint.contains("--criteria"), "{hint}");
        assert!(
            read(&t.0, &task.id).unwrap().is_none(),
            "a refusal must take no ref"
        );

        // With the criterion supplied, the same call goes through and records
        // who set it.
        let inv = invocation(&[
            "claim",
            &task.id.to_string(),
            "--criteria",
            "The binary exits 0.",
        ]);
        assert_eq!(run_verb(&t, &inv).unwrap(), 0);
        let text = t.task_text(&task.id);
        assert!(text.contains("criteria_by: claimer"), "{text}");
        assert!(text.contains("The binary exits 0."), "{text}");
    }

    #[test]
    fn a_blocker_that_is_not_done_refuses_with_7_and_names_it() {
        let t = Temp::new_repo();
        let blocker = open_task("00000000ffff");
        let mut blocked = open_task("000000000001");
        blocked.blocked_by = vec![blocker.id.clone()];
        t.seed(&blocker);
        t.seed(&blocked);

        let inv = invocation(&["claim", &blocked.id.to_string()]);
        let err = run_verb(&t, &inv).unwrap_err();
        assert_eq!(err.code, 7);
        assert!(
            err.message.contains(&blocker.id.to_string()),
            "{}",
            err.message
        );

        // `closed` does not unblock either: the work was not carried out.
        let mut closed = blocker.clone();
        closed.status = TaskStatus::Closed;
        t.seed(&closed);
        let err = run_verb(&t, &inv).unwrap_err();
        assert_eq!(err.code, 7);
        assert!(err.message.contains("closed"), "{}", err.message);

        // Done unblocks.
        let mut done = blocker.clone();
        done.status = TaskStatus::Done;
        t.seed(&done);
        assert_eq!(run_verb(&t, &inv).unwrap(), 0);
    }

    #[test]
    fn the_claim_anchors_the_criterion_and_the_applicable_constraints() {
        let t = Temp::new_repo();
        let mut task = open_task("000000000001");
        task.scope = vec!["crates/ank-cli/src/claim.rs".into()];
        t.seed(&task);
        let store = t.store();

        let put_adr = |e: &Entity| {
            std::fs::write(
                t.0.join(".ank/adr").join(format!("{}.md", e.id())),
                serialize_entity(e),
            )
            .unwrap()
        };
        // Wider than the task, narrower than the task, out of perimeter, and
        // in perimeter but only proposed.
        put_adr(&adr(
            "aaaaaaaaaaaa",
            &["crates/ank-cli/**"],
            "Wider.",
            AdrStatus::Accepted,
        ));
        put_adr(&adr(
            "bbbbbbbbbbbb",
            &["crates/ank-cli/src/claim.rs"],
            "Exact.",
            AdrStatus::Accepted,
        ));
        put_adr(&adr(
            "cccccccccccc",
            &["docs/**"],
            "Elsewhere.",
            AdrStatus::Accepted,
        ));
        put_adr(&adr(
            "dddddddddddd",
            &["crates/ank-cli/**"],
            "Not yet binding.",
            AdrStatus::Proposed,
        ));

        let applicable = applicable_constraints(&store, &t.repo(), &task).unwrap();
        let ids: Vec<&str> = applicable.iter().map(|(i, _)| i.as_str()).collect();
        assert_eq!(
            ids,
            vec!["ADR-aaaaaaaaaaaa", "ADR-bbbbbbbbbbbb"],
            "{applicable:?}"
        );

        let before = constraints_hash(&applicable);
        // A constraint accepted mid-work moves the hash, which is what lets
        // `done` warn.
        put_adr(&adr(
            "cccccccccccc",
            &["crates/ank-cli/**"],
            "Newly binding.",
            AdrStatus::Accepted,
        ));
        let after = constraints_hash(&applicable_constraints(&store, &t.repo(), &task).unwrap());
        assert_ne!(before, after);

        // Editing noise does not move it; a change of meaning does.
        let noisy = vec![("ADR-a".to_string(), "Text.  \n\n".to_string())];
        let clean = vec![("ADR-a".to_string(), "Text.".to_string())];
        assert_eq!(constraints_hash(&noisy), constraints_hash(&clean));

        // And the record really carries both hashes.
        let inv = invocation(&["claim", &task.id.to_string()]);
        run_verb(&t, &inv).unwrap();
        match read(&t.0, &task.id).unwrap().unwrap().record {
            Record::Claim(c) => {
                assert_eq!(c.criteria, freeze_hash_short("A verifiable criterion.\n"));
                assert_eq!(c.constraints, after);
            }
            other => panic!("expected a claim, got {other:?}"),
        }
    }

    #[test]
    fn the_ttl_is_capped_by_the_configured_ceiling() {
        let cfg = test_config();
        assert_eq!(resolve_ttl(None, &cfg).unwrap(), DEFAULT_TTL);
        assert_eq!(
            resolve_ttl(Some("10m"), &cfg).unwrap(),
            Duration::from_secs(600)
        );
        // An agent cannot grant itself twenty-four hours and hoard.
        assert_eq!(resolve_ttl(Some("24h"), &cfg).unwrap(), cfg.claim_ttl_max);
        assert!(resolve_ttl(Some("30"), &cfg).unwrap_err().hint.is_some());
    }

    #[test]
    fn the_refusal_points_at_another_ready_task_in_the_same_scope() {
        let t = Temp::new_repo();
        let wanted = open_task("000000000001");
        let ready = open_task("00000000ffff");
        t.seed(&wanted);
        t.seed(&ready);
        take(&t, &wanted, "codex@host-9", DEFAULT_TTL).unwrap();

        let inv = invocation(&["claim", &wanted.id.to_string()]);
        let err = run_verb(&t, &inv).unwrap_err();
        assert_eq!(err.code, 4);
        let hint = err.hint.unwrap();
        assert!(hint.contains(&ready.id.to_string()), "{hint}");
        assert!(hint.contains("another ready task"), "{hint}");
    }

    // -----------------------------------------------------------------------
    // Timestamps
    // -----------------------------------------------------------------------

    #[test]
    fn utc_timestamps_round_trip_including_across_a_leap_day() {
        for (secs, text) in [
            (0i64, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_782_000_000, "2026-06-21T00:00:00Z"),
            (1_782_018_123, "2026-06-21T05:02:03Z"),
            (4_102_444_800, "2100-01-01T00:00:00Z"),
        ] {
            assert_eq!(format_utc(secs), text);
            assert_eq!(parse_utc(text), Some(secs), "{text}");
        }
        // The shorter form the log uses reads back too.
        assert_eq!(
            parse_utc("2026-07-31T02:47Z"),
            parse_utc("2026-07-31T02:47:00Z")
        );
        for bad in [
            "",
            "2026-07-31",
            "2026-07-31T02:47",
            "2026-13-01T00:00:00Z",
            "x",
        ] {
            assert_eq!(parse_utc(bad), None, "{bad}");
        }
        assert_eq!(parse_utc(&now_utc()).is_some(), true);
    }

    // -----------------------------------------------------------------------
    // Small helpers for the verb-level tests
    // -----------------------------------------------------------------------

    fn invocation(argv: &[&str]) -> Invocation {
        crate::cli::parse(&argv.iter().map(|s| s.to_string()).collect::<Vec<_>>()).unwrap()
    }

    fn test_config() -> Config {
        config::parse(&config::default_yaml(), Path::new("config.yml")).unwrap()
    }

    fn run_verb(t: &Temp, inv: &Invocation) -> Result<i32> {
        let repo = Repo {
            root: t.0.clone(),
            ank: t.0.join(".ank"),
        };
        let mut out = Vec::new();
        run(inv, &repo, &test_config(), "claude-code@ank", &mut out)
    }
}
