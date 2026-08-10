//! The done verb: verification, proofs, and the completion ref (§4, §7).
//!
//! The point where "faking must cost more than doing" becomes code.
//!
//! **Two modes, never ambiguous.** A task that declares `verify` has all of its
//! verifiers run by Ank, and `--proof` is refused — the agent cannot
//! short-circuit its own verification. A task that declares none requires
//! `--proof`, and Ank validates what it can: a `commit:` is checked against
//! git, an `assertion:` is recorded as it stands and marked weak. The two modes
//! never overlap, so there is no combination in which an agent chooses how much
//! verification to submit to.
//!
//! **The frozen criterion is checked before anything runs.** The freeze is
//! anchored in the claim record, not defended by this command
//! (ADR-6b3f19e08a24): editing the file unblocks nothing, it only makes the
//! divergence visible, and it surfaces here as a code 6 before a single
//! verifier starts.
//!
//! **The ref is not deleted, it is transformed** (ADR-bcf222a31525). A `done`
//! lives on the branch that produced it, so between the work and the merge the
//! task would look free to every other agent. The claim record becomes a
//! completion record pointing at the commit, with no TTL, and only `check`
//! prunes it once the default branch has caught up.
//!
//! A `done` that fails leaves the ref exactly as it was. Transforming it before
//! the verdict is in would mark a task finished that is not, and nobody could
//! pick it up again.

use crate::claim::{self, ClaimRecord, Held, Record};
use crate::cli::{CliError, Invocation, Result};
use crate::config::Config;
use crate::git;
use crate::repo::Repo;
use crate::store::{version_of, Store};
use crate::verify;
use ank_core::{
    append_log, freeze_hash_short, verify_frozen, Entity, EntityId, LogEntry, Proof, ProofType,
    ScopeSet, TaskStatus,
};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;

fn parse_proof_type(text: &str) -> Option<ProofType> {
    match text {
        "commit" => Some(ProofType::Commit),
        "human-review" => Some(ProofType::HumanReview),
        "assertion" => Some(ProofType::Assertion),
        "test" => Some(ProofType::Test),
        _ => None,
    }
}

/// A hash of the scope files' content at execution time, `scope/<hash>`.
///
/// This is what actually captures what was tested. An agent's nominal case is
/// an uncommitted working tree, so anchoring on the HEAD sha alone would almost
/// always point at a state nobody ran anything against.
pub fn scope_hash(root: &Path, globs: &[String]) -> Option<String> {
    let set = ScopeSet::new(globs).ok()?;
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect(root, root, &set, &mut files, 0);
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut h = Sha256::new();
    for (rel, bytes) in &files {
        h.update(rel.as_bytes());
        h.update([0]);
        h.update(bytes);
        h.update([0]);
    }
    Some(format!("scope/{}", &hex::encode(h.finalize())[..12]))
}

fn collect(
    root: &Path,
    dir: &Path,
    set: &ScopeSet,
    out: &mut Vec<(String, Vec<u8>)>,
    depth: usize,
) {
    // A guard rather than a promise of completeness: a scope pointing at a
    // pathologically deep tree should cost a truncated hash, not a stack.
    if depth > 32 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        // Neither is ever in a scope, and both are large.
        if name == ".git" || name == "target" {
            continue;
        }
        if path.is_dir() {
            collect(root, &path, set, out, depth + 1);
            continue;
        }
        let Ok(rel) = path.strip_prefix(root) else {
            continue;
        };
        let rel = rel.to_string_lossy().replace('\\', "/");
        if set.matches(&rel) {
            if let Ok(bytes) = std::fs::read(&path) {
                out.push((rel, bytes));
            }
        }
    }
}

pub fn run(
    inv: &Invocation,
    repo: &Repo,
    cfg: &Config,
    identity: &str,
    out: &mut dyn Write,
) -> Result<i32> {
    let store = Store::new(&repo.ank);

    // HEAD is derived: the task this agent holds a live claim on. An explicit
    // id is allowed but redundant, and must match — it is never a way to act on
    // somebody else's task (§4).
    let (id, held) = resolve_head(&repo.root, &store, inv.positionals.first(), identity)?;
    let loaded = store.load(&id)?;
    let base_version = version_of(&loaded.entity);
    let Entity::Task(mut task) = loaded.entity else {
        return Err(CliError::new(1, format!("{id} is not a task")));
    };

    let criteria = task.done_criteria.clone().unwrap_or_default();
    let Record::Claim(claim_record) = &held.record else {
        return Err(
            CliError::new(4, format!("{id} carries a completion record, not a claim"))
                .with_hint("ank context"),
        );
    };

    // Before anything runs. A criterion weakened after the claim is a code 6,
    // and the verifiers never start: running them would lend the transition a
    // credibility the anchor has already withdrawn.
    check_frozen_criteria(&id, &criteria, claim_record)?;

    // The claim record's other hash, and until now the decorative one. Before
    // anything runs for the same reason: an agent told a new rule landed over
    // these files should hear it before it spends a minute on verifiers, not
    // after the transition is already written.
    warn_on_constraint_drift(&store, repo, &task, claim_record, inv.style());

    let declared: Vec<String> = task.verify.clone();
    let proofs = if declared.is_empty() {
        let usage = ProofUsage {
            command: "ank done".to_string(),
            purpose: format!("move {id} to done"),
        };
        vec![submitted_proof(inv, &repo.root, &usage, Some(&criteria))?]
    } else {
        if inv.value("--proof").is_some() {
            return Err(CliError::new(
                5,
                format!(
                    "{id} declares verifiers ({}), so --proof is refused",
                    declared.join(", ")
                ),
            )
            .with_hint("ank done"));
        }
        run_verifiers(
            repo,
            cfg,
            &declared,
            &task.scope,
            &criteria,
            &id,
            inv.style(),
        )?
    };

    // Durable state first, the ref second. A file written and a ref left behind
    // is recoverable by re-running; a ref moved to completed over a file that
    // never took would hide a task nobody can pick up again.
    task.status = TaskStatus::Done;
    task.proof.extend(proofs.iter().cloned());
    let entry = LogEntry {
        timestamp: claim::now_utc(),
        who: identity.to_string(),
        message: done_message(&proofs),
    };
    task.body = append_log(&task.body, &entry);
    store.write(&Entity::Task(task.clone()), base_version)?;

    let completed = claim::complete(&repo.root, &id, identity)?;

    if inv.json() {
        let _ = writeln!(
            out,
            "{{\"task\":\"{id}\",\"status\":\"done\",\"commit\":\"{}\",\"branch\":{},\"proofs\":{}}}",
            completed.commit,
            completed
                .branch
                .as_ref()
                .map(|b| format!("\"{b}\""))
                .unwrap_or_else(|| "null".to_string()),
            proofs.len()
        );
    } else if !inv.quiet() {
        for p in &proofs {
            let _ = writeln!(
                out,
                "proof recorded: {} -> {}{}",
                p.verifier.as_deref().unwrap_or(p.proof_type.as_str()),
                p.reference,
                p.tree
                    .as_ref()
                    .map(|t| format!("  ({t})"))
                    .unwrap_or_default()
            );
        }
        let _ = writeln!(
            out,
            "{} -> {}",
            inv.style().id(&id.to_string()),
            // Through `landed`, not `green`: the colour of a landing state is
            // the colour of its marker, and one table answers both (§4).
            inv.style().landed("done")
        );
    }
    Ok(0)
}

fn done_message(proofs: &[Proof]) -> String {
    let refs: Vec<String> = proofs
        .iter()
        .map(|p| format!("{}:{}", p.proof_type.as_str(), p.reference))
        .collect();
    format!("done, proof {}", refs.join(" "))
}

/// The task this agent may finish. An id given explicitly must match HEAD,
/// otherwise code 6: the optional id exists for explicitness in scripts, never
/// as a way to act on another agent's task (§4).
fn resolve_head(
    cwd: &Path,
    store: &Store,
    given: Option<&String>,
    identity: &str,
) -> Result<(EntityId, Held)> {
    let mut mine: Option<(EntityId, Held)> = None;
    for r in git::ank_refs(cwd)? {
        let Some(rest) = r.name.strip_prefix(claim::CLAIMS_PREFIX) else {
            continue;
        };
        let Ok(id) = EntityId::parse(rest) else {
            continue;
        };
        let Some(held) = claim::read(cwd, &id)? else {
            continue;
        };
        if let Record::Claim(c) = &held.record {
            if c.holder == identity && !claim::is_expired(c, claim::now_secs(), &id)? {
                mine = Some((id, held));
                break;
            }
        }
    }

    let Some((id, held)) = mine else {
        return Err(CliError::new(6, "no task in progress for this agent").with_hint("ank context"));
    };
    if let Some(given) = given {
        let asked = store.resolve(given)?;
        if asked != id {
            return Err(
                CliError::new(6, format!("{asked} is not the task in progress ({id})"))
                    .with_hint(format!("ank done {id}")),
            );
        }
    }
    Ok((id, held))
}

/// The freeze, checked at the point of use. The CLI is not a gatekeeper: the
/// hash lives in the claim record, which the file's editor does not control.
/// A constraint accepted while the claim was held (§7, TASK-bfa325e55424).
///
/// The claim record carries two hashes. The criteria freeze is checked above
/// and refuses; this is the other one, and §7 says exactly what it is for: it
/// closes the long-work window, because a constraint accepted while the agent
/// works changes what applies to its scope, and `done` warns — inviting a
/// re-read of `ank context`.
///
/// Half of that was true. `check` reported the case and `done` never read
/// `ClaimRecord.constraints` at all, so the field was written at every claim,
/// carried for the whole life of the ref, and consulted only by a verb the
/// agent is not required to run. The window it left open is the one the design
/// already judged worth closing: claim, work for an hour, a rule lands over the
/// scope meanwhile, and the transition completes in silence.
///
/// **It warns and never blocks.** A constraint that landed after the work
/// started does not necessarily concern work already finished, and refusing
/// would punish exactly the case §7 singles out.
///
/// **On standard error**, unlike the `running:` lines beside it. Those already
/// make `done --json` unparseable, which §4 forbids and which is a defect of
/// its own; adding a second line to stdout would deepen it to say something no
/// parser asked for.
///
/// A constraint set that cannot be computed says nothing rather than guessing:
/// this is an advisory read, and failing a `done` over it would be the blocking
/// this is explicitly not.
fn warn_on_constraint_drift(
    store: &Store,
    repo: &Repo,
    task: &ank_core::Task,
    claim: &ClaimRecord,
    style: crate::style::Style,
) {
    let Ok(applicable) = claim::applicable_constraints(store, repo, task) else {
        return;
    };
    if claim::constraints_hash(&applicable) == claim.constraints {
        return;
    }
    let style = style.on_stderr();
    eprintln!(
        "{} constraints over this scope changed while the claim was held ({} bind it now)",
        style.yellow("warning:"),
        applicable.len()
    );
    eprintln!("  -> ank context");
}

fn check_frozen_criteria(id: &EntityId, criteria: &str, claim: &ClaimRecord) -> Result<()> {
    if criteria.trim().is_empty() {
        return Err(CliError::new(7, format!("{id} has no done_criteria")).with_hint("ank context"));
    }
    if !verify_frozen(criteria, &claim.criteria) {
        return Err(CliError::new(
            6,
            format!(
                "done_criteria of {id} changed since the claim (claimed {}, now {})",
                claim.criteria,
                freeze_hash_short(criteria)
            ),
        )
        .with_hint(format!("git diff -- .ank/tasks/{id}.md")));
    }
    Ok(())
}

/// Runs the declared verifiers and gathers one proof each.
///
/// **It takes no writer**, and that is the point rather than an accident: the
/// progress it reports goes to standard error, so a function with no way to
/// reach stdout cannot put a line ahead of the JSON document `done` prints
/// there (§4, TASK-2eefcdd80124).
#[allow(clippy::too_many_arguments)]
fn run_verifiers(
    repo: &Repo,
    cfg: &Config,
    declared: &[String],
    scope: &[String],
    criteria: &str,
    id: &EntityId,
    style: crate::style::Style,
) -> Result<Vec<Proof>> {
    let head = git::run(&repo.root, &["rev-parse", "HEAD"]).unwrap_or_default();
    let tree = scope_hash(&repo.root, scope);
    let criteria_hash = freeze_hash_short(criteria);
    let mut proofs = Vec::new();

    for name in declared {
        let Some(def) = cfg.verifier(name) else {
            // Not the agent's code failing: the corpus references a verifier
            // the configuration does not declare.
            return Err(CliError::new(
                9,
                format!("{id} declares verifier '{name}', absent from config.yml"),
            )
            .with_hint(format!("ank config verifiers.{name}.run \"<command>\"")));
        };
        let outcome = verify::run(&repo.root, name, def)?;
        // Progress, and therefore standard error: it is not part of the answer,
        // and §4 requires `--json` to leave stdout byte-for-byte what a
        // caller's parser reads. This line used to precede the JSON document on
        // stdout, so `ank done --json | <parser>` failed on its first line
        // (TASK-2eefcdd80124). Unconditional rather than gated on `--json`,
        // because a gate at each printing site is one more chance to forget
        // one, and progress belongs on stderr for a human too.
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
            // Every proof already gathered is discarded with the transition: a
            // partial verification anchors nothing.
            return Err(verify::failure(&outcome, &def.run));
        }
        proofs.push(Proof {
            proof_type: ProofType::Test,
            reference: outcome.reference(&head, false),
            tree: tree.clone(),
            criteria: Some(criteria_hash.clone()),
            verifier: Some(verify::definition_ref(name, def)),
        });
    }
    Ok(proofs)
}

/// How a caller of [`submitted_proof`] names itself in the hints that parser
/// emits.
///
/// The grammar is one thing and the next command is another. §4 is explicit
/// that a hint is the exact command to run, and `ank done --proof commit:<sha>`
/// is not the command an `attest` caller needs — a shared parser emitting a
/// generic hint would trade a real duplication for a broken error surface.
/// So the caller supplies its own, and only its own.
pub struct ProofUsage {
    /// The command up to `--proof`: `ank done`, or `ank attest TASK-8f3a`.
    pub command: String,
    /// What the proof is required *for*, completing "proof required to ...".
    pub purpose: String,
}

/// The `<type>:<ref>` grammar of `--proof`, parsed in one place.
///
/// Two callers: `done`, on the path taken only by a task that declares no
/// verifier, and `attest`, which records a proof made elsewhere. `attest` was
/// written with a copy of this because the original was private and widening it
/// was outside TASK-1f4f7b57039b's scope, and the failure mode was never the
/// duplication itself — it was the drift. The day one copy learned a new proof
/// type, or stopped checking a commit against git, nothing made the other
/// follow, and two verbs disagreeing about what a proof *is* would be a worse
/// defect than the copy.
///
/// `criteria` is optional because the two callers differ there and the
/// difference is real: `done` always holds the frozen criterion it just
/// verified, while `attest` records against whatever the finished task carries,
/// which may be nothing.
pub fn submitted_proof(
    inv: &Invocation,
    cwd: &Path,
    usage: &ProofUsage,
    criteria: Option<&str>,
) -> Result<Proof> {
    let ProofUsage { command, purpose } = usage;

    let Some(raw) = inv.value("--proof") else {
        return Err(CliError::new(5, format!("proof required to {purpose}"))
            .with_hint(format!("{command} --proof test:<ci-run-ref>")));
    };
    let (kind, reference) = raw.split_once(':').ok_or_else(|| {
        CliError::new(
            5,
            format!("unreadable proof '{raw}', expected <type>:<ref>"),
        )
        .with_hint(format!("{command} --proof commit:<sha>"))
    })?;
    let proof_type = parse_proof_type(kind).ok_or_else(|| {
        CliError::new(5, format!("unknown proof type '{kind}'")).with_hint(format!(
            "{command} --proof commit|test|human-review|assertion:<ref>"
        ))
    })?;
    if reference.trim().is_empty() {
        return Err(
            CliError::new(5, format!("proof '{raw}' carries no reference"))
                .with_hint(format!("{command} --proof commit:<sha>")),
        );
    }

    // Ank validates what it can. A commit is checkable by anyone with git, so
    // it is checked here rather than trusted; an assertion anchors nothing and
    // is recorded as it stands, visible as weak to `check`.
    if proof_type == ProofType::Commit {
        let spec = format!("{}^{{commit}}", reference.trim());
        let args = ["rev-parse", "--verify", "--quiet", spec.as_str()];
        if !git::output(cwd, &args)?.status.success() {
            return Err(CliError::new(
                5,
                format!("commit {reference} not found in this repository"),
            )
            .with_hint(format!("git log --oneline -1 {reference}")));
        }
    }

    Ok(Proof {
        proof_type,
        reference: reference.trim().to_string(),
        tree: None,
        criteria: criteria.map(freeze_hash_short),
        verifier: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_core::{serialize_entity, CriteriaBy, Task};
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    struct Temp(PathBuf);

    impl Temp {
        fn new(verifiers: &str) -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-done-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(p.join(".ank/tasks")).unwrap();
            std::fs::create_dir_all(p.join(".ank/adr")).unwrap();
            let t = Temp(p);
            for args in [
                vec!["init", "-q", "-b", "main"],
                vec!["config", "user.email", "test@ank.local"],
                vec!["config", "user.name", "Test"],
                vec!["config", "core.autocrlf", "false"],
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
                format!("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n{verifiers}"),
            )
            .unwrap();
            std::fs::write(t.0.join("seed.txt"), "x").unwrap();
            t.commit();
            t
        }

        fn commit(&self) {
            for args in [
                vec!["add", "-A"],
                vec!["-c", "commit.gpgsign=false", "commit", "-qm", "seed"],
            ] {
                Command::new("git")
                    .current_dir(&self.0)
                    .args(&args)
                    .status()
                    .unwrap();
            }
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

        fn store(&self) -> Store {
            Store::new(self.0.join(".ank"))
        }

        fn seed(&self, verify: &[&str]) -> EntityId {
            let id = EntityId::parse("TASK-000000000001").unwrap();
            let task = Task {
                id: id.clone(),
                slug: Some("example".into()),
                title: "Example task".into(),
                created: "2026-07-28T00:00:00Z".into(),
                author: None,
                status: TaskStatus::Open,
                scope: vec!["src/**".into()],
                blocked_by: vec![],
                done_criteria: Some("A verifiable criterion.\n".into()),
                criteria_by: Some(CriteriaBy::Creator),
                verify: verify.iter().map(|v| v.to_string()).collect(),
                proof: vec![],
                schema: 1,
                version: 1,
                body: "\nBody.\n".into(),
            };
            std::fs::write(
                self.0.join(".ank/tasks/TASK-000000000001.md"),
                serialize_entity(&Entity::Task(task)),
            )
            .unwrap();
            id
        }

        fn claim(&self, id: &EntityId, who: &str) {
            let Entity::Task(task) = self.store().load(id).unwrap().entity else {
                panic!("not a task")
            };
            let criteria = freeze_hash_short(task.done_criteria.as_deref().unwrap_or(""));
            claim::acquire(
                &self.0,
                &task,
                who,
                Duration::from_secs(1800),
                &criteria,
                "ddddeeeeffff",
                None,
            )
            .unwrap();
            let mut moved = task;
            moved.status = TaskStatus::InProgress;
            self.store().write(&Entity::Task(moved), 1).unwrap();
        }

        fn done(&self, args: &[&str], who: &str) -> Result<String> {
            let argv: Vec<String> = std::iter::once("done".to_string())
                .chain(args.iter().map(|a| a.to_string()))
                .collect();
            let inv = crate::cli::parse(&argv).unwrap();
            let mut out = Vec::new();
            run(&inv, &self.repo(), &self.cfg(), who, &mut out)?;
            Ok(String::from_utf8_lossy(&out).to_string())
        }

        fn task(&self) -> Task {
            let id = EntityId::parse("TASK-000000000001").unwrap();
            match self.store().load(&id).unwrap().entity {
                Entity::Task(t) => t,
                _ => panic!("not a task"),
            }
        }

        fn record(&self) -> Option<Record> {
            let id = EntityId::parse("TASK-000000000001").unwrap();
            claim::read(&self.0, &id).unwrap().map(|h| h.record)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    const PASSING: &str =
        "verifiers:\n  ok-one:\n    run: echo one\n  ok-two:\n    run: echo two\n";
    const FAILING: &str = "verifiers:\n  says-no:\n    run: exit 3\n";

    #[test]
    fn every_declared_verifier_runs_and_leaves_its_own_proof() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one", "ok-two"]);
        t.claim(&id, "claude-code@ank");

        let out = t.done(&[], "claude-code@ank").unwrap();
        // The `running:` lines are no longer here to be asserted: they are
        // progress, they go to standard error, and stdout under `--json` is a
        // parser's input (TASK-2eefcdd80124). What proves the verifiers ran is
        // the proof list below; that they are *reported*, and on which stream,
        // is asserted through the binary in `tests/cli.rs`.
        assert!(
            !out.contains("running:"),
            "progress reached stdout, where a JSON document also goes: {out}"
        );

        let task = t.task();
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(
            task.proof.len(),
            2,
            "one entry per verifier: {:?}",
            task.proof
        );
        let expected = freeze_hash_short("A verifiable criterion.\n");
        for p in &task.proof {
            assert_eq!(p.proof_type, ProofType::Test);
            assert!(p.reference.starts_with("local/"), "{:?}", p.reference);
            // The definition that ran is anchored, not the current config.
            let v = p.verifier.as_ref().expect("the definition is anchored");
            assert!(v.contains('@'), "{v}");
            assert_eq!(p.criteria.as_deref(), Some(expected.as_str()));
            assert!(p.tree.as_deref().unwrap_or("").starts_with("scope/"));
        }
        assert_ne!(
            task.proof[0].verifier, task.proof[1].verifier,
            "two verifiers, two definitions"
        );
    }

    #[test]
    fn a_task_declaring_verifiers_refuses_proof_outright() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one"]);
        t.claim(&id, "claude-code@ank");

        let err = t
            .done(&["--proof", "assertion:it works"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 5, "{}", err.message);
        assert!(err.message.contains("ok-one"), "{}", err.message);
        assert_eq!(
            t.task().status,
            TaskStatus::InProgress,
            "nothing was written"
        );
    }

    #[test]
    fn a_failing_verifier_refuses_the_transition_and_leaves_the_claim_intact() {
        let t = Temp::new(FAILING);
        let id = t.seed(&["says-no"]);
        t.claim(&id, "claude-code@ank");

        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(
            err.code, 5,
            "a verifier that ran and said no: {}",
            err.message
        );

        // The file did not move, and the ref still carries a claim -- not a
        // completion. Transforming it here would mark finished a task that is
        // not, and nobody could pick it up again.
        let task = t.task();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.proof.is_empty());
        match t.record().expect("the ref survives a failed done") {
            Record::Claim(c) => assert_eq!(c.holder, "claude-code@ank"),
            other => panic!("expected the claim to be intact, got {other:?}"),
        }
    }

    #[test]
    fn a_verifier_absent_from_the_config_is_the_environment_not_a_failure() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["no-such-verifier"]);
        t.claim(&id, "claude-code@ank");

        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 9, "{}", err.message);
        assert!(err.message.contains("no-such-verifier"), "{}", err.message);
        assert_eq!(t.task().status, TaskStatus::InProgress);
    }

    #[test]
    fn the_frozen_criterion_is_checked_before_a_single_verifier_runs() {
        // A verifier that leaves a trace if it runs, so the assertion is about
        // ordering and not merely about the error.
        let marker = std::env::temp_dir().join(format!("ank-ran-{}.marker", std::process::id()));
        let _ = std::fs::remove_file(&marker);
        let t = Temp::new(&format!(
            "verifiers:\n  touches:\n    run: \"touch '{}'\"\n",
            marker.display().to_string().replace('\\', "/")
        ));
        let id = t.seed(&["touches"]);
        t.claim(&id, "claude-code@ank");

        // The criterion is weakened after the claim, which is the case the
        // anchor exists for.
        let mut task = t.task();
        task.done_criteria = Some("Anything at all.\n".into());
        t.store().write(&Entity::Task(task), 2).unwrap();

        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 6, "a diverged freeze is code 6: {}", err.message);
        assert!(
            err.message.contains("changed since the claim"),
            "{}",
            err.message
        );
        assert!(
            !marker.exists(),
            "no verifier may run once the anchor diverges"
        );
        assert_eq!(t.task().status, TaskStatus::InProgress);
    }

    // -----------------------------------------------------------------------
    // Without verifiers: --proof
    // -----------------------------------------------------------------------

    #[test]
    fn without_verifiers_proof_is_mandatory_and_names_the_flag() {
        let t = Temp::new("");
        let id = t.seed(&[]);
        t.claim(&id, "claude-code@ank");

        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 5);
        assert!(err.message.contains("proof required"), "{}", err.message);
        assert!(err.hint.unwrap().contains("--proof"));
    }

    #[test]
    fn a_commit_proof_is_checked_and_an_assertion_is_taken_as_it_stands() {
        let t = Temp::new("");
        let id = t.seed(&[]);
        t.claim(&id, "claude-code@ank");
        let head = git::run(&t.0, &["rev-parse", "HEAD"]).unwrap();

        // A commit that does not exist is refused: anyone with git can check
        // this one, so it is checked rather than trusted.
        let err = t
            .done(
                &["--proof", "commit:0000000000000000000000000000000000000000"],
                "claude-code@ank",
            )
            .unwrap_err();
        assert_eq!(err.code, 5, "{}", err.message);
        assert_eq!(t.task().status, TaskStatus::InProgress);

        t.done(&["--proof", &format!("commit:{head}")], "claude-code@ank")
            .unwrap();
        let task = t.task();
        assert_eq!(task.status, TaskStatus::Done);
        assert_eq!(task.proof.len(), 1);
        assert_eq!(task.proof[0].proof_type, ProofType::Commit);
        assert!(
            task.proof[0].verifier.is_none(),
            "nothing ran, so nothing is anchored"
        );
    }

    #[test]
    fn a_malformed_proof_is_named_precisely() {
        let t = Temp::new("");
        let id = t.seed(&[]);
        t.claim(&id, "claude-code@ank");
        for (arg, needle) in [
            ("no-colon-at-all", "expected <type>:<ref>"),
            ("nonsense:x", "unknown proof type"),
            ("assertion:", "carries no reference"),
        ] {
            let err = t.done(&["--proof", arg], "claude-code@ank").unwrap_err();
            assert_eq!(err.code, 5, "{arg}: {}", err.message);
            assert!(err.message.contains(needle), "{arg}: {}", err.message);
            assert!(err.hint.is_some(), "{arg}");
        }
    }

    /// Through both verbs, on the same repository, in the same test.
    ///
    /// `attest` carried a copy of this grammar, and the failure mode was never
    /// the duplication itself -- it was the drift. Two verbs disagreeing about
    /// what a proof *is* would be a worse defect than the copy, and asserting
    /// the agreement by inspection is how it goes unnoticed. So the two are run
    /// against the same inputs and compared, which is a test that fails if
    /// anyone reintroduces a second parser.
    #[test]
    fn done_and_attest_refuse_a_malformed_proof_identically() {
        let t = Temp::new("");
        let id = t.seed(&[]);
        t.claim(&id, "claude-code@ank");

        let attest = |arg: &str| {
            let argv: Vec<String> = ["attest", &id.to_string(), "--proof", arg]
                .iter()
                .map(|a| a.to_string())
                .collect();
            let inv = crate::cli::parse(&argv).unwrap();
            let mut out = Vec::new();
            crate::human::attest(&inv, &t.repo(), "marie@laptop", &mut out).unwrap_err()
        };

        // Finished first, because `attest` applies to a done task and refusing
        // an unfinished one would answer before the proof is ever parsed.
        t.done(
            &["--proof", "assertion:reviewed by hand"],
            "claude-code@ank",
        )
        .unwrap();

        // The last one reaches the git check rather than the grammar: a commit
        // is verifiable by anyone with git, so both verbs check it rather than
        // trust it, and both have to say the same thing when it is not there.
        for arg in [
            "no-colon-at-all",
            "nonsense:x",
            "assertion:",
            "commit:",
            "commit:deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        ] {
            let by_done = {
                // The task is finished now, so `done` refuses it before parsing.
                // A second repository, at the same point of the grammar.
                let u = Temp::new("");
                let i = u.seed(&[]);
                u.claim(&i, "claude-code@ank");
                u.done(&["--proof", arg], "claude-code@ank").unwrap_err()
            };
            let by_attest = attest(arg);

            assert_eq!(
                by_done.code, by_attest.code,
                "{arg}: done says {} and attest says {}",
                by_done.code, by_attest.code
            );
            assert_eq!(
                by_done.message, by_attest.message,
                "{arg}: the diagnosis is the grammar's, not the verb's"
            );

            // The hint is where the two are allowed to differ, and only there:
            // §4 makes it the exact command to run, and that command names the
            // verb. Strip each caller's own prefix and what remains must match.
            //
            // Not every hint carries one. A commit that is not in the
            // repository points at `git log`, which is the same next command
            // whoever asked -- so both strip to themselves and still agree.
            let done_hint = by_done.hint.expect("done names a next command");
            let attest_hint = by_attest.hint.expect("attest names a next command");
            assert_eq!(
                done_hint.trim_start_matches("ank done "),
                attest_hint.trim_start_matches(&format!("ank attest {id} ") as &str),
                "{arg}: the hints differ only by who is speaking\n  done:   {done_hint}\n  attest: {attest_hint}"
            );
            if done_hint.starts_with("ank ") {
                assert!(done_hint.starts_with("ank done "), "{arg}: {done_hint}");
                assert!(
                    attest_hint.starts_with(&format!("ank attest {id} ")),
                    "{arg}: {attest_hint}"
                );
            }
        }

        // And the missing-proof case, where the message names the purpose and
        // is therefore the one place the two are allowed to read differently.
        let missing = attest("");
        assert_eq!(missing.code, 5);
    }

    // -----------------------------------------------------------------------
    // The ref, HEAD, and the log
    // -----------------------------------------------------------------------

    #[test]
    fn a_successful_done_turns_the_claim_into_a_completion_without_a_ttl() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one"]);
        t.claim(&id, "claude-code@ank");
        let head = git::run(&t.0, &["rev-parse", "HEAD"]).unwrap();

        t.done(&[], "claude-code@ank").unwrap();

        match t.record().expect("the ref is transformed, never deleted") {
            Record::Completed(c) => {
                assert_eq!(c.commit, head);
                assert_eq!(c.branch.as_deref(), Some("main"));
                assert_eq!(c.identity, "claude-code@ank");
                assert!(!c.completed.is_empty());
            }
            other => panic!("expected a completion record, got {other:?}"),
        }
    }

    #[test]
    fn done_writes_one_log_entry_naming_its_proof() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one"]);
        t.claim(&id, "claude-code@ank");
        t.done(&[], "claude-code@ank").unwrap();

        let entries = ank_core::parse_log(&t.task().body);
        assert_eq!(entries.len(), 1, "{entries:?}");
        assert_eq!(entries[0].who, "claude-code@ank");
        assert!(
            entries[0].message.starts_with("done, proof "),
            "{:?}",
            entries[0]
        );
    }

    #[test]
    fn done_without_a_claim_is_refused_and_never_touches_another_agents_task() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one"]);

        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert!(
            err.message.contains("no task in progress"),
            "{}",
            err.message
        );

        // Held by somebody else: still not this agent's to finish.
        t.claim(&id, "codex@host-9");
        let err = t.done(&[], "claude-code@ank").unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert_eq!(t.task().status, TaskStatus::InProgress);
    }

    #[test]
    fn an_explicit_id_must_match_head() {
        let t = Temp::new(PASSING);
        let id = t.seed(&["ok-one"]);
        t.claim(&id, "claude-code@ank");

        // A second task, not the one in progress.
        std::fs::copy(
            t.0.join(".ank/tasks/TASK-000000000001.md"),
            t.0.join(".ank/tasks/TASK-00000000ffff.md"),
        )
        .unwrap();
        let other = t.0.join(".ank/tasks/TASK-00000000ffff.md");
        let text = std::fs::read_to_string(&other)
            .unwrap()
            .replace("TASK-000000000001", "TASK-00000000ffff");
        std::fs::write(&other, text).unwrap();

        let err = t
            .done(&["TASK-00000000ffff"], "claude-code@ank")
            .unwrap_err();
        assert_eq!(err.code, 6, "{}", err.message);
        assert!(
            err.message.contains("not the task in progress"),
            "{}",
            err.message
        );

        // The redundant but matching form goes through.
        t.done(&["TASK-000000000001"], "claude-code@ank").unwrap();
        assert_eq!(t.task().status, TaskStatus::Done);
    }

    #[test]
    fn the_scope_hash_captures_the_files_and_not_the_commit() {
        let t = Temp::new("");
        std::fs::create_dir_all(t.0.join("src")).unwrap();
        std::fs::write(t.0.join("src/a.rs"), "fn a() {}").unwrap();
        let globs = vec!["src/**".to_string()];

        let first = scope_hash(&t.0, &globs).unwrap();
        assert!(first.starts_with("scope/"), "{first}");
        assert_eq!(scope_hash(&t.0, &globs).unwrap(), first, "stable");

        // The working tree changes with no commit: this is what a local proof
        // anchors, and the HEAD sha alone would not have moved.
        std::fs::write(t.0.join("src/a.rs"), "fn a() { todo!() }").unwrap();
        assert_ne!(scope_hash(&t.0, &globs).unwrap(), first);

        // A file outside the scope does not move it.
        let second = scope_hash(&t.0, &globs).unwrap();
        std::fs::write(t.0.join("elsewhere.txt"), "noise").unwrap();
        assert_eq!(scope_hash(&t.0, &globs).unwrap(), second);
    }
}
