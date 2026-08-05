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
use crate::repo::Repo;
use crate::store::Store;
use ank_core::{
    freeze, freeze_hash_short, Adr, AdrStatus, CriteriaBy, Entity, EntityId, ScopeSet, Task,
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

/// Default TTL (§3). Short on purpose: it is renewed implicitly by `log`, so
/// working is enough to keep the lock.
pub const DEFAULT_TTL: Duration = Duration::from_secs(30 * 60);

/// Clock-drift tolerance on expiry (§7). At the scale of a 30-minute TTL, NTP
/// is more than enough, and two minutes cost far less than a claim wrongly
/// stolen from a machine whose clock runs fast.
pub const DRIFT_TOLERANCE: Duration = Duration::from_secs(2 * 60);

pub fn ref_name(id: &EntityId) -> String {
    format!("{CLAIMS_PREFIX}{id}")
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

/// A claim in force: who holds the task and until when, plus the two hashes
/// that anchor what was frozen at pickup (§7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClaimRecord {
    pub task: String,
    pub holder: String,
    pub claimed: String,
    pub expires: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Record {
    Claim(ClaimRecord),
    Completed(CompletedRecord),
}

impl Record {
    pub fn task(&self) -> &str {
        match self {
            Record::Claim(c) => &c.task,
            Record::Completed(c) => &c.task,
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

fn corrupt(id: &EntityId, detail: impl std::fmt::Display) -> CliError {
    // Code 9: a coordination ref nobody can read is an environment to repair,
    // not a failure of the agent's work (§4). Never a silent fallback to the
    // other state — that is what would let a completion read as a free task.
    CliError::new(
        9,
        format!("unreadable claim record on {}: {detail}", ref_name(id)),
    )
    .with_hint(format!("git update-ref -d {}", ref_name(id)))
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
pub fn parse_record(text: &str, id: &EntityId) -> Result<Record> {
    use serde_yaml::Value;
    let value: Value = serde_yaml::from_str(text).map_err(|e| corrupt(id, e))?;
    let Value::Mapping(mut map) = value else {
        return Err(corrupt(id, "not a YAML mapping"));
    };
    let state = map
        .remove(Value::from("state"))
        .ok_or_else(|| corrupt(id, "no state field"))?;
    let state = state
        .as_str()
        .ok_or_else(|| corrupt(id, "state is not a string"))?
        .to_string();
    let rest = Value::Mapping(map);
    match state.as_str() {
        STATE_CLAIM => Ok(Record::Claim(
            serde_yaml::from_value(rest).map_err(|e| corrupt(id, e))?,
        )),
        STATE_COMPLETED => Ok(Record::Completed(
            serde_yaml::from_value(rest).map_err(|e| corrupt(id, e))?,
        )),
        other => Err(corrupt(id, format!("unknown state '{other}'"))),
    }
}

// ---------------------------------------------------------------------------
// Ref operations. The compare-and-swap is git's.
// ---------------------------------------------------------------------------

/// The record a task's ref carries, if it carries one. An absent ref is
/// `None`, never an error: that is the nominal state of a free task.
pub fn read(cwd: &Path, id: &EntityId) -> Result<Option<Held>> {
    let name = ref_name(id);
    let args = ["rev-parse", "--verify", "--quiet", name.as_str()];
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
        record: parse_record(&text, id)?,
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
fn update(cwd: &Path, id: &EntityId, new: &str, old: Option<&str>) -> Result<Cas> {
    let name = ref_name(id);
    let args = ["update-ref", name.as_str(), new, old.unwrap_or("")];
    let out = git::output(cwd, &args)?;
    if out.status.success() {
        return Ok(Cas::Won);
    }
    match current_object(cwd, id)? {
        // The ref is where we left it: nothing moved, so the refusal did not
        // come from contention. Reporting a lost race here would send an agent
        // to take another task when the real problem is the environment.
        Some(o) if Some(o.as_str()) == old => Err(git::failed(&args, &out)),
        None if old.is_none() => Err(git::failed(&args, &out)),
        _ => Ok(Cas::Lost),
    }
}

fn current_object(cwd: &Path, id: &EntityId) -> Result<Option<String>> {
    let name = ref_name(id);
    let out = git::output(cwd, &["rev-parse", "--verify", "--quiet", name.as_str()])?;
    if !out.status.success() {
        return Ok(None);
    }
    let o = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if o.is_empty() { None } else { Some(o) })
}

/// Puts `record` on the ref. `witness` is the object the caller read: `None`
/// requires the ref to be absent, `Some` requires it to be exactly there.
pub fn put(cwd: &Path, id: &EntityId, record: &Record, witness: Option<&str>) -> Result<Cas> {
    let blob = write_blob(cwd, record)?;
    update(cwd, id, &blob, witness)
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
pub fn delete(cwd: &Path, id: &EntityId) -> Result<bool> {
    let name = ref_name(id);
    if current_object(cwd, id)?.is_none() {
        return Ok(false);
    }
    let args = ["update-ref", "-d", name.as_str()];
    let out = git::output(cwd, &args)?;
    if !out.status.success() {
        return Err(git::failed(&args, &out));
    }
    Ok(true)
}

// ---------------------------------------------------------------------------
// Expiry
// ---------------------------------------------------------------------------

/// Whether a claim has lapsed, judged on the timestamp the record carries plus
/// the drift tolerance (§7). A record whose timestamp does not read back is
/// corrupt, and saying "expired" about it would be a silent fallback.
pub fn is_expired(claim: &ClaimRecord, now: i64, id: &EntityId) -> Result<bool> {
    let expires = parse_utc(&claim.expires)
        .ok_or_else(|| corrupt(id, format!("unreadable expiry '{}'", claim.expires)))?;
    Ok(now > expires + DRIFT_TOLERANCE.as_secs() as i64)
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
        criteria: criteria_hash.to_string(),
        constraints: constraints_hash.to_string(),
    });

    match put(cwd, id, &record, witness)? {
        Cas::Won => {}
        // Somebody landed between our read and our write. Re-read to name
        // them: the winner is the one whose message the agent needs.
        Cas::Lost => return Err(lost_the_race(cwd, id, now, other_ready)?),
    }

    let Record::Claim(written) = record else {
        unreachable!("just built as a claim")
    };
    Ok(Acquired {
        id: id.clone(),
        holder: written.holder,
        expires: written.expires,
        taken_over: witness.is_some(),
    })
}

/// Replaces whatever the ref carries with a completion record, keeping the
/// address. Called by `done` (TASK-e5f6a7b8c9d0); no TTL is written, because
/// what ends the completion ref is durable state catching up, not time.
pub fn complete(cwd: &Path, id: &EntityId, identity: &str) -> Result<CompletedRecord> {
    let commit = git::run(cwd, &["rev-parse", "HEAD"])?;
    let branch = git::current_branch(cwd)?;
    let witness = current_object(cwd, id)?;
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
        Cas::Won => Ok(record),
        Cas::Lost => Err(
            CliError::new(4, format!("{id} moved while it was being completed"))
                .with_hint(format!("ank claim {id}")),
        ),
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
pub fn prune(cwd: &Path, tasks_prefix: &str, default_branch: &str) -> Result<Vec<EntityId>> {
    let mut pruned = Vec::new();
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(CLAIMS_PREFIX) else {
            // Another ank namespace, or an orphan ref whose name is not an
            // identifier. Not ours to judge here.
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        let path = format!("{}/{id}.md", tasks_prefix.trim_end_matches('/'));
        let Some(text) = git::file_at(cwd, default_branch, &path)? else {
            // Absent from the default branch: this is precisely the unmerged
            // case the ref exists for.
            continue;
        };
        // A file that does not parse settles nothing, so it prunes nothing;
        // reporting it is `check`'s job, not maintenance's.
        let Ok(Entity::Task(t)) = ank_core::parse_entity(&text) else {
            continue;
        };
        if matches!(t.status, TaskStatus::Done | TaskStatus::Closed) {
            delete(cwd, &id)?;
            pruned.push(id);
        }
    }
    Ok(pruned)
}

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
    if let Some(c) = inv.value("--criteria") {
        if !c.trim().is_empty() {
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
    check_blockers(&repo.root, &store, &task, ready.as_deref())?;
    task.status
        .check_transition(TaskStatus::InProgress)
        .map_err(|e| CliError::new(6, e.to_string()).with_hint(format!("ank show {}", task.id)))?;

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

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{}\",\"holder\":\"{}\",\"expires\":\"{}\"}}",
            acquired.id, acquired.holder, acquired.expires
        );
    } else if !inv.quiet() {
        let slug = task.slug.as_deref().unwrap_or("");
        let _ = writeln!(out, "claimed {} {slug} -> HEAD", acquired.id);
    }
    Ok(0)
}

fn ensure_trailing_newline(text: &str) -> String {
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
    use ank_core::{serialize_entity, Adr, Proof};
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
            self.porcelain(&["-c", "commit.gpgsign=false", "commit", "-qm", message]);
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
            schema: 1,
            version: 1,
            body: "\nFree body.\n".into(),
        }
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
            criteria: "aaaabbbbcccc".into(),
            constraints: "ddddeeeeffff".into(),
        });
        assert_eq!(put(&t.0, &id, &record, None).unwrap(), Cas::Won);

        let refs = git::ank_refs(&t.0).unwrap();
        assert_eq!(refs.len(), 1, "{refs:?}");
        assert_eq!(refs[0].name, ref_name(&id));
        assert_eq!(read(&t.0, &id).unwrap().unwrap().record, record);
    }

    // -----------------------------------------------------------------------
    // The record and its two states
    // -----------------------------------------------------------------------

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
            criteria: "123456789012".into(),
            constraints: "000000000000".into(),
        });
        let text = serialize_record(&record);
        assert_eq!(parse_record(&text, &id).unwrap(), record, "{text}");

        let done = Record::Completed(CompletedRecord {
            task: id.to_string(),
            commit: "1234567890123456789012345678901234567890".into(),
            branch: Some("main".into()),
            identity: "claude-code@ank".into(),
            completed: "2026-07-31T02:30:00Z".into(),
        });
        let text = serialize_record(&done);
        assert_eq!(parse_record(&text, &id).unwrap(), done, "{text}");
    }

    #[test]
    fn an_unknown_state_is_named_never_read_as_the_other_one() {
        let id = EntityId::parse("TASK-000000000001").unwrap();

        let err = parse_record("state: abandoned\ntask: TASK-000000000001\n", &id).unwrap_err();
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
            let err = parse_record(text, &id).unwrap_err();
            assert_eq!(err.code, 9, "{text}");
            assert!(err.message.contains("unreadable claim record"), "{text}");
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
        git::run(&t.0, &["update-ref", &ref_name(&id), &blob]).unwrap();

        let err = read(&t.0, &id).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains(&ref_name(&id)), "{}", err.message);
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

        let done = complete(&t.0, &task.id, "claude-code@ank").unwrap();
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

    #[test]
    fn pruning_follows_the_default_branch_and_never_the_working_tree() {
        let t = Temp::new_repo();
        let mut task = open_task("000000000001");
        t.seed(&task);
        t.commit_all("seed");
        take(&t, &task, "claude-code@ank", DEFAULT_TTL).unwrap();
        complete(&t.0, &task.id, "claude-code@ank").unwrap();

        // The working tree says done; the branch does not yet. This is the
        // unmerged case, and pruning here would reopen the window.
        task.status = TaskStatus::Done;
        task.proof = vec![Proof {
            proof_type: ank_core::ProofType::Commit,
            reference: "0123456".into(),
            tree: None,
            criteria: None,
            verifier: None,
        }];
        t.seed(&task);
        assert_eq!(
            prune(&t.0, ".ank/tasks", "main").unwrap(),
            vec![],
            "the tree is not the branch"
        );
        assert!(read(&t.0, &task.id).unwrap().is_some());

        // Once the branch carries it, the ref has no further use.
        t.commit_all("done");
        assert_eq!(
            prune(&t.0, ".ank/tasks", "main").unwrap(),
            vec![task.id.clone()]
        );
        assert!(read(&t.0, &task.id).unwrap().is_none());
    }

    #[test]
    fn pruning_covers_closed_and_leaves_everything_else_alone() {
        let t = Temp::new_repo();
        let mut closed = open_task("000000000001");
        let live = open_task("00000000ffff");
        t.seed(&closed);
        t.seed(&live);
        t.commit_all("seed");

        take(&t, &closed, "claude-code@ank", DEFAULT_TTL).unwrap();
        take(&t, &live, "codex@host-9", DEFAULT_TTL).unwrap();

        closed.status = TaskStatus::Closed;
        t.seed(&closed);
        t.commit_all("closed");

        assert_eq!(prune(&t.0, ".ank/tasks", "main").unwrap(), vec![closed.id]);
        assert!(
            read(&t.0, &live.id).unwrap().is_some(),
            "an open task's claim is not maintenance's business"
        );

        // A task absent from the default branch is exactly the unmerged case.
        let unmerged = open_task("00000000aaaa");
        t.seed(&unmerged);
        take(&t, &unmerged, "claude-code@ank", DEFAULT_TTL).unwrap();
        assert_eq!(prune(&t.0, ".ank/tasks", "main").unwrap(), vec![]);
        assert!(read(&t.0, &unmerged.id).unwrap().is_some());
    }

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
