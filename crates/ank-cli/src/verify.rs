//! Execution of named verifiers through `sh -c` (§4).
//!
//! **Ank runs the verification itself.** That is the whole point: if the agent
//! ran the tests and then reported the result through `--proof`, nothing would
//! be anchored — it could simply claim it passes. The task declares its
//! verifiers, `done` runs them, and the agent never self-reports.
//!
//! A verifier is **named**, declared in `config.yml`, never an inline command.
//! A task can arrive through a pull request from a fork, and an inline command
//! would be arbitrary code execution triggered by `ank done`. Git has exactly
//! this problem with hooks and solved it by never running them on clone; here
//! the verifiers live in a file the repository controls, so changing one goes
//! through review like any other change.
//!
//! **A broken environment is not a task failure.** `sh` missing, a command that
//! does not exist, a process that will not spawn: all code 9 with the exact
//! command to run. Code 5 stays reserved for a verifier that ran and said no —
//! confusing the two sends an agent to fix code that was never wrong.
//!
//! Dispatch routes no verifier by name: `done` calls this module for the
//! task's list, and `verify` for the verifiers a change touches
//! (ADR-c621cb9c6739).

use crate::cli::CliError;
use crate::config::Verifier;
use ank_contract::ExitCode;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub type Result<T> = std::result::Result<T, CliError>;

const GIT_FOR_WINDOWS: &str = "https://git-scm.com/download/win";
const POSIX_SH: &str = "install a POSIX sh, or Git for Windows which ships one";

/// What a verifier did. `ok` is the only thing that decides the transition;
/// the rest is what gets anchored in the proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub name: String,
    pub ok: bool,
    pub code: Option<i32>,
    pub elapsed: Duration,
    /// Hash of what the verifier printed, stdout and stderr together. The
    /// proof anchors this rather than the text: a proof entry is a one-line
    /// diff, and megabytes of test output in a task file would make the format
    /// unreadable for the one thing it exists to record.
    pub output_hash: String,
    pub timed_out: bool,
    /// The last [`TAIL_LINES`] lines of that output, kept for a run that said
    /// no and empty otherwise: what a red close prints so it says why.
    pub tail: Vec<String>,
    /// Where the whole output of a failing run was kept, when its caller asked
    /// for it to be ([`kept_output`]).
    pub kept: Option<PathBuf>,
}

/// How many of a failing verifier's last lines `done` prints.
pub const TAIL_LINES: usize = 20;

/// Where the output of `task`'s verifier `name` is kept when it fails:
/// `<git-common-dir>/ank/output/<task>/<name>.out`.
///
/// Under the common directory and not the temporary one, so every linked
/// worktree names the same file and a reader finds it after `done` has exited;
/// one file per task and verifier, so the latest failure replaces the previous
/// one and a pass removes it.
pub fn kept_output(worktree: &Path, task: &str, name: &str) -> Option<PathBuf> {
    crate::git::common_dir(worktree).map(|d| {
        d.join("ank")
            .join("output")
            .join(task)
            .join(format!("{}.out", sanitized(name)))
    })
}

fn sanitized(name: &str) -> String {
    name.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "-")
}

/// Writes `output` to `path` through a sibling file renamed into place, so a
/// reader sees the previous failure or this one, never half of either.
fn keep(path: &Path, output: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(
        ".{}.{}.tmp",
        path.file_name().and_then(|n| n.to_str()).unwrap_or("out"),
        std::process::id()
    ));
    std::fs::write(&tmp, output)?;
    std::fs::rename(&tmp, path).inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
    })
}

impl Outcome {
    /// `local/<output hash>@<head sha>` — the reference form of §4's trust
    /// hierarchy for a proof Ank produced itself.
    pub fn reference(&self, head: &str, dirty: bool) -> String {
        let head: String = head.chars().take(7).collect();
        // The dirty marker is not decoration: an agent's nominal case is an
        // uncommitted tree, so a proof naming only the HEAD sha would almost
        // always point at a state that was never tested.
        let suffix = if dirty { "+dirty" } else { "" };
        format!("local/{}@{head}{suffix}", &self.output_hash[..12])
    }
}

/// Hash of the definition that actually ran, `<name>@<hash>`.
///
/// `config.yml` stays editable by an agent, and replacing a verifier with
/// `true` is the obvious workaround. This is the first of the two defences,
/// and neither rests on good faith: what ran is anchored in the proof rather
/// than in the current state of the file, so a verifier weakened before or
/// after the `done` — same commit or another — is detectable by comparing this
/// hash with the definition at that commit. The second defence is `check`,
/// which reports the pattern. Faking has to cost more than doing.
pub fn definition_hash(v: &Verifier) -> String {
    let mut h = Sha256::new();
    h.update(v.run.trim().as_bytes());
    h.update([0]);
    h.update(v.timeout.as_secs().to_string().as_bytes());
    // `inputs` is appended only when declared (ADR-d8a328e311b9), so a
    // verifier declaring none keeps the hash every proof recorded before it
    // existed. Each glob goes in length-prefixed: two globs can never trade
    // bytes and produce one hash. `env` is no field (ADR-7224dea611c1).
    if let Some(inputs) = &v.inputs {
        // A set, not a sequence: the order globs are listed in changes
        // nothing about which files can change the result.
        let globs: std::collections::BTreeSet<&str> = inputs.iter().map(String::as_str).collect();
        h.update(b"\0inputs");
        for g in globs {
            put(&mut h, g);
        }
    }
    hex::encode(h.finalize())[..12].to_string()
}

fn put(h: &mut Sha256, s: &str) {
    h.update((s.len() as u64).to_le_bytes());
    h.update(s.as_bytes());
}

pub fn definition_ref(name: &str, v: &Verifier) -> String {
    format!("{name}@{}", definition_hash(v))
}

// ---------------------------------------------------------------------------
// The run registry (ADR-a2faf59f6ce0, ADR-7224dea611c1)
// ---------------------------------------------------------------------------
//
// **What a result is keyed on.** `name@hash`, the definition that ran, and a
// digest of the content on disk when it ran: every file git counts as the work
// tree -- tracked, or untracked and not ignored -- that matches the verifier's
// `inputs`, or every such file when it declares none. Content on disk and not
// a commit, because `done` runs on an uncommitted tree and a key built on HEAD
// would name a state nobody tested.
//
// **Where it lives.** `<git-common-dir>/ank/runs/`, one file per key, never
// versioned: sibling worktrees of one repository share it with no
// coordination, and no other machine ever reads it.
//
// **What may be reused.** A passing entry with the same key, no older than
// `proof.reuse_max_age`, and nothing else: never a failure, which would forbid
// a plain retry and hide a flaky test, and never an attestation on
// `refs/ank/proof`, which nothing here reads.

/// The content key of one verifier, `name@hash` and the inputs digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunKey {
    pub verifier: String,
    pub inputs: String,
}

impl RunKey {
    /// The registry file the key is recorded under.
    fn file_name(&self) -> String {
        let mut h = Sha256::new();
        put(&mut h, &self.verifier);
        put(&mut h, &self.inputs);
        hex::encode(h.finalize())[..32].to_string()
    }
}

/// The work tree as `done` and `verify` read it: the files git counts, the
/// tracked ones gone from disk, and the blob each file would be committed as,
/// for the files somebody asked to have hashed.
pub struct Snapshot {
    pub files: Vec<String>,
    pub deleted: std::collections::HashSet<String>,
    /// One per file, `None` for a file nobody asked to have hashed.
    pub blob: Vec<Option<String>>,
}

/// The work tree listed once and hashed once, over every file a verifier of
/// `names` reads and every file `also` asks for.
///
/// **Three git processes, whatever it holds** (ADR-cc65f1388a71): the two of
/// the listing, and one `hash-object --stdin-paths` over the union. `None`
/// where git refuses or a path cannot be hashed.
pub fn snapshot(
    worktree: &Path,
    names: &[(&str, &Verifier)],
    also: impl Fn(&str) -> bool,
) -> Option<Snapshot> {
    let listed = crate::git::worktree_files(worktree)?;
    let files = listed.counted;
    let mut wanted: Vec<bool> = files.iter().map(|p| also(p)).collect();
    for (_, v) in names {
        match &v.inputs {
            Some(globs) => {
                let set = ank_core::ScopeSet::new(globs).ok()?;
                for (i, p) in files.iter().enumerate() {
                    wanted[i] |= set.matches(p);
                }
            }
            None => wanted.iter_mut().for_each(|w| *w = true),
        }
    }

    let union: Vec<usize> = (0..files.len()).filter(|&i| wanted[i]).collect();
    let mut blob: Vec<Option<String>> = vec![None; files.len()];
    if !union.is_empty() {
        let mut input = String::new();
        for &i in &union {
            input.push_str(&files[i]);
            input.push('\n');
        }
        let args = ["hash-object", "--stdin-paths"];
        let out = crate::git::output_with_stdin(worktree, &args, input.as_bytes()).ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let ids: Vec<&str> = text.lines().collect();
        if ids.len() != union.len() {
            return None;
        }
        for (&i, id) in union.iter().zip(ids) {
            blob[i] = Some(id.to_string());
        }
    }
    Some(Snapshot {
        files,
        deleted: listed.deleted,
        blob,
    })
}

/// The keys of `names`, in order, computed over the work tree as it stands.
///
/// **Two git processes for the whole list, whatever it holds**
/// (ADR-cc65f1388a71): one listing of the work tree, and one `hash-object
/// --stdin-paths` over the union of the files any verifier reads. `None` where
/// a key cannot be established -- git refusing, a path it cannot hash -- and
/// then nothing is reused and nothing recorded: a verb that cannot name its key
/// pays the full cost rather than guessing.
pub fn keys(worktree: &Path, names: &[(&str, &Verifier)]) -> Option<Vec<RunKey>> {
    keys_of(&snapshot(worktree, names, |_| false)?, names)
}

/// The keys of `names` over a snapshot already taken, which hashed at least
/// every file their inputs match.
pub fn keys_of(snap: &Snapshot, names: &[(&str, &Verifier)]) -> Option<Vec<RunKey>> {
    let files = &snap.files;
    let blob = &snap.blob;
    let mut selected: Vec<Vec<usize>> = Vec::with_capacity(names.len());
    for (_, v) in names {
        let picked: Vec<usize> = match &v.inputs {
            Some(globs) => {
                let set = ank_core::ScopeSet::new(globs).ok()?;
                (0..files.len())
                    .filter(|&i| set.matches(&files[i]))
                    .collect()
            }
            None => (0..files.len()).collect(),
        };
        selected.push(picked);
    }

    let mut out = Vec::with_capacity(names.len());
    for ((name, v), picked) in names.iter().zip(selected) {
        let mut paths: Vec<(&str, &str)> = picked
            .iter()
            .map(|&i| (files[i].as_str(), blob[i].as_deref().unwrap_or("")))
            .collect();
        paths.sort();
        let mut h = Sha256::new();
        for (path, id) in paths {
            put(&mut h, path);
            put(&mut h, id);
        }
        out.push(RunKey {
            verifier: definition_ref(name, v),
            inputs: hex::encode(h.finalize())[..16].to_string(),
        });
    }
    Some(out)
}

/// One recorded run: what it was keyed on, what it answered, who ran it and
/// when, and the reference its proof entry carried.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    pub key: RunKey,
    pub ok: bool,
    /// The exit code, `None` for a run killed by a signal or by its timeout.
    pub code: Option<i32>,
    pub by: String,
    /// Seconds since the epoch.
    pub at: i64,
    pub reference: String,
}

/// The registry directory of the repository `worktree` belongs to.
pub fn registry(worktree: &Path) -> Option<PathBuf> {
    crate::git::common_dir(worktree).map(|d| d.join("ank").join("runs"))
}

/// Records `run` under `dir`, replacing whatever the same key held: the latest
/// answer for a key is the one that counts, so a failure after a pass forbids
/// the reuse the pass allowed.
///
/// Written to a temporary file and renamed into place, so a sibling worktree
/// reading at the same moment sees the old entry or the new one, never half.
pub fn record(dir: &Path, run: &Run) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let text = format!(
        "verifier: {}\ninputs: {}\nok: {}\nexit: {}\nby: {}\nat: {}\nref: {}\n",
        run.key.verifier,
        run.key.inputs,
        run.ok,
        run.code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "none".into()),
        run.by,
        run.at,
        run.reference,
    );
    let name = run.key.file_name();
    let tmp = dir.join(format!(".{name}.{}", std::process::id()));
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, dir.join(name))
}

/// The entry recorded under `key`, if any and if it reads whole.
pub fn lookup(dir: &Path, key: &RunKey) -> Option<Run> {
    let text = std::fs::read_to_string(dir.join(key.file_name())).ok()?;
    let field = |name: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(name)?.strip_prefix(": "))
            .map(str::to_string)
    };
    let run = Run {
        key: RunKey {
            verifier: field("verifier")?,
            inputs: field("inputs")?,
        },
        ok: field("ok")? == "true",
        code: field("exit")?.parse().ok(),
        by: field("by")?,
        at: field("at")?.parse().ok()?,
        reference: field("ref")?,
    };
    // The file name is a digest of the key, so this only ever disagrees on a
    // file somebody wrote by hand, and then it is not this key's entry.
    (run.key == *key).then_some(run)
}

/// The entry `done` may reuse for `key` at `now`: passing, and no older than
/// `max_age`. Anything else, absent included, means run the verifier.
pub fn reusable(dir: &Path, key: &RunKey, now: i64, max_age: Duration) -> Option<Run> {
    let run = lookup(dir, key)?;
    let age = now.saturating_sub(run.at);
    (run.ok && run.code == Some(0) && age >= 0 && (age as u64) <= max_age.as_secs()).then_some(run)
}

/// Records one entry per verifier in `ran` that finished, pass or fail, at the
/// instant this is called. `ran` and `outcomes` are aligned: the keys of the
/// verifiers started, in the order they were handed to the runner.
///
/// Shared by `done` and `verify`, so that a run either verb started is recorded
/// on one rule: a failure is recorded as surely as a pass, since the latest
/// answer for a key is the one that counts, and a verifier killed because
/// another failed has no answer and leaves the key as it was.
pub fn record_outcomes(
    dir: &Path,
    ran: &[&RunKey],
    outcomes: &[Result<Option<Outcome>>],
    identity: &str,
    head: &str,
    style: crate::style::Style,
) {
    let at = crate::claim::now_secs();
    for (key, outcome) in ran.iter().zip(outcomes) {
        let Ok(Some(o)) = outcome else { continue };
        let run = Run {
            key: (*key).clone(),
            ok: o.ok,
            code: if o.timed_out { None } else { o.code },
            by: identity.to_string(),
            at,
            reference: o.reference(head, false),
        };
        if let Err(e) = record(dir, &run) {
            eprintln!(
                "{} the run of {} was not recorded for reuse: {e}",
                style.on_stderr().yellow("warning:"),
                o.name
            );
        }
    }
}

// ---------------------------------------------------------------------------
// The verify verb (ADR-c621cb9c6739)
// ---------------------------------------------------------------------------
//
// **What it selects.** Every verifier `config.yml` declares whose `inputs`
// match a changed file, and every verifier that declares no inputs, since a
// verifier reading the whole tree is touched by any change. A file is changed
// when it was committed on the branch since the merge-base with the default
// branch, when it is tracked and its working copy differs from HEAD (deleted
// included), or when it is untracked and not ignored.
//
// **What it writes.** Nothing a reader of the corpus can see: no proof, no
// ref, no entity, and no claim is asked for. The one thing it leaves is the
// run registry, under the git common directory and outside `.ank/`, keyed on
// exactly what `done` computes, so that the `done` which follows finds the run
// and reuses it on the terms ADR-a2faf59f6ce0 sets. The selection itself is
// printed and never recorded: it is how this verb chose, never what a proof
// may rest on.
//
// **It reuses nothing.** The selection is the economy; a verifier it selects is
// one the caller is asking to see run.

/// The files changed in `worktree`, and the base they were measured from: the
/// merge-base with `default`, or `None` when no base could be established.
///
/// Five git processes whatever the tree holds (ADR-cc65f1388a71): two for the
/// listing, one `cat-file --batch-check` for what HEAD records, one
/// `hash-object --stdin-paths` for what the working copy would record, and,
/// when a default branch is named, one `merge-base` and one `diff-tree`.
/// `None` for the list where git cannot answer at all.
pub fn changed_files(
    worktree: &Path,
    default: Option<&str>,
) -> Option<(Vec<String>, Option<String>)> {
    let files = crate::git::worktree_files(worktree)?;
    let mut changed: std::collections::BTreeSet<String> = files.deleted.iter().cloned().collect();

    // What HEAD records for every listed path: a path it does not carry is
    // untracked, and changed by being there.
    let mut input = String::new();
    for p in &files.counted {
        input.push_str("HEAD:");
        input.push_str(p);
        input.push('\n');
    }
    let mut tracked: Vec<(&String, String)> = Vec::new();
    if !files.counted.is_empty() {
        let out = crate::git::output_with_stdin(
            worktree,
            &["cat-file", "--batch-check"],
            input.as_bytes(),
        )
        .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let answers: Vec<&str> = text.lines().collect();
        if answers.len() != files.counted.len() {
            return None;
        }
        for (p, line) in files.counted.iter().zip(answers) {
            let mut parts = line.split(' ');
            match (parts.next(), parts.next()) {
                (Some(oid), Some("blob")) => tracked.push((p, oid.to_string())),
                _ => {
                    changed.insert(p.clone());
                }
            }
        }
    }
    if !tracked.is_empty() {
        let mut input = String::new();
        for (p, _) in &tracked {
            input.push_str(p);
            input.push('\n');
        }
        let args = ["hash-object", "--stdin-paths"];
        let out = crate::git::output_with_stdin(worktree, &args, input.as_bytes()).ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let now: Vec<&str> = text.lines().collect();
        if now.len() != tracked.len() {
            return None;
        }
        for ((p, head), now) in tracked.iter().zip(now) {
            if head != now {
                changed.insert((*p).clone());
            }
        }
    }

    // Committed on the branch since it left the default one. The local branch
    // first, then the remote-tracking one: a clone that never checked the
    // default branch out still has a base to measure from.
    let base = default.and_then(|d| {
        [d.to_string(), format!("refs/remotes/origin/{d}")]
            .iter()
            .find_map(|rev| crate::git::merge_base(worktree, rev, "HEAD").ok().flatten())
    });
    if let Some(base) = &base {
        let args = [
            "diff-tree",
            "-r",
            "-z",
            "--name-only",
            base.as_str(),
            "HEAD",
        ];
        let out = crate::git::output(worktree, &args).ok()?;
        if !out.status.success() {
            return None;
        }
        for p in out.stdout.split(|&b| b == 0).filter(|p| !p.is_empty()) {
            changed.insert(String::from_utf8_lossy(p).into_owned());
        }
    }
    Some((changed.into_iter().collect(), base))
}

/// Whether verifier `v` is touched by `changed`: always without inputs, and
/// otherwise when one of its globs matches one of the files. `None` when the
/// globs do not parse, which the configuration parser already refuses.
pub fn selects(v: &Verifier, changed: &[String]) -> Option<bool> {
    match &v.inputs {
        None => Some(true),
        Some(globs) => {
            let set = ank_core::ScopeSet::new(globs).ok()?;
            Some(changed.iter().any(|p| set.matches(p)))
        }
    }
}

pub fn run_verb(
    inv: &crate::cli::Invocation,
    repo: &crate::repo::Repo,
    cfg: &crate::config::Config,
    identity: &str,
    out: &mut dyn std::io::Write,
) -> Result<ExitCode> {
    crate::git::ensure_worktree_usable(repo)?;
    let style = inv.style();

    // An indeterminable default branch degrades rather than refuses
    // (ADR-9307e5d214a7): the tree's own changes are still a selection, and
    // the line below says what it could not measure.
    let default = crate::git::resolve_default_branch(
        cfg.default_branch.as_deref(),
        crate::git::origin_head(&repo.worktree)?.as_deref(),
    )
    .ok();
    let Some((changed, base)) = changed_files(&repo.worktree, default.as_deref()) else {
        return Err(CliError::new(
            ExitCode::Environment,
            "git could not list the work tree, so no change can be selected from",
        )
        .with_hint("git ls-files --cached --others --exclude-standard"));
    };
    if base.is_none() && !inv.quiet() {
        // Selecting less than a change touches is the one direction this verb
        // must not err in, so the gap is said out loud.
        eprintln!(
            "{} no merge-base with the default branch: only the work tree's changes are selected from",
            style.on_stderr().yellow("warning:"),
        );
    }

    let mut selected: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for (name, v) in &cfg.verifiers {
        if selects(v, &changed).unwrap_or(true) {
            selected.push(name.clone());
        } else {
            skipped.push(name.clone());
        }
    }

    let defs: Vec<(&str, &Verifier)> = selected
        .iter()
        .map(|n| (n.as_str(), &cfg.verifiers[n]))
        .collect();
    // Taken before anything runs, as `done` takes them: what a run is recorded
    // against is the content it was started on.
    let registry = registry(&repo.worktree);
    let keys = registry.as_ref().and_then(|_| {
        if defs.is_empty() {
            Some(Vec::new())
        } else {
            keys(&repo.worktree, &defs)
        }
    });
    let head = crate::git::run(&repo.worktree, &["rev-parse", "HEAD"]).unwrap_or_default();

    let outcomes = crate::done::run_concurrently(
        repo,
        cfg,
        &selected,
        "config.yml",
        None,
        cfg.verifier_jobs(),
    );
    match (&registry, &keys) {
        (Some(dir), Some(keys)) => {
            let ran: Vec<&RunKey> = keys.iter().collect();
            record_outcomes(dir, &ran, &outcomes, identity, &head, style);
        }
        _ if !selected.is_empty() => eprintln!(
            "{} no content key could be established: these runs are not recorded for reuse",
            style.on_stderr().yellow("warning:"),
        ),
        _ => {}
    }

    // Reported in configuration order once all have finished, and the first
    // failure in that order is the answer, as `done` reports it.
    let mut rows: Vec<(String, Outcome)> = Vec::new();
    for (name, outcome) in selected.iter().zip(outcomes) {
        let Some(outcome) = outcome? else { break };
        let err = style.on_stderr();
        eprintln!(
            "running: {name} ... {} ({:.1}s)",
            if outcome.ok {
                err.green("ok")
            } else {
                err.red("FAILED")
            },
            outcome.elapsed.as_secs_f64()
        );
        if !outcome.ok {
            return Err(failure(&outcome, &cfg.verifiers[name].run));
        }
        rows.push((name.clone(), outcome));
    }

    if inv.json() {
        let ran = rows.iter().map(|(name, o)| {
            crate::json::Obj::new()
                .str("name", name)
                .str("verifier", &definition_ref(name, &cfg.verifiers[name]))
                .bool("ok", o.ok)
                .finish()
        });
        let doc = crate::json::Obj::document()
            .opt_str("base", base.as_deref())
            .num("changed", changed.len())
            .array("ran", ran)
            .strings("skipped", &skipped)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let from = match &base {
            Some(b) => format!("since {}", &b[..b.len().min(7)]),
            None => "in the work tree".to_string(),
        };
        let _ = writeln!(
            out,
            "{} file(s) changed {from}; ran {} of {} verifier(s), recorded for reuse",
            changed.len(),
            rows.len(),
            cfg.verifiers.len()
        );
        for (name, _) in &rows {
            let _ = writeln!(out, "  {name}  {}", style.green("ok"));
        }
        for name in &skipped {
            let _ = writeln!(out, "  {name}  skipped, no input changed");
        }
    }
    Ok(ExitCode::Ok)
}

/// Locates a POSIX shell.
///
/// `sh` on `PATH` when there is one. Otherwise, on Windows, the one Git for
/// Windows ships: git is already a hard dependency, so the shell comes free and
/// verifiers stay written once, in POSIX syntax, for the whole team. Deriving
/// it from git's own location on `PATH` keeps this out of `git.rs` and out of
/// the plumbing list — asking git where it lives would be a new subcommand for
/// something a directory walk answers.
///
/// **Never a silent fallback to `cmd`.** A verifier written in POSIX syntax and
/// handed to `cmd` fails in ways that look like the code is wrong.
pub fn find_sh() -> Result<PathBuf> {
    if let Some(found) = on_path("sh") {
        return Ok(found);
    }
    if let Some(git) = on_path("git") {
        // C:\Program Files\Git\cmd\git.exe -> C:\Program Files\Git\bin\sh.exe
        let mut dir = git.parent();
        for _ in 0..3 {
            let Some(base) = dir else { break };
            for candidate in ["bin/sh.exe", "usr/bin/sh.exe", "bin/sh"] {
                let p = base.join(candidate);
                if p.is_file() {
                    return Ok(p);
                }
            }
            dir = base.parent();
        }
    }
    Err(CliError::new(
        ExitCode::Environment,
        "sh not found: verifiers run through sh -c on the three platforms",
    )
    .with_hint(if cfg!(windows) {
        GIT_FOR_WINDOWS
    } else {
        POSIX_SH
    }))
}

fn on_path(name: &str) -> Option<PathBuf> {
    let exts: &[&str] = if cfg!(windows) { &["", ".exe"] } else { &[""] };
    let path = std::env::var_os(ank_contract::env::PATH)?;
    for dir in std::env::split_paths(&path) {
        for ext in exts {
            let p = dir.join(format!("{name}{ext}"));
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

/// Runs one verifier in `cwd`.
///
/// Output goes to a temporary file rather than a pipe, deliberately: a verifier
/// that prints more than the pipe buffer holds would deadlock against a parent
/// that is waiting for it to exit, and a test suite is exactly the kind of
/// command that prints a lot.
pub fn run(cwd: &Path, name: &str, v: &Verifier) -> Result<Outcome> {
    Ok(run_until(cwd, name, v, None, &|| false)?.expect("a run nothing stops is never cancelled"))
}

/// [`run`], polling `stop` while the verifier runs: once it answers true the
/// verifier is killed and `None` comes back, an outcome nobody will read.
///
/// This is what lets `done` run verifiers side by side (TASK-26eab418555a):
/// once one fails, the ones declared after it can no longer change the
/// answer, and letting them run on would leave them running after `done`
/// has already said no.
///
/// `keep` is where a failing run's whole output is written, replacing what a
/// previous failure left there, and what a passing run removes; `None` keeps
/// nothing. A cancelled run touches neither: it has no answer.
pub fn run_until(
    cwd: &Path,
    name: &str,
    v: &Verifier,
    keep: Option<&Path>,
    stop: &dyn Fn() -> bool,
) -> Result<Option<Outcome>> {
    let sh = find_sh()?;
    let out_path = std::env::temp_dir().join(format!(
        "ank-verify-{}-{}.out",
        std::process::id(),
        name.replace(|c: char| !c.is_alphanumeric(), "-")
    ));
    let file = std::fs::File::create(&out_path).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot capture verifier output: {e}"),
        )
    })?;
    let errs = file.try_clone().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot capture verifier output: {e}"),
        )
    })?;

    let started = Instant::now();
    let mut command = Command::new(&sh);
    // A group of its own, so that killing the verifier kills what it started:
    // `sh -c "a; b"` keeps the shell as a parent, and a kill aimed at the
    // shell alone leaves `b` running with nobody waiting for it.
    #[cfg(unix)]
    std::os::unix::process::CommandExt::process_group(&mut command, 0);
    let mut child = command
        .current_dir(cwd)
        .arg("-c")
        .arg(&v.run)
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(errs))
        .spawn()
        .map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("cannot run verifier '{name}': {e}"),
            )
            .with_hint(format!("{} -c {:?}", sh.display(), v.run))
        })?;

    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if stop() {
                    kill_tree(&mut child);
                    let _ = std::fs::remove_file(&out_path);
                    return Ok(None);
                }
                if started.elapsed() >= v.timeout {
                    kill_tree(&mut child);
                    timed_out = true;
                    break std::process::ExitStatus::default();
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                return Err(CliError::new(
                    ExitCode::Environment,
                    format!("cannot wait for verifier '{name}': {e}"),
                ))
            }
        }
    };

    let elapsed = started.elapsed();
    let output = std::fs::read(&out_path).unwrap_or_default();
    let _ = std::fs::remove_file(&out_path);
    let code = status.code();

    // 126 and 127 are the shell's own way of saying "I could not run that":
    // not executable, and not found. Neither is the agent's code failing, so
    // neither is a code 5.
    if !timed_out {
        if let Some(c @ (126 | 127)) = code {
            let tail = String::from_utf8_lossy(&output);
            let tail = tail.lines().last().unwrap_or("").trim();
            return Err(CliError::new(
                ExitCode::Environment,
                format!("verifier '{name}' could not run (shell code {c}): {tail}"),
            )
            .with_hint(format!("sh -c {:?}", v.run)));
        }
    }

    let ok = !timed_out && status.success();
    let (mut tail, mut kept) = (Vec::new(), None);
    if ok {
        if let Some(path) = keep {
            let _ = std::fs::remove_file(path);
        }
    } else {
        let text = String::from_utf8_lossy(&output);
        let lines: Vec<&str> = text.lines().collect();
        tail = lines[lines.len().saturating_sub(TAIL_LINES)..]
            .iter()
            .map(|l| l.to_string())
            .collect();
        if let Some(path) = keep {
            match self::keep(path, &output) {
                Ok(()) => kept = Some(path.to_path_buf()),
                // The verdict stands without the file: a code 5 is still the
                // answer, and the tail printed below still says why.
                Err(e) => eprintln!("warning: cannot keep the output at {}: {e}", path.display()),
            }
        }
    }

    let mut h = Sha256::new();
    h.update(&output);
    Ok(Some(Outcome {
        name: name.to_string(),
        ok,
        code,
        elapsed,
        output_hash: hex::encode(h.finalize()),
        timed_out,
        tail,
        kept,
    }))
}

/// Kills a verifier and everything it started, then reaps it.
///
/// On Unix the verifier leads a process group of its own and the group is
/// killed whole. On Windows `taskkill /T` walks the tree from the shell down.
/// Both through a process rather than a binding: no crate enters for one
/// signal, and the shell's own kill is the fallback when neither runs.
fn kill_tree(child: &mut std::process::Child) {
    let pid = child.id().to_string();
    #[cfg(unix)]
    let _ = Command::new("kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(windows)]
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &pid])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    #[cfg(not(any(unix, windows)))]
    let _ = pid;
    let _ = child.kill();
    let _ = child.wait();
}

/// The code 5 a failed verifier deserves, with the elapsed time when it ran out
/// of it.
pub fn failure(outcome: &Outcome, run: &str) -> CliError {
    let detail = if outcome.timed_out {
        format!(
            "verifier '{}' timed out after {:.1}s",
            outcome.name,
            outcome.elapsed.as_secs_f64()
        )
    } else {
        format!(
            "verifier '{}' failed (exit {})",
            outcome.name,
            outcome
                .code
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".into())
        )
    };
    CliError::new(ExitCode::Proof, detail).with_hint(format!("sh -c {run:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verifier(run: &str, secs: u64) -> Verifier {
        Verifier {
            run: run.to_string(),
            timeout: Duration::from_secs(secs),
            inputs: None,
        }
    }

    fn cwd() -> PathBuf {
        std::env::temp_dir()
    }

    #[test]
    fn sh_is_found_on_the_three_platforms() {
        // git is a hard dependency and ships sh on Windows, so this must
        // succeed everywhere the tool is supported. On this machine sh is not
        // on PATH at all, which is precisely the case the fallback exists for.
        let sh = find_sh().expect("sh must be resolvable wherever ank runs");
        assert!(sh.is_file(), "{}", sh.display());
    }

    #[test]
    fn a_verifier_that_passes_is_ok_and_hashes_its_output() {
        let out = run(&cwd(), "greet", &verifier("echo hello", 30)).unwrap();
        assert!(out.ok);
        assert_eq!(out.code, Some(0));
        assert!(!out.timed_out);
        assert_eq!(out.output_hash.len(), 64);

        // The hash is of the output, so two different outputs differ.
        let other = run(&cwd(), "greet", &verifier("echo goodbye", 30)).unwrap();
        assert_ne!(out.output_hash, other.output_hash);
        // And the same output hashes the same, which is what makes a proof
        // comparable across runs.
        let again = run(&cwd(), "greet", &verifier("echo hello", 30)).unwrap();
        assert_eq!(out.output_hash, again.output_hash);
    }

    #[test]
    fn a_verifier_that_says_no_is_a_failure_and_not_an_environment_problem() {
        let out = run(&cwd(), "nope", &verifier("exit 1", 30)).unwrap();
        assert!(!out.ok);
        assert_eq!(out.code, Some(1));

        let err = failure(&out, "exit 1");
        assert_eq!(
            err.code,
            ExitCode::Proof,
            "a verifier that ran and refused is code 5"
        );
        assert!(err.message.contains("nope"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    #[test]
    fn a_command_that_does_not_exist_is_the_environment_and_not_the_code() {
        // 127 from the shell. Reporting this as a failed verifier would send
        // an agent to fix code that was never wrong.
        let err = run(&cwd(), "missing", &verifier("no-such-command-anywhere", 30)).unwrap_err();
        assert_eq!(err.code, ExitCode::Environment, "{}", err.message);
        assert!(err.message.contains("missing"), "{}", err.message);
        assert!(err.hint.is_some(), "and it says what to run");
    }

    #[test]
    fn a_timeout_is_a_failure_carrying_the_elapsed_time() {
        let out = run(&cwd(), "slow", &verifier("sleep 30", 1)).unwrap();
        assert!(out.timed_out);
        assert!(!out.ok);
        assert!(out.elapsed >= Duration::from_secs(1));
        assert!(
            out.elapsed < Duration::from_secs(20),
            "the child is killed, not waited on: {:?}",
            out.elapsed
        );

        let err = failure(&out, "sleep 30");
        assert_eq!(
            err.code,
            ExitCode::Proof,
            "exceeding the timeout is a code 5 (§4)"
        );
        assert!(err.message.contains("timed out"), "{}", err.message);
        assert!(err.message.contains('s'), "with the elapsed time");
    }

    #[test]
    fn a_verifier_that_prints_a_great_deal_does_not_deadlock() {
        // The reason output goes to a file rather than a pipe: a test suite is
        // exactly the kind of command that outruns a pipe buffer, and the
        // deadlock would look like a hung tool.
        let out = run(
            &cwd(),
            "chatty",
            &verifier(
                "i=0; while [ $i -lt 4000 ]; do echo 'a line of output'; i=$((i+1)); done",
                60,
            ),
        )
        .unwrap();
        assert!(out.ok, "code {:?}", out.code);
    }

    #[test]
    fn the_definition_hash_moves_when_the_definition_does() {
        let a = verifier("cargo test --workspace", 600);
        let same = verifier("  cargo test --workspace  ", 600);
        let weakened = verifier("true", 600);
        let slower = verifier("cargo test --workspace", 60);

        assert_eq!(definition_hash(&a), definition_hash(&same), "trimmed alike");
        assert_ne!(
            definition_hash(&a),
            definition_hash(&weakened),
            "replacing a verifier with true is the workaround this anchors"
        );
        assert_ne!(
            definition_hash(&a),
            definition_hash(&slower),
            "the timeout is part of what ran"
        );
        assert_eq!(
            definition_ref("cargo-test", &a),
            format!("cargo-test@{}", definition_hash(&a))
        );
    }

    /// ADR-7224dea611c1: `inputs` is part of what ran, and a verifier
    /// declaring none hashes exactly as it did before it existed.
    #[test]
    fn the_definition_hash_covers_inputs_and_nothing_when_none_is_declared() {
        let p = std::path::Path::new("config.yml");
        let hash = |yaml: &str| {
            let cfg = crate::config::parse(&format!("schema: 1\nverifiers:\n  t:\n{yaml}"), p)
                .unwrap_or_else(|e| panic!("{yaml}: {}", e.message));
            definition_hash(&cfg.verifiers["t"])
        };
        let bare = hash("    run: cargo test\n");
        // sha256("cargo test\0600"), first twelve hex digits: the formula
        // every proof recorded so far was hashed with.
        assert_eq!(bare, "ea889101030e");

        let inputs = hash("    run: cargo test\n    inputs: [\"src/**\", \"Cargo.toml\"]\n");
        assert_ne!(inputs, bare);
        assert_eq!(
            hash("    run: cargo test\n    inputs:\n      - Cargo.toml\n      - src/**\n"),
            inputs,
            "a set of globs, whatever order and style lists them"
        );
        assert_ne!(
            hash("    run: cargo test\n    inputs: [\"src/**\"]\n"),
            inputs
        );
    }

    #[test]
    fn the_local_reference_names_the_head_and_says_when_the_tree_was_dirty() {
        let out = Outcome {
            name: "t".into(),
            ok: true,
            code: Some(0),
            elapsed: Duration::from_secs(1),
            output_hash: "0123456789abcdef".repeat(4),
            timed_out: false,
            tail: Vec::new(),
            kept: None,
        };
        assert_eq!(
            out.reference("a3f9c21deadbeef", false),
            "local/0123456789ab@a3f9c21"
        );
        assert_eq!(
            out.reference("a3f9c21deadbeef", true),
            "local/0123456789ab@a3f9c21+dirty",
            "an uncommitted tree is the agent's nominal case, and must show"
        );
    }
}

// ---------------------------------------------------------------------------
// One task's verifiers at a time per repository (ADR-eab130f731f3)
// ---------------------------------------------------------------------------
//
// **Where.** `<git-common-dir>/ank/verifier.lock`, the one directory every
// worktree of a repository shares, so two `done` in two worktrees contend and
// two unrelated repositories never do.
//
// **How.** An OS file lock (`File::lock`), which the operating system releases
// when the process ends however it ends: a killed `done` leaves nothing held
// and no file to clean up. Who holds it is written beside it, in
// `verifier.holder`, and not in the locked file itself: on Windows the lock is
// mandatory, and a waiter could not read a locked file to say who holds it.

/// The lock, held for as long as this value lives.
pub struct VerifierLock {
    _file: std::fs::File,
    holder: PathBuf,
}

impl Drop for VerifierLock {
    fn drop(&mut self) {
        // A clean release says nobody holds it; a killed holder leaves a stale
        // line that is only read while somebody else holds the lock, and that
        // somebody has overwritten it by then.
        let _ = std::fs::remove_file(&self.holder);
    }
}

/// Takes the repository's verifier lock for `task` run by `identity`,
/// waiting while another `done` holds it and saying once, on standard error,
/// which task and identity that is. `None` when there is no common directory
/// to hold it in, or the lock cannot be opened: the lock orders runs, it never
/// refuses one.
pub fn take_verifier_lock(worktree: &Path, task: &str, identity: &str) -> Option<VerifierLock> {
    let dir = crate::git::common_dir(worktree)?.join("ank");
    std::fs::create_dir_all(&dir).ok()?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(dir.join("verifier.lock"))
        .ok()?;
    let holder = dir.join("verifier.holder");

    // The holder writes its line just after taking the lock, so a waiter that
    // arrives in between finds no line yet: it tries again for a moment before
    // settling for not knowing.
    let mut tries = 0;
    loop {
        match file.try_lock() {
            Ok(()) => break,
            Err(std::fs::TryLockError::WouldBlock) => {
                let line = std::fs::read_to_string(&holder).unwrap_or_default();
                let line = line.trim();
                if line.is_empty() && tries < 50 {
                    tries += 1;
                    std::thread::sleep(Duration::from_millis(20));
                    continue;
                }
                let who = line
                    .split_once(' ')
                    .map(|(t, i)| format!("{t} (by {i})"))
                    .unwrap_or_else(|| "another done".to_string());
                eprintln!(
                    "waiting: the verifiers of {who} hold the repository's verifier lock; \
                     {task} runs its own once they finish (verifier_lock: false runs them together)"
                );
                file.lock().ok()?;
                break;
            }
            Err(std::fs::TryLockError::Error(_)) => return None,
        }
    }

    let tmp = dir.join(format!("verifier.holder.{}", std::process::id()));
    if std::fs::write(&tmp, format!("{task} {identity}\n")).is_ok()
        && std::fs::rename(&tmp, &holder).is_err()
    {
        let _ = std::fs::remove_file(&tmp);
    }
    Some(VerifierLock {
        _file: file,
        holder,
    })
}
