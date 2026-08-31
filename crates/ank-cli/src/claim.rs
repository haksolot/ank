//! Claims on git refs: pickup, TTL, re-acquisition, completion refs (§7).
//!
//! A claim never touches a task file. It lives in `refs/ank/claims/<id>`, one
//! ref per task (ADR-4e7c25b1f639): writing it into the file would produce a
//! git diff on every pickup, which is exactly the noise separating the two
//! planes exists to avoid, and a single global ref would put two agents taking
//! two different tasks in contention over one address.
//!
//! **The ref has two states at one address**, and it is the record that says
//! which — never the address (ADR-6d8736c04cfa). `claim` writes a `claim`
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
use crate::index::Index;
use crate::repo::Repo;
use crate::store::Store;
use ank_contract::ExitCode;
use ank_core::{
    freeze, freeze_hash_short, Adr, AdrStatus, CriteriaBy, Entity, EntityId, Proof, ScopeSet, Task,
    TaskStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use std::path::{Path, PathBuf};
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

/// Default TTL (§3). Short on purpose: it is renewed implicitly by the holder's
/// work, so working is enough to keep the lock.
///
/// The tool's value, and what a repository states for itself is
/// `claim_ttl_default` (ADR-0bb7ea8991bc). This is still the number a record
/// written before the lease was recorded reads as, which is why it stays a
/// constant rather than becoming a configuration lookup: an absent field is
/// read as the tool's default everywhere else in the format too.
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
    CliError::new(
        ExitCode::Environment,
        format!("unreadable record on {name}: {detail}"),
    )
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
    live_claims_where(cwd, Some(except), now, &|holder| holder == identity)
}

/// The same walk, with the identity test handed in.
///
/// One enumeration and not two, because the two questions asked of it are
/// mirror images — what this identity also holds, and what everybody else
/// holds — and a second copy of "a lapsed claim is not a live one" is a second
/// chance to read expiry differently. That predicate is the whole reason
/// ADR-052accd6e3b2's signal does not fire on abandoned work forever.
///
/// **`except` is an option and not a task**, because the two callers differ on
/// whether there is anything to exclude. Reading this corpus, the task being
/// claimed is the one claim that must not be reported back to its own claimer.
/// Reading a corpus the reader declared, there is nothing to exclude at all:
/// ids are minted without coordination and two corpora do not collide, so an id
/// that matched over there would be a different task, and dropping it would
/// hide a real claim behind a coincidence.
fn live_claims_where(
    cwd: &Path,
    except: Option<&EntityId>,
    now: i64,
    keep: &dyn Fn(&str) -> bool,
) -> Result<Vec<(EntityId, ClaimRecord)>> {
    let mut held = Vec::new();
    // The enumeration and the records in one reading (TASK-5690eae1e008): this
    // walk asked `cat-file` once per claim ref, and a corpus carrying five
    // hundred of them paid five hundred processes to answer a question about
    // one identity.
    let (refs, records) = git::ank_records(cwd)?;
    for r in refs {
        let Some(rest) = r.name.strip_prefix(CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        if except == Some(&id) {
            continue;
        }
        let Some(text) = records.get(&r.object) else {
            continue;
        };
        let Ok(Record::Claim(c)) = parse_record(text, &r.name) else {
            continue;
        };
        if !keep(&c.holder) || is_expired(&c, now, &id).unwrap_or(true) {
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
/// Refusing on state and never on identity (ADR-91b77f036884): what is read is
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
        ExitCode::Prerequisite,
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
/// is in the plumbing ADR-9307e5d214a7 allows; what is lost here is the debug
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
        .map_err(|e| CliError::new(ExitCode::Environment, format!("git hash-object: {e}")))?;
    child
        .stdin
        .take()
        .ok_or_else(|| CliError::new(ExitCode::Environment, "git hash-object: no stdin"))?
        .write_all(text.as_bytes())
        .map_err(|e| CliError::new(ExitCode::Environment, format!("git hash-object: {e}")))?;
    let out = child
        .wait_with_output()
        .map_err(|e| CliError::new(ExitCode::Environment, format!("git hash-object: {e}")))?;
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
    /// A remote exists and could not be reached. The write stands locally, and
    /// what the caller does with that depends on whether anything survives the
    /// failed push that is worth having (ADR-af533e7a3e03): a claim degrades,
    /// warns and exits 0 — §2, with the risk of a concurrent claim displayed
    /// rather than hidden — while a verb whose whole product is the ref fails.
    /// The value is the same either way; the two sentences below are what
    /// separate the readings.
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
    ///
    /// **It is a failure and not a warning, and the name says so**
    /// (ADR-af533e7a3e03). The two sentences sit side by side because the two
    /// risks are different, and the difference goes further than the wording:
    /// a claim that did not travel still governs this clone, so its verb
    /// degrades, while a detached proof's whole product is the ref, so there is
    /// no degraded mode left to fall back to and `attest --detached` exits 9.
    /// A method named `warning` returning the text of an error would be a
    /// comment that lies in the one place a reader checks first.
    /// The same sentence for a claim being given back, and it is a third
    /// sentence because it is a third risk. [`Sync::warning`] reports an
    /// acquisition that did not travel, where the danger is that the task is
    /// taken twice; this reports a *revocation* that did not travel, where the
    /// danger is the mirror image — the claim is gone here and reads as live
    /// everywhere else, until the lease on the stale ref runs out. Which is
    /// why it does not name the local status: `release` leaves the task open
    /// and `close` leaves it closed, and one sentence serves both.
    ///
    /// `release` and `close` are both on the degrading side of
    /// ADR-af533e7a3e03: the hand-back is written to disk and stands whatever
    /// the remote did, so the verb exits 0. The constraint says degrades,
    /// *warns* and exits zero, and this is the warning half.
    pub fn revocation_warning(&self) -> Option<String> {
        match self {
            Sync::Local | Sync::Pushed => None,
            Sync::Unsynchronised(_) => Some(
                "claim deletion not pushed: the claim is gone in this clone only, and \
                 another clone still reads the task as held until the claim expires"
                    .to_string(),
            ),
        }
    }

    pub fn proof_failure(&self) -> Option<String> {
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
/// `done` replaces the record, it does not delete (ADR-6d8736c04cfa). Returns
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
pub fn delete(cwd: &Path, id: &EntityId) -> Result<Deleted> {
    delete_at(cwd, &claim_ref(id))
}

/// The same deletion, addressed by ref. `check` prunes a proof ref through it
/// once the file on the default branch carries the same attestation.
pub fn delete_at(cwd: &Path, name: &str) -> Result<Deleted> {
    let Some(witness) = current_object(cwd, name)? else {
        return Ok(Deleted {
            existed: false,
            sync: Sync::Local,
        });
    };
    let args = ["update-ref", "-d", name];
    let out = git::output(cwd, &args)?;
    if !out.status.success() {
        return Err(git::failed(&args, &out));
    }
    // **The push result is carried out, not swallowed.** It used to be dropped
    // here, and the two verbs that delete a claim therefore had nothing to warn
    // from: they degraded and exited 0 in complete silence, which is two of the
    // three things ADR-af533e7a3e03 asks of a degrading verb. A failure to
    // reach the remote is still not a failure of the verb — the hand-back is
    // already on disk — so it leaves as a value the caller reports rather than
    // as an error.
    let sync = match push(cwd, name, None, Some(&witness)) {
        Ok(written) => written.sync,
        // The push helper itself could not run git at all. The deletion took
        // locally, so this is the same degradation with a coarser reason.
        Err(e) => Sync::Unsynchronised(e.to_string()),
    };
    Ok(Deleted {
        existed: true,
        sync,
    })
}

/// What a deletion of a coordination ref did: whether there was one to delete,
/// and how far the deletion reached.
///
/// Two facts and not one because the callers need both and for different
/// reasons: `close` reports `claim_revoked` from `existed`, and both `close`
/// and `release` owe a warning when `sync` says the remote never heard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deleted {
    pub existed: bool,
    pub sync: Sync,
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
    // One reading of the plane rather than a `rev-parse` and a `cat-file` per
    // ref (TASK-5690eae1e008). The record is the one the enumeration named, so
    // this asks the same question it always asked -- in two processes for the
    // whole namespace instead of two per claim in it.
    let (refs, records) = git::ank_records(cwd)?;
    for r in refs {
        let Some(rest) = r.name.strip_prefix(CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        // A ref the batch has nothing for is skipped, which is what the pair of
        // processes this replaces did: `rev-parse` answered `None` for a ref
        // released between the enumeration and the read, and that race is real
        // -- another agent handing back a task must not fail this one's `done`.
        let Some(text) = records.get(&r.object) else {
            continue;
        };
        let Record::Claim(c) = parse_record(text, &r.name)? else {
            continue;
        };
        if c.holder != identity {
            continue;
        }
        let lapsed = is_expired(&c, now_secs(), &id)?;
        let standing = Standing {
            id,
            object: r.object,
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
/// [`renew`] and not a variant of it: re-acquisition restores the lock, it does
/// not re-bless the work, so it wants exactly the write that carries the anchors
/// over and moves nothing but the expiry. The name survives because the caller's
/// situation is what differs — the lease ran out — and not the write.
pub fn retake(cwd: &Path, standing: &Standing, cap: Duration) -> Result<Written> {
    renew(cwd, &standing.id, &standing.object, &standing.record, cap)
}

/// One renewal of a claim: the expiry recomputed from the lease the record
/// carries, re-capped by `claim_ttl_max`, swapped on the object the record was
/// read from (§3, §7).
///
/// **The one implementation, and that is the whole point of it.** The renewal
/// `log` performs, the re-acquisition of a lapsed claim and the renewal every
/// other verb of the holder performs (ADR-0bb7ea8991bc) are the same write on
/// the same terms; the first two were two copies of these four lines, and
/// TASK-1b45f41e7b99 is what one of them getting the lease wrong costs — a
/// second reading of the expiry is a second chance to read it differently.
///
/// **The anchors are carried over, never recomputed.** Re-freezing the criterion
/// here would erase a divergence introduced while the claim stood, which is the
/// one thing `done` checks the hash to catch.
///
/// The applied lease is written back rather than carried over. On a record from
/// before the field existed that turns a `0` into the default it was just read
/// as, so the record describes itself from the first renewal and the unset value
/// leaves the coordination plane instead of being copied forward for the life of
/// the claim.
pub fn renew(
    cwd: &Path,
    id: &EntityId,
    object: &str,
    record: &ClaimRecord,
    cap: Duration,
) -> Result<Written> {
    let ttl = renewal_ttl(record, cap);
    let refreshed = Record::Claim(ClaimRecord {
        expires: format_utc(now_secs() + ttl.as_secs() as i64),
        ttl: ttl.as_secs(),
        ..record.clone()
    });
    put(cwd, id, &refreshed, Some(object))
}

// ---------------------------------------------------------------------------
// Renewal by working (§3, ADR-0bb7ea8991bc)
// ---------------------------------------------------------------------------

// What a verb is about as far as the lease is concerned is a property of the
// verb, so it is declared with the verb table in `ank-contract` and read here
// (ADR-6fd69efb629c). Re-exported because `crate::claim::Renews` is where the
// rule's own module puts it in every reader's hands, and because the act of
// renewing — the only thing that touches a ref — stays in this file.
pub use ank_contract::Renews;

/// Renews the caller's lease when the verb that just ran was work on the task it
/// holds (§3, ADR-0bb7ea8991bc).
///
/// **Silent, errors included**, which is why the caller gets no `Result`.
/// Renewal is a side effect of working and never the answer to the question
/// asked: `show` and `edit` do not coordinate and must keep answering outside a
/// usable git (ADR-9307e5d214a7), and a verb that failed to renew has still done
/// what it was called for. `log` reports because reporting is that verb's job.
///
/// **A lapsed claim is not renewed.** Taking one back stays the re-acquisition
/// `log` and `done` perform (§3) — extending it from a read would let a passive
/// verb silently retake a claim §3 hands to the two verbs that write.
pub fn renew_by_working(
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    renews: Renews,
    named: Option<&str>,
) {
    let _ = renewed_by_working(repo, cfg, identity, renews, named);
}

/// The body of [`renew_by_working`], with the errors it swallows still visible
/// to a reader and to a test.
fn renewed_by_working(
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    renews: Renews,
    named: Option<&str>,
) -> Result<bool> {
    if renews == Renews::Never {
        return Ok(false);
    }
    let Some(standing) = on_task(&repo.corpus, identity)? else {
        return Ok(false);
    };
    if standing.lapsed {
        return Ok(false);
    }
    if renews == Renews::Named {
        // No id given is no task named, and this verb names one: `ank show`
        // with nothing to show never reached a task, so it worked on none.
        let Some(given) = named else {
            return Ok(false);
        };
        // Resolved through the store, because the caller types a prefix and the
        // held id is whole. A prefix that resolves to nothing, or to two
        // entities, is a verb that already failed; it renews nothing either way.
        if Store::new(&repo.ank).resolve(given).ok().as_ref() != Some(&standing.id) {
            return Ok(false);
        }
    }
    renew(
        &repo.corpus,
        &standing.id,
        &standing.object,
        &standing.record,
        cfg.claim_ttl_max,
    )?;
    Ok(true)
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
/// (ADR-91b77f036884).
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

/// Whether two scopes meet at all.
///
/// Shared with `context`, which asks it of a spec and the task in hand: what
/// decides that a constraint bears on a task has to be what decides that a
/// specification governs it, or one page would name the two for different
/// perimeters.
pub(crate) fn scopes_intersect(a: &[String], b: &[String]) -> Result<bool> {
    let invalid =
        |e: ank_core::Error| CliError::new(ExitCode::Generic, format!("invalid scope: {e}"));
    let set_a = ScopeSet::new(a).map_err(invalid)?;
    let set_b = ScopeSet::new(b).map_err(invalid)?;
    Ok(a.iter().any(|g| set_b.overlaps_dir(g, b)) || b.iter().any(|g| set_a.overlaps_dir(g, a)))
}

// ---------------------------------------------------------------------------
// What two scopes have in common (ADR-052accd6e3b2)
// ---------------------------------------------------------------------------
//
// A different question from `scopes_intersect`, which answers whether a
// constraint bears on a task and needs nothing but a yes. Here the answer is
// what a reader acts on: `crates/ank-cli/**` against `crates/ank-cli/tests/**`
// overlaps on everything under the second, and a line saying only "these
// overlap" leaves the reader exactly where they started.
//
// **Nothing is expanded.** An intersection of globs is not an intersection of
// sets: two globs overlap when some path *could* match both, and the set of
// paths that do changes the moment a file is added. Where both sides are
// literal the answer is a path; where one is a pattern the honest answer is the
// narrower pattern, written as a glob.
//
// Coarse by construction, and ADR-052accd6e3b2 argues that is why it is a
// signal and never a refusal. A false positive costs one line; making it
// precise first would cost the change.

/// The characters that make a glob a pattern rather than a path.
const WILDCARDS: [char; 4] = ['*', '?', '[', '{'];

fn is_pattern(glob: &str) -> bool {
    glob.contains(WILDCARDS)
}

/// The deepest directory a glob cannot widen past: everything before the last
/// `/` preceding the first wildcard. `crates/ank-cli/**` gives
/// `crates/ank-cli`, `docs/*.md` gives `docs`, and `**/*.rs` gives the root,
/// which is empty and contains everything.
fn literal_dir(glob: &str) -> &str {
    let head = match glob.find(WILDCARDS) {
        Some(i) => &glob[..i],
        None => glob,
    };
    match head.rfind('/') {
        Some(i) => &glob[..i],
        None => "",
    }
}

/// Whether `child` names a place inside `parent`, compared on segment
/// boundaries: `crates/ank-cli` is not inside `crates/ank`.
fn under(child: &str, parent: &str) -> bool {
    parent.is_empty() || child == parent || child.starts_with(&format!("{parent}/"))
}

/// Of two patterns that meet, the one a reader should look at: the deeper
/// literal directory first, then the one carrying more literal text. The final
/// tiebreak is lexical so that the same pair always yields the same answer,
/// whichever order it is asked in.
fn narrower<'a>(a: &'a str, b: &'a str) -> &'a str {
    let key = |g: &str| {
        (
            literal_dir(g).len(),
            g.chars().filter(|c| !WILDCARDS.contains(c)).count(),
        )
    };
    match key(a).cmp(&key(b)) {
        std::cmp::Ordering::Less => b,
        std::cmp::Ordering::Greater => a,
        std::cmp::Ordering::Equal => std::cmp::min(a, b),
    }
}

/// A literal path against a pattern. Two ways they meet, and they give
/// different answers: the pattern matches the path, and then the path is what
/// they have in common; or the pattern lives under the path, and then the
/// pattern is.
fn path_against_pattern(path: &str, pattern: &str) -> Option<String> {
    if ScopeSet::new(&[pattern.to_string()]).is_ok_and(|s| s.matches(path)) {
        return Some(path.to_string());
    }
    under(literal_dir(pattern), path).then(|| pattern.to_string())
}

/// What two globs have in common, or `None` when no path could match both.
///
/// An invalid glob answers `None` rather than raising: this feeds a signal that
/// accompanies a write, and a scope nobody can compile is `check`'s finding to
/// report, not a reason to refuse a claim.
pub fn glob_overlap(a: &str, b: &str) -> Option<String> {
    let a = a.trim_end_matches('/');
    let b = b.trim_end_matches('/');
    if a == b {
        return Some(a.to_string());
    }
    match (is_pattern(a), is_pattern(b)) {
        // Two paths meet on the deeper of them, which is the one both cover, or
        // they do not meet at all.
        (false, false) => {
            if under(b, a) {
                Some(b.to_string())
            } else if under(a, b) {
                Some(a.to_string())
            } else {
                None
            }
        }
        (false, true) => path_against_pattern(a, b),
        (true, false) => path_against_pattern(b, a),
        // Two patterns: neither can be expanded honestly, so the answer is the
        // narrower, and they meet exactly when one's literal directory sits
        // inside the other's.
        (true, true) => {
            let (da, db) = (literal_dir(a), literal_dir(b));
            match (under(db, da), under(da, db)) {
                (true, true) => Some(narrower(a, b).to_string()),
                (true, false) => Some(b.to_string()),
                (false, true) => Some(a.to_string()),
                (false, false) => None,
            }
        }
    }
}

/// Everything two scopes have in common, deduplicated and ordered.
///
/// Empty means they do not meet, which is what `find --free` filters on and
/// what keeps `claim` silent in the ordinary case.
pub fn scope_overlap(a: &[String], b: &[String]) -> Vec<String> {
    let mut common: Vec<String> = Vec::new();
    for ga in a {
        for gb in b {
            if let Some(c) = glob_overlap(ga, gb) {
                if !common.contains(&c) {
                    common.push(c);
                }
            }
        }
    }
    common.sort();
    common
}

/// A live claim held by somebody else over ground this task also covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeClash {
    pub id: EntityId,
    pub holder: String,
    /// Never empty: a clash with nothing in common is not one.
    pub common: Vec<String>,
}

impl ScopeClash {
    /// The one line `claim` prints, naming the holder, the task and the ground
    /// (ADR-052accd6e3b2). Written here rather than at the call site because
    /// `find --free` counts what this describes, and a filter and a warning
    /// that disagree about what an overlap is would be worse than either alone.
    pub fn line(&self) -> String {
        format!(
            "{} holds {}, overlapping on {}",
            self.holder,
            self.id,
            self.common.join(" ")
        )
    }
}

/// Every live claim held by another identity whose scope meets `task`'s.
///
/// **The scope comes from the corpus, never from the ref.** A claim record
/// carries who and until when and nothing else (ADR-4e7c25b1f639), so the other
/// task is loaded to be asked what ground it covers. A claimed task this
/// checkout does not carry is skipped in silence: it is a branch that has not
/// arrived, which `check` reports and a warning cannot.
pub fn scope_clashes(
    cwd: &Path,
    store: &Store,
    task: &Task,
    identity: &str,
    now: i64,
) -> Result<Vec<ScopeClash>> {
    let mut clashes = Vec::new();
    for (id, record) in live_claims_where(cwd, Some(&task.id), now, &|holder| holder != identity)? {
        let Ok(loaded) = store.load(&id) else {
            continue;
        };
        let Entity::Task(other) = loaded.entity else {
            continue;
        };
        let common = scope_overlap(&task.scope, &other.scope);
        if common.is_empty() {
            continue;
        }
        clashes.push(ScopeClash {
            id,
            holder: record.holder,
            common,
        });
    }
    Ok(clashes)
}

// ---------------------------------------------------------------------------
// A live claim this identity holds in another corpus the reader declared
// (ADR-ed3e14d0f991)
// ---------------------------------------------------------------------------
//
// **The rule stays per corpus, and this is the fact that replaces the refusal
// nobody can arbitrate.** `refs/ank/*` is per repository (ADR-4e7c25b1f639,
// ADR-a1de673043b4), so a cross-corpus refusal could only be enforced by
// whatever process happened to see both corpora at once, and two such processes
// would not see each other. So nothing below refuses: it names, exactly as
// ADR-052accd6e3b2 names two claims whose scopes intersect, and the claim is
// taken all the same.
//
// **Only what the reader already declared is ever read or named**
// (ADR-621a7fd96ce1, ADR-96174f1ac2b7). The map is handed in; nothing here
// walks a filesystem looking for a corpus, reads a git remote, or derives a
// location from a path or a slug. A caller who has declared nothing hands in an
// empty map, no corpus is opened, no line is added, and what they see is what
// they saw before this existed.
//
// **Reading crosses, writing never does.** What is opened over there is
// `refs/ank/claims/*` and nothing else: no store, no index, no ref written, no
// file touched. The lock ADR-a1de673043b4 left standing is that claims are not
// *arbitrated* across a boundary, and naming one is not arbitrating it.

/// A live claim this identity holds in another corpus the reader declared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElsewhereClaim {
    pub task: EntityId,
    /// The corpus, as the reader declared it and as [`rendered`] spells it.
    /// The location and not the repository identity: the identity keys the map
    /// on the *tree* a reader stands in (ADR-96174f1ac2b7), which is not what
    /// this claim is in, and a path is what the reader can act on.
    pub corpus: String,
    pub expires: String,
}

impl ElsewhereClaim {
    /// The one line `claim` prints, naming the task, the corpus and the lease.
    ///
    /// The identity is passed in rather than carried, because it is the same
    /// for every row: the question this answers is what *this* caller already
    /// holds, and repeating it on the struct would be a field that can only
    /// ever hold one value.
    pub fn line(&self, identity: &str) -> String {
        format!(
            "{identity} holds {} in {}, until {}",
            self.task, self.corpus, self.expires
        )
    }
}

/// Every live claim `identity` holds in a declared corpus that is not `here`,
/// and one warning per declared corpus that could not be read.
///
/// `declared` is [`crate::config::declarations`]'s map, handed in rather than
/// read here: a function that reads the reader's home is a function no test can
/// pin, and the golden that says a caller with no declaration sees today's
/// bytes is exactly the test that needs to hand in an empty one.
///
/// **The corpus in hand is skipped by its path, not by the map's key.** The key
/// is the identity of the tree a reader stands in and the value is where its
/// corpus lives, so two keys can name one corpus and the key cannot answer
/// "is this the one I am already in". [`crate::repo::same_corpus`] canonicalises
/// both sides, which is the same question `--repo` asks, and the canonical path
/// is also what deduplicates one corpus declared twice.
///
/// **Degrade, never fail** (§2). A declared corpus that is not there, is not a
/// corpus, or whose refs cannot be read costs one line and the claim is taken —
/// the rule [`crate::repo::peers_of`] already follows for a peer, and the same
/// reason: a claim does not fail because something the reader was told about is
/// missing. Once per corpus, because the map holds each one once and the
/// canonical path drops the rest.
pub fn claims_elsewhere(
    here: &Path,
    declared: &BTreeMap<String, String>,
    identity: &str,
    now: i64,
) -> (Vec<ElsewhereClaim>, Vec<String>) {
    let mut held = Vec::new();
    let mut warnings = Vec::new();
    let mut seen: Vec<PathBuf> = Vec::new();

    // Sorted by the declared location rather than left in the map's order,
    // which is the order of a root commit's hex and means nothing to a reader.
    let mut locations: Vec<&String> = declared.values().collect();
    locations.sort();
    locations.dedup();

    for location in locations {
        let root = Path::new(location);
        if crate::repo::same_corpus(root, here) {
            continue;
        }
        match std::fs::canonicalize(root) {
            Ok(real) if seen.contains(&real) => continue,
            Ok(real) => seen.push(real),
            // Not a refusal and not a skip: `at` below is what says whether a
            // corpus is there, and it says it with the warning this branch
            // would otherwise pre-empt.
            Err(_) => {}
        }
        let Ok(repo) = crate::repo::at(root) else {
            warnings.push(unreadable(location, "is not a corpus"));
            continue;
        };
        match live_claims_where(&repo.corpus, None, now, &|holder| holder == identity) {
            Ok(claims) => held.extend(claims.into_iter().map(|(task, record)| ElsewhereClaim {
                task,
                corpus: rendered(location),
                expires: record.expires,
            })),
            Err(_) => warnings.push(unreadable(location, "could not be read")),
        }
    }
    (held, warnings)
}

/// A path as this corpus renders one: forward slashes, on every platform.
///
/// **One rendering, held whatever the platform.** Windows CI caught the two
/// halves of one sentence disagreeing: the corpus came out `C:/corpora/front`
/// because the reader wrote it that way in `corpora.yml` and the map is echoed
/// verbatim, and `corpora.yml`'s own location came out `C:\Users\...` because
/// `Path::display` renders with the platform's separator. Two spellings of one
/// kind of thing in one sentence is not cosmetic: a reader cannot grep for a
/// path spelled one way in one clause and the other way in the next, and an
/// agent keying on the corpus would read two spellings of one directory as two
/// corpora.
///
/// **Forward slashes and not the platform's**, because that is already what
/// this corpus does everywhere a path reaches a reader: `normalize_path`
/// unifies a scope before it is matched, and `context`, `done` and `human` each
/// unify a path before printing it. A message is read on a machine other than
/// the one that wrote it, and the corpus it names is the same corpus either
/// way.
fn rendered(path: &str) -> String {
    path.replace('\\', "/")
}

/// The one sentence a declared corpus that did not answer costs, ending with
/// where the declaration that named it lives.
///
/// Both paths go through [`rendered`], and that is the point rather than a
/// detail: this sentence is where the two spellings met.
fn unreadable(location: &str, why: &str) -> String {
    let map = crate::config::corpora_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| crate::config::CORPORA_FILE.to_string());
    format!(
        "the corpus declared at {} {why}, and what it holds is not named \
         (correct the entry in {})",
        rendered(location),
        rendered(&map)
    )
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
            ExitCode::Unavailable,
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
        ExitCode::Unavailable,
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
        ExitCode::Unavailable,
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
        ExitCode::Prerequisite,
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
        None => CliError::new(
            ExitCode::Unavailable,
            format!("{id} was taken and released while claiming"),
        )
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
/// of the recorded commit (ADR-6d8736c04cfa): `done` writes to the working
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
) -> Result<ExitCode> {
    // **The reader's declarations, read once and handed down**
    // (ADR-96174f1ac2b7). Silent on every error, for the reason
    // `repo::is_declared` gives: resolution has already refused on a map it
    // could not parse by the time any verb runs, so a second reading has
    // nothing new to report and no business failing a claim over it.
    let declared = config::declarations().unwrap_or_default();
    run_with(inv, repo, cfg, identity, &declared, out)
}

/// The verb with the reader's declarations handed in.
///
/// Split from [`run`] so the map is an argument and not an ambient file: the
/// hard boundary of ADR-ed3e14d0f991 is that a caller who declared nothing sees
/// the bytes they saw before, and a test that cannot say "declared nothing"
/// cannot assert it.
fn run_with(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    declared: &BTreeMap<String, String>,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let prefix = inv.positionals.first().ok_or_else(|| {
        CliError::new(ExitCode::Generic, "claim expects a task id").with_hint("ank claim <id>")
    })?;
    let store = Store::new(&repo.ank);
    let loaded = store.load_prefix(prefix)?;
    let base_version = crate::store::version_of(&loaded.entity);
    // What a `--criteria` that writes one would replace, kept before the
    // destructuring consumes it (ADR-f7dc76886db2).
    let before = loaded.entity.clone();
    let Entity::Task(mut task) = loaded.entity else {
        return Err(
            CliError::new(ExitCode::Generic, format!("{prefix} is not a task"))
                .with_hint(format!("ank show {prefix}")),
        );
    };

    let ttl = resolve_ttl(inv.value("--ttl"), cfg)?;
    let ready = other_ready_task(&repo.corpus, &store, &task).map(|id| id.to_string());

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
                    ExitCode::Transition,
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
            return Err(CliError::new(
                ExitCode::Prerequisite,
                format!("{} has no done_criteria", task.id),
            )
            .with_hint(format!(
                "ank claim {} --criteria \"<verifiable criterion>\"",
                task.id
            )))
        }
    };
    // The remote's view first, so a task held in another clone is refused with
    // its holder named rather than taken here and unwound by a rejected push
    // (§7). Silent when there is no remote, and silent when there is one and it
    // cannot be reached — that is the write's news to break, not the read's.
    sync_from_remote(&repo.corpus, &task.id)?;

    check_blockers(&repo.corpus, &store, &task, ready.as_deref())?;
    task.status
        .check_transition(TaskStatus::InProgress)
        .map_err(|e| {
            CliError::new(ExitCode::Transition, e.to_string())
                .with_hint(format!("ank show {}", task.id))
        })?;

    // Last of the preconditions, and last on purpose. Everything above refuses
    // on the task asked for; this one refuses on what the caller already holds,
    // and an agent told to hand back its work for a task that would have
    // refused it anyway has paid for nothing.
    already_holding(&repo.corpus, identity, &task.id, now_secs())?;

    let constraints = constraints_hash(&applicable_constraints(&store, repo, &task)?);
    let acquired = acquire(
        &repo.corpus,
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
    let claimed = Entity::Task(task.clone());
    let version = store.write(&claimed, base_version)?;

    // **The criterion this call wrote, accounted for** (ADR-f7dc76886db2).
    // `claim` is on the list of three because `--criteria` writes a
    // `done_criteria` the task did not have, which is content, and the whole
    // authority model then rests on it. The status this same write moved is a
    // transition, which the claim ref already records and which earns no entry:
    // a plain `claim` writes none, and a test on `done` says the same of the
    // other direction.
    // Asked of the two states rather than remembered from the branch above:
    // what earns an entry is content that moved, and the field is where that
    // is legible.
    let wrote_criteria =
        matches!(&before, Entity::Task(t) if t.done_criteria != task.done_criteria);
    if wrote_criteria {
        crate::entries::record_edit(
            &store,
            &Index::open(&repo.ank)?,
            &claimed,
            identity,
            &now_utc(),
            &crate::entries::edit_message(
                &["done_criteria".to_string(), "criteria_by".to_string()],
                base_version,
                version,
                &crate::entries::replaced_hash(&before),
                &crate::entries::content_hash(&claimed),
            ),
        )?;
    }

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
    let also_held = live_claims_of(&repo.corpus, identity, &acquired.id, now_secs())?;
    // What somebody else is holding that covers the same files
    // (ADR-052accd6e3b2). Named and never refused: scope overlap is coarse
    // enough that refusing on it would turn a glob into a mutex, and would push
    // agents to declare narrower scopes than the truth to get past it. Computed
    // after the ref is taken because it is a signal about the work, not a
    // precondition of the claim -- the task is already held by the time this is
    // read, and the criterion says the exit code is 0.
    let clashes = scope_clashes(&repo.corpus, &store, &task, identity, now_secs())?;
    // What this identity already holds in a corpus the reader declared
    // (ADR-ed3e14d0f991). Named and never refused, for the reason the ADR
    // gives: a refusal across corpora could only be arbitrated by whatever
    // process saw both, and `refs/ank/*` is per repository. Read after the ref
    // is taken, with the clashes, because it is a fact about the caller and not
    // a precondition of the claim -- the exit code is the one a claim with
    // nothing to report gives.
    let (elsewhere, unread) = claims_elsewhere(&repo.corpus, declared, identity, now_secs());
    let warnings: Vec<String> = also_held
        .iter()
        .map(|(id, c)| format!("{identity} already holds {id} until {}", c.expires))
        .chain((!also_held.is_empty()).then(way_out))
        .chain(clashes.iter().map(ScopeClash::line))
        // The same line a human reads and a `--json` caller reads, exactly as
        // the two above: the structured field below is the shape, not a second
        // set of facts.
        .chain(elsewhere.iter().map(|e| e.line(identity)))
        .chain(unread)
        // A claim that did not reach the remote holds in this clone alone, and
        // §7 is explicit that the risk is displayed rather than hidden.
        .chain(acquired.sync.warning())
        .collect();

    if inv.json() {
        let doc = crate::json::Obj::document()
            .str("task", &acquired.id.to_string())
            .str("holder", &acquired.holder)
            .str("expires", &acquired.expires)
            .strings("warnings", &warnings);
        // **The key arrives with the facts and never without them.** A document
        // is free to gain a field within a contract version, and an empty array
        // on every claim would still be a byte a parser sees that it did not see
        // before -- which is the one thing ADR-ed3e14d0f991 says a caller with no
        // declaration must not pay. So `claim` declares two documents, the way
        // `log` does, and the golden for the caller who declared nothing is the
        // one that was already there.
        let doc = match elsewhere.is_empty() {
            true => doc,
            false => doc.array(
                "claims_elsewhere",
                elsewhere.iter().map(|e| {
                    crate::json::Obj::new()
                        .str("task", &e.task.to_string())
                        .str("corpus", &e.corpus)
                        .str("expires", &e.expires)
                        .finish()
                }),
            ),
        };
        let _ = writeln!(out, "{}", doc.finish());
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
    Ok(ExitCode::Ok)
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
///
/// **The default the flag falls back to is the repository's**,
/// `claim_ttl_default`, and the cap binds it exactly as it binds a value the
/// caller typed (ADR-0bb7ea8991bc). A repository whose default sits above its
/// own cap is not a configuration that fails to load — it is a claim granted
/// the cap, which is the same answer `--ttl 24h` gets.
fn resolve_ttl(flag: Option<&str>, cfg: &Config) -> Result<Duration> {
    let asked = match flag {
        Some(v) => config::parse_duration(v).map_err(|e| {
            CliError::new(ExitCode::Generic, e).with_hint("ank claim <id> --ttl 30m")
        })?,
        None => cfg.claim_ttl_default,
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
/// (ADR-6d8736c04cfa), so it is consulted here. The refusal itself does not
/// move — claiming on top of unmerged work is the real risk — only what the
/// agent is told about it, which is the answer `acquire` already gives for the
/// claimed task itself.
fn check_blockers(cwd: &Path, store: &Store, task: &Task, other_ready: Option<&str>) -> Result<()> {
    let map = status_map(store)?;
    let blockers = task
        .active_blockers(|id| map.get(id).copied())
        .map_err(|e| CliError::new(ExitCode::Prerequisite, e.to_string()).with_hint("ank check"))?;

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
        return Err(CliError::new(
            ExitCode::Prerequisite,
            format!("{} is blocked by {first}{why}", task.id),
        )
        .with_hint(format!("ank show {first}")));
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
///
/// A candidate whose ref carries a live claim is skipped for the same reason,
/// and it took a defect to notice that this is the same case one ref state over
/// (TASK-f601ba59229e). The file reads `open` here exactly as often — the
/// holder claimed on their own branch and the status this one carries has not
/// moved — and `ank claim <id>` on it refuses with code 4, `held by`. The ready
/// task the message promises is one whose ref is free (§7), not one whose file
/// reads `open` on this branch only because something has not landed yet.
///
/// A lapsed claim is not that case and is still offered: pickup after expiry is
/// a legal transition, `acquire` takes it, and skipping it would trade one
/// wrong answer for another. The predicate is therefore `is_expired`, the same
/// reading every other caller gets. A record whose expiry does not parse is
/// treated as live: `acquire` refuses on it too, so offering it would print the
/// refusing command all over again.
fn other_ready_task(cwd: &Path, store: &Store, task: &Task) -> Option<EntityId> {
    let map = status_map(store).ok()?;
    let now = now_secs();
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
        match read(cwd, id) {
            Ok(Some(Held {
                record: Record::Completed(_),
                ..
            })) => continue,
            Ok(Some(Held {
                record: Record::Claim(c),
                ..
            })) if !is_expired(&c, now, id).unwrap_or(false) => continue,
            _ => {}
        }
        return Some(id.clone());
    }
    None
}

/// Anchors the ADRs whose constraint applies here, so that a reader of this
/// module lands on them: claims in refs (ADR-4e7c25b1f639), the ref's second
/// state (ADR-6d8736c04cfa), git per verb and plumbing by criterion
/// (ADR-9307e5d214a7), freeze by hash (ADR-6b3f19e08a24), one surface
/// (ADR-91b77f036884).
#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{serialize_entity, Adr};
    use std::path::{Path, PathBuf};
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
            // Maintenance off, because a test here fingerprints the corpus and
            // git is otherwise free to repack it between two snapshots
            // (TASK-fc6bef21e268).
            t.porcelain(&["config", "gc.auto", "0"]);
            t.porcelain(&["config", "maintenance.auto", "false"]);
            t
        }

        /// Porcelain is forbidden to the tool (ADR-9307e5d214a7), not to the
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
                corpus: self.0.clone(),
                worktree: self.0.clone(),
                ank: self.0.join(".ank"),
            }
        }

        /// Through the store's own paths rather than a literal directory: the
        /// fixture must not be the one place that still knows the layout.
        fn seed(&self, task: &Task) {
            let e = Entity::Task(task.clone());
            let path = self.store().path_of(&task.id);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, serialize_entity(&e)).unwrap();
        }

        fn task_text(&self, id: &EntityId) -> String {
            std::fs::read_to_string(self.store().read_path_of(id)).unwrap()
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// Every git repository inside a fixture, found rather than listed.
    ///
    /// A directory holding a `HEAD` file and an `objects` directory is one,
    /// whether it is the `.git` beside a working tree or a bare corpus. Found,
    /// because a list would have to be maintained, and the thing being guarded
    /// against is exactly a repository nobody remembered to enrol.
    fn repositories_under(root: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
                found.push(dir);
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    stack.push(e.path());
                }
            }
        }
        found
    }

    /// What git itself answers for one key of one repository, `None` when the
    /// key is unset -- which is the state this asserts against, since an unset
    /// `maintenance.auto` means maintenance is on.
    fn config_of(git_dir: &Path, key: &str) -> Option<String> {
        let out = Command::new("git")
            .arg("--git-dir")
            .arg(git_dir)
            .args(["config", "--get", key])
            .output()
            .expect("git must be installed: it is a hard dependency");
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Asserts every repository under `root` is one git will not maintain.
    ///
    /// Read back out of the fixture, never grepped out of this file: the
    /// subject is what `git init` plus the configuration actually produced.
    fn assert_unmaintained(root: &Path) {
        let repos = repositories_under(root);
        assert!(
            !repos.is_empty(),
            "no repository found under {}: this asserts nothing",
            root.display()
        );
        for git_dir in repos {
            let at = git_dir.display();
            assert_eq!(
                config_of(&git_dir, "gc.auto").as_deref(),
                Some("0"),
                "gc.auto at {at}"
            );
            assert_eq!(
                config_of(&git_dir, "maintenance.auto").as_deref(),
                Some("false"),
                "maintenance.auto at {at}"
            );
        }
    }

    /// A fixture repository is not maintained under the test.
    ///
    /// `naming_a_claim_elsewhere_writes_nothing_in_the_corpus_it_names`
    /// fingerprints a foreign corpus before and after a read and asserts the
    /// bytes did not move. It failed on ubuntu-latest in run 33284185681 and
    /// passed on the other two platforms: the first snapshot carried
    /// `objects/maintenance.lock`, a `tmp_pack` and six loose objects, the
    /// second a multi-pack-index, two packs and `info/refs`. Git had repacked
    /// the repository between the two; ank had written nothing. The assertion
    /// was right and its subject was moving.
    ///
    /// Every fixture in this module is built by `Temp::new_repo`, so a fifth
    /// one added below is held to this without anyone remembering to enrol it.
    #[test]
    fn a_fixture_repository_is_not_maintained_under_the_test() {
        let t = Temp::new_repo();
        assert_unmaintained(&t.0);
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
        // that ADR-9307e5d214a7 allows.
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
        assert_eq!(err.code, ExitCode::Environment);
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
            assert_eq!(err.code, ExitCode::Environment, "{text}");
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
        assert_eq!(err.code, ExitCode::Environment);
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
        assert_eq!(run_verb(&t, &inv).unwrap(), ExitCode::Ok);

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
            assert_eq!(
                err.code,
                ExitCode::Unavailable,
                "the losers exit with 4: {}",
                err.render()
            );
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
        assert_eq!(err.code, ExitCode::Unavailable);
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
        assert_eq!(err.code, ExitCode::Unavailable);
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
        assert_eq!(held_err.code, ExitCode::Unavailable);
        assert_ne!(held_err.message, err.message);
        assert!(!held_err.message.contains("finished on another branch"));
    }

    #[test]
    fn release_deletes_the_ref_and_says_whether_there_was_one() {
        let t = Temp::new_repo();
        let task = open_task("000000000001");
        t.seed(&task);

        assert!(
            !delete(&t.0, &task.id).unwrap().existed,
            "nothing to delete yet"
        );
        take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert!(delete(&t.0, &task.id).unwrap().existed);
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
        assert_eq!(err.code, ExitCode::Prerequisite);
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
        assert_eq!(run_verb(&t, &inv).unwrap(), ExitCode::Ok);
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
        assert_eq!(err.code, ExitCode::Prerequisite);
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
        assert_eq!(err.code, ExitCode::Prerequisite);
        assert!(err.message.contains("closed"), "{}", err.message);

        // Done unblocks.
        let mut done = blocker.clone();
        done.status = TaskStatus::Done;
        t.seed(&done);
        assert_eq!(run_verb(&t, &inv).unwrap(), ExitCode::Ok);
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

    /// The ground two globs share, named rather than merely detected
    /// (ADR-052accd6e3b2).
    ///
    /// The pairs are the corpus's own shapes, and the answer that matters is
    /// the second column: a signal saying "these overlap" leaves the reader
    /// where they started, and one saying `crates/ank-cli/tests/**` tells them
    /// which file to expect a conflict in. Nothing is expanded into a file
    /// list, because the list would be wrong the moment a file is added.
    #[test]
    fn two_globs_are_answered_with_the_ground_they_share() {
        let common = |a: &str, b: &str| glob_overlap(a, b);
        for (a, b, expected) in [
            // The pair that produced two merge conflicts in one session.
            (
                "crates/ank-cli/**",
                "crates/ank-cli/tests/**",
                Some("crates/ank-cli/tests/**"),
            ),
            // A pattern against a file it matches: the file is the answer, and
            // it is a path because there is a path to give.
            (
                "crates/ank-cli/**",
                "crates/ank-cli/tests/cli.rs",
                Some("crates/ank-cli/tests/cli.rs"),
            ),
            // A pattern living under a directory named literally.
            ("crates", "crates/ank-cli/**", Some("crates/ank-cli/**")),
            // Same prefix, two patterns: the narrower one carries more literal
            // text, and the answer does not depend on the order asked.
            ("docs/**", "docs/*.md", Some("docs/*.md")),
            // Disjoint, and neither a prefix of the other on a segment
            // boundary -- `crates/ank-core` is not inside `crates/ank`.
            ("crates/ank-cli/**", "docs/**", None),
            ("crates/ank/**", "crates/ank-core/**", None),
            (
                "crates/ank-cli/src/claim.rs",
                "crates/ank-cli/src/human.rs",
                None,
            ),
            // A trailing separator is not a different perimeter.
            ("docs/", "docs", Some("docs")),
        ] {
            assert_eq!(common(a, b).as_deref(), expected, "{a} against {b}");
            assert_eq!(
                common(b, a).as_deref(),
                expected,
                "{b} against {a} answered differently from the other order"
            );
        }

        // A scope is a list, and what two lists share is every pair that meets,
        // deduplicated: the same file reached through two globs is one answer.
        let mine = vec!["crates/ank-cli/src/claim.rs".to_string(), "docs/**".into()];
        let theirs = vec![
            "crates/ank-cli/**".to_string(),
            "docs/getting-started.md".into(),
        ];
        assert_eq!(
            scope_overlap(&mine, &theirs),
            vec![
                "crates/ank-cli/src/claim.rs".to_string(),
                "docs/getting-started.md".to_string()
            ]
        );
        assert!(scope_overlap(&mine, &["skill/SKILL.md".to_string()]).is_empty());
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
        assert_eq!(err.code, ExitCode::Unavailable);
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

    fn run_verb(t: &Temp, inv: &Invocation) -> Result<ExitCode> {
        let mut out = Vec::new();
        run_declared(t, inv, &BTreeMap::new(), &mut out)
    }

    /// The verb with the reader's declarations handed in, and its bytes kept.
    ///
    /// Never [`run`], which would read whichever `corpora.yml` the machine
    /// running the suite happens to carry: what a test asserts about a caller
    /// with no declaration has to be a caller with no declaration, and an
    /// ambient file is the one thing that could make that sentence false on
    /// somebody else's laptop.
    fn run_declared(
        t: &Temp,
        inv: &Invocation,
        declared: &BTreeMap<String, String>,
        out: &mut Vec<u8>,
    ) -> Result<ExitCode> {
        run_with(
            inv,
            &t.repo(),
            &test_config(),
            "claude-code@ank",
            declared,
            out,
        )
    }

    /// A second corpus, in a repository of its own, holding one task.
    ///
    /// Its own repository because that is where a corpus's claims live
    /// (ADR-9e56318631f3): a directory with a `.ank/` and no git is a corpus
    /// whose refs cannot be read, which is the *other* test.
    fn second_corpus(hex: &str) -> (Temp, Task) {
        let t = Temp::new_repo();
        let task = open_task(hex);
        t.seed(&task);
        t.commit_all("seed");
        (t, task)
    }

    fn declaring(entries: &[(&str, &Path)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(key, path)| (key.to_string(), path.display().to_string()))
            .collect()
    }

    /// Forty hex characters, which is the only key `declarations` lets through.
    fn key(n: u8) -> String {
        format!("{:040x}", n)
    }

    // -----------------------------------------------------------------------
    // One live claim per identity is per corpus, and the rest are named
    // (ADR-ed3e14d0f991)
    // -----------------------------------------------------------------------

    /// The whole of the decision in one call: the claim is taken, the exit code
    /// is the one a claim with nothing to report gives, and the claim held in
    /// the other declared corpus is named with the corpus it is in.
    #[test]
    fn a_claim_held_in_another_declared_corpus_is_named_and_never_refused() {
        let here = Temp::new_repo();
        let mine = open_task("00000000ca01");
        here.seed(&mine);
        here.commit_all("seed");

        let (there, theirs) = second_corpus("00000000ca02");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();

        let declared = declaring(&[(&key(1), &there.0)]);
        let inv = invocation(&["claim", &mine.id.to_string()]);
        let mut out = Vec::new();
        assert_eq!(
            run_declared(&here, &inv, &declared, &mut out).unwrap(),
            ExitCode::Ok,
            "the code is the one a claim with nothing to report gives"
        );

        // Taken all the same, and on the ref rather than only on paper.
        assert!(matches!(
            read(&here.0, &mine.id).unwrap().unwrap().record,
            Record::Claim(_)
        ));

        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains(&format!("claude-code@ank holds {}", theirs.id)),
            "the claim held elsewhere is named: {text}"
        );
        assert!(
            text.contains(&format!("in {}", named(&there.0))),
            "with the corpus it is in: {text}"
        );
        assert!(text.starts_with("warning:"), "as a warning: {text}");
    }

    /// The same facts, under a field of its own.
    #[test]
    fn json_carries_the_claims_elsewhere_under_a_field_of_its_own() {
        let here = Temp::new_repo();
        let mine = open_task("00000000cb01");
        here.seed(&mine);
        here.commit_all("seed");

        let (there, theirs) = second_corpus("00000000cb02");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();

        let declared = declaring(&[(&key(1), &there.0)]);
        let inv = invocation(&["claim", &mine.id.to_string(), "--json"]);
        let mut out = Vec::new();
        assert_eq!(
            run_declared(&here, &inv, &declared, &mut out).unwrap(),
            ExitCode::Ok
        );

        let text = String::from_utf8(out).unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&text).unwrap();
        let rows = doc["claims_elsewhere"].as_sequence().expect(&text);
        assert_eq!(rows.len(), 1, "{text}");
        assert_eq!(
            rows[0]["task"].as_str(),
            Some(theirs.id.to_string().as_str())
        );
        assert_eq!(rows[0]["corpus"].as_str(), Some(named(&there.0).as_str()));
        assert!(rows[0]["expires"].as_str().is_some(), "{text}");
        // One document, one line, and the shape `ank-contract` declares for it.
        assert_eq!(text.lines().count(), 1, "{text}");
    }

    /// The hard boundary, pinned against a golden: with nothing declared, no
    /// corpus is read, none is named, and the bytes are the ones that were
    /// there before any of this existed — on both surfaces.
    #[test]
    fn a_caller_with_no_declaration_sees_the_bytes_it_saw_before() {
        // A corpus that would be named the moment anything went looking for
        // one, holding a live claim under the very identity that is claiming
        // here. Nothing declares it, so nothing may find it.
        let (there, theirs) = second_corpus("00000000cc02");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();

        let here = Temp::new_repo();
        let mine = open_task("00000000cc01");
        here.seed(&mine);
        here.commit_all("seed");

        let mut out = Vec::new();
        let inv = invocation(&["claim", "TASK-00000000cc01"]);
        assert_eq!(
            run_declared(&here, &inv, &BTreeMap::new(), &mut out).unwrap(),
            ExitCode::Ok
        );
        assert_eq!(
            String::from_utf8(out).unwrap(),
            "claimed TASK-00000000cc01 example -> HEAD\n",
            "a caller with no declaration sees the line it always saw"
        );

        // And the machine surface, whose golden in `tests/golden-json/` carries
        // exactly these keys in exactly this order.
        let here = Temp::new_repo();
        let mine = open_task("00000000cc01");
        here.seed(&mine);
        here.commit_all("seed");
        let mut out = Vec::new();
        let inv = invocation(&["claim", "TASK-00000000cc01", "--json"]);
        run_declared(&here, &inv, &BTreeMap::new(), &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        let expires = expires_in(&text);
        assert_eq!(
            text,
            format!(
                "{{\"contract\":1,\"task\":\"TASK-00000000cc01\",\
                 \"holder\":\"claude-code@ank\",\"expires\":\"{expires}\",\
                 \"warnings\":[]}}\n"
            ),
            "no key arrives that a caller with no declaration did not already read"
        );
    }

    /// The instant the document carries, which is the one value in it this test
    /// cannot know in advance. Read back out rather than pinned, the way
    /// `tests/golden-json/` redacts it.
    fn expires_in(doc: &str) -> String {
        let v: serde_yaml::Value = serde_yaml::from_str(doc).unwrap();
        v["expires"].as_str().unwrap().to_string()
    }

    /// A corpus nobody declared is never read and never named, asserted from
    /// the reading side rather than through the whole verb.
    #[test]
    fn only_a_declared_corpus_is_ever_read_or_named() {
        let (there, theirs) = second_corpus("00000000cd02");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();
        let here = Temp::new_repo();

        let (held, warnings) =
            claims_elsewhere(&here.0, &BTreeMap::new(), "claude-code@ank", now_secs());
        assert!(held.is_empty(), "nothing declared, nothing found: {held:?}");
        assert!(warnings.is_empty(), "{warnings:?}");

        // Declared, and now it answers: the difference between the two calls is
        // the map and nothing else.
        let (held, warnings) = claims_elsewhere(
            &here.0,
            &declaring(&[(&key(1), &there.0)]),
            "claude-code@ank",
            now_secs(),
        );
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(held.len(), 1, "{held:?}");
        assert_eq!(held[0].task, theirs.id);
    }

    /// The corpus in hand is not reported back to itself, however the reader
    /// declared it, and a corpus declared twice is named once.
    #[test]
    fn the_corpus_being_claimed_in_is_never_among_the_ones_named() {
        let here = Temp::new_repo();
        let a = open_task("00000000ce01");
        let b = open_task("00000000ce02");
        here.seed(&a);
        here.seed(&b);
        here.commit_all("seed");
        acquire(&here.0, &b, "claude-code@ank", DEFAULT_TTL, "h", "c", None).unwrap();

        // Declared under its own path and under a second key, which is what two
        // trees sharing one corpus look like in the map.
        let declared = declaring(&[(&key(1), &here.0), (&key(2), &here.0)]);
        let (held, warnings) = claims_elsewhere(&here.0, &declared, "claude-code@ank", now_secs());
        assert!(
            held.is_empty(),
            "the corpus in hand is `also_held`'s business, not this one: {held:?}"
        );
        assert!(warnings.is_empty(), "{warnings:?}");

        // And one corpus declared twice under two keys is opened once.
        let (there, theirs) = second_corpus("00000000ce03");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();
        let declared = declaring(&[(&key(1), &there.0), (&key(2), &there.0)]);
        let (held, _) = claims_elsewhere(&here.0, &declared, "claude-code@ank", now_secs());
        assert_eq!(held.len(), 1, "named once, not once per key: {held:?}");
    }

    /// Somebody else's claim over there is not this caller's, and a lapsed one
    /// is nobody's.
    #[test]
    fn only_this_identity_and_only_a_live_claim_is_named() {
        let here = Temp::new_repo();
        let (there, theirs) = second_corpus("00000000cf02");
        acquire(
            &there.0,
            &theirs,
            "codex@host-9",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();
        let declared = declaring(&[(&key(1), &there.0)]);

        let (held, _) = claims_elsewhere(&here.0, &declared, "claude-code@ank", now_secs());
        assert!(held.is_empty(), "another identity holds it: {held:?}");

        let (held, _) = claims_elsewhere(&here.0, &declared, "codex@host-9", now_secs());
        assert_eq!(held.len(), 1, "{held:?}");

        // Past the lease and past the drift tolerance it is nobody's, which is
        // the predicate `live_claims_where` already applies here.
        let later =
            now_secs() + DEFAULT_TTL.as_secs() as i64 + DRIFT_TOLERANCE.as_secs() as i64 + 1;
        let (held, _) = claims_elsewhere(&here.0, &declared, "codex@host-9", later);
        assert!(held.is_empty(), "a lapsed claim is not held: {held:?}");
    }

    /// A declared corpus that cannot be read costs one line and never the
    /// claim.
    #[test]
    fn a_declared_corpus_that_cannot_be_read_warns_once_and_the_claim_is_taken() {
        let here = Temp::new_repo();
        let mine = open_task("00000000c001");
        here.seed(&mine);
        here.commit_all("seed");

        // Three ways to be unreadable: a path that is not there, a directory
        // that is no corpus, and a corpus whose repository is not one, so its
        // refs cannot be read.
        let absent = here.0.join("no-such-corpus");
        let bare = Temp::new_repo();
        std::fs::remove_dir_all(bare.0.join(".ank")).unwrap();
        let detached = Temp::new_repo();
        std::fs::remove_dir_all(detached.0.join(".git")).unwrap();

        let declared = declaring(&[
            (&key(1), &absent),
            (&key(2), &bare.0),
            (&key(3), &detached.0),
        ]);
        let (held, warnings) = claims_elsewhere(&here.0, &declared, "claude-code@ank", now_secs());
        assert!(held.is_empty(), "{held:?}");
        assert_eq!(warnings.len(), 3, "one line per corpus, once: {warnings:?}");
        // Each named once, in the order the reader would read them off the
        // map's values rather than in the order this test wrote them.
        for location in [&absent, &bare.0, &detached.0] {
            let named: Vec<&String> = warnings
                .iter()
                .filter(|w| w.contains(&named(location)))
                .collect();
            assert_eq!(named.len(), 1, "{location:?} named once: {warnings:?}");
            assert!(
                named[0].contains("corpora.yml"),
                "and where to correct it: {}",
                named[0]
            );
        }

        // And the claim is taken, with the same code as one with nothing to
        // report.
        let inv = invocation(&["claim", &mine.id.to_string()]);
        let mut out = Vec::new();
        assert_eq!(
            run_declared(&here, &inv, &declared, &mut out).unwrap(),
            ExitCode::Ok
        );
        assert!(matches!(
            read(&here.0, &mine.id).unwrap().unwrap().record,
            Record::Claim(_)
        ));
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("claimed"), "{text}");
    }

    /// Reading crosses and writing never does: the declared corpus is left
    /// exactly as it was found.
    #[test]
    fn naming_a_claim_elsewhere_writes_nothing_in_the_corpus_it_names() {
        let here = Temp::new_repo();
        let mine = open_task("00000000c101");
        here.seed(&mine);
        here.commit_all("seed");

        let (there, theirs) = second_corpus("00000000c102");
        acquire(
            &there.0,
            &theirs,
            "claude-code@ank",
            DEFAULT_TTL,
            "h",
            "c",
            None,
        )
        .unwrap();
        let before = fingerprint(&there.0);

        let inv = invocation(&["claim", &mine.id.to_string()]);
        let mut out = Vec::new();
        run_declared(&here, &inv, &declaring(&[(&key(1), &there.0)]), &mut out).unwrap();

        assert_eq!(
            before,
            fingerprint(&there.0),
            "a corpus that was read must be byte for byte what it was"
        );
    }

    /// A fixture path spelled the way `claim` spells one.
    ///
    /// Not `Path::display`, which renders with the platform's separator: the
    /// binary holds one rendering on every platform, so a test comparing
    /// against the platform's would assert the opposite of the rule on the one
    /// platform where the two differ.
    fn named(path: &Path) -> String {
        rendered(&path.display().to_string())
    }

    /// One rendering, held whatever the platform, in both halves of the
    /// sentence that carries two paths.
    ///
    /// Driven with a Windows path on every platform rather than left to a
    /// Windows runner to discover, because that is the only way the assertion
    /// means anything on the two platforms where `Path::display` already
    /// agrees with it. Windows CI is what found the defect; this is what stops
    /// it coming back between runs there.
    #[test]
    fn one_rendering_is_held_for_every_path_a_line_carries() {
        let claim = ElsewhereClaim {
            task: EntityId::parse("TASK-000000000001").unwrap(),
            corpus: rendered(r"C:\corpora\front"),
            expires: "2026-08-25T18:41:12Z".into(),
        };
        let line = claim.line("claude-code@ank");
        assert!(line.contains("C:/corpora/front"), "{line}");
        assert!(
            !line.contains('\\'),
            "a path is spelled one way, whatever wrote it: {line}"
        );

        // The sentence where the two spellings met: the location the reader
        // typed, and the map's own location, which `Path::display` renders.
        let w = unreadable(r"C:\corpora\front", "is not a corpus");
        assert!(w.contains("C:/corpora/front"), "{w}");
        assert!(
            !w.contains('\\'),
            "both halves of the sentence, not one of them: {w}"
        );
    }

    /// Every path under `root`, with what each file holds. Enough to catch a
    /// ref written, an index created, or an entity moved.
    fn fingerprint(root: &Path) -> Vec<(PathBuf, u64)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, u64)>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for e in entries.flatten() {
                let p = e.path();
                match p.is_dir() {
                    true => walk(&p, out),
                    false => out.push((
                        p.clone(),
                        std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
                    )),
                }
            }
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out.sort();
        out
    }
}
