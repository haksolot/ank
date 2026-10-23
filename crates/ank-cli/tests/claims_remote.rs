//! A remote named `ank` carries `refs/ank/claims/*`, and everything else stays
//! on origin (ADR-b6fe84504ff2, TASK-894b2c054049).
//!
//! Through the binary, over bare remotes reached by `file://`, and counted with
//! `GIT_TRACE2_EVENT` at an absolute path (ADR-cc65f1388a71): one `start` per
//! process, written by git itself. Only the processes ank starts are counted --
//! a trace2 session id with no `/` -- because a push over `file://` starts a
//! `receive-pack` and its own children, which are the remote's work and not
//! the verb's.
//!
//! The origin-only counts are literals measured on the binary before this
//! change (5e2d826a): claim 12, log 8 once its renewal travels, release 8,
//! done 14, attest 11, accept 11, status --remote --json 25. They are what "identical
//! to the binary before the change" is held to, together with every network
//! invocation, argument for argument.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "tool/1.0";
const HUMAN: &str = "human:marie";
const T: &str = "TASK-000000000001";
const A: &str = "ADR-00000000000a";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("claims-remote-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [protocol \"file\"]\n\tallow = always\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn spawn(program: &str) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// Returns once the wall clock is in a later second than it was on entry, so
/// the renewal `log` makes stamps an expiry the ref does not already carry and
/// therefore travels.
fn cross_a_second() {
    let now = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };
    let start = now();
    while now() == start {
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// One git process the binary started, as its arguments after `git`.
type Call = Vec<String>;

/// The processes a trace2 event file records that the binary itself started.
fn calls(trace: &Path) -> Vec<Call> {
    let text = fs::read_to_string(trace).unwrap_or_default();
    text.lines()
        .filter(|l| l.contains("\"event\":\"start\""))
        .filter(|l| {
            let sid = l.split("\"sid\":\"").nth(1).unwrap_or("");
            !sid.split('"').next().unwrap_or("").contains('/')
        })
        .map(|l| {
            let argv = l.split("\"argv\":[").nth(1).unwrap_or("");
            let argv = argv.split("\"]").next().unwrap_or("");
            argv.trim_start_matches('"')
                .split("\",\"")
                .skip(1)
                .map(str::to_string)
                .collect()
        })
        .collect()
}

/// The calls that cross to a remote: every push, ls-remote and fetch.
fn network(calls: &[Call]) -> Vec<&Call> {
    calls
        .iter()
        .filter(|c| {
            matches!(
                c.first().map(String::as_str),
                Some("push" | "ls-remote" | "fetch")
            )
        })
        .collect()
}

/// The remote a network call names, `origin` or `ank`.
fn remote_of(call: &Call) -> &str {
    call.iter()
        .map(String::as_str)
        .find(|a| *a == "origin" || *a == "ank")
        .unwrap_or_else(|| panic!("no remote named in {call:?}"))
}

/// The `refs/ank/...` a network call touches, its pattern or the ref of its
/// refspec.
fn ref_of(call: &Call) -> String {
    let arg = call
        .iter()
        .rev()
        .find(|a| a.contains("refs/ank/"))
        .unwrap_or_else(|| panic!("no ank ref in {call:?}"));
    let arg = arg.rsplit(':').next().unwrap();
    arg.trim_start_matches('+').to_string()
}

/// Every network call, as `<verb> <remote> <ref>`, the shape each assertion
/// below compares against a literal.
fn shapes(calls: &[Call]) -> Vec<String> {
    network(calls)
        .iter()
        .map(|c| format!("{} {} {}", c[0], remote_of(c), ref_of(c)))
        .collect()
}

/// A repository holding one open task and one proposed ADR, with bare remotes
/// named by `remotes`, each holding `main`.
struct Clone {
    base: PathBuf,
    repo: PathBuf,
    head: String,
}

impl Clone {
    fn new(what: &str, remotes: &[&str]) -> Clone {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            repo.join(".ank/entities").join(format!("{T}.md")),
            format!(
                "---\nid: {T}\ntype: task\nslug: example\ntitle: The task in hand\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
        fs::write(
            repo.join(".ank/entities").join(format!("{A}.md")),
            format!(
                "---\nid: {A}\ntype: adr\nslug: a-decision\ntitle: Decision {A}\n\
                 created: 2026-08-01T00:00:00Z\nauthor: {HUMAN}\nstatus: proposed\n\
                 scope:\n  - src/**\nconstraint: |\n  The rule binds.\nschema: 4\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);
        for name in remotes {
            let bare = base.join(format!("{name}.git"));
            fs::create_dir_all(&bare).unwrap();
            git(&bare, &["init", "-q", "--bare", "-b", "main"]);
            git(&repo, &["remote", "add", name, &file_url(&bare)]);
            git(&repo, &["push", "-q", name, "main"]);
        }
        if remotes.contains(&"origin") {
            git(
                &repo,
                &[
                    "config",
                    "--add",
                    "remote.origin.fetch",
                    "+refs/ank/*:refs/ank/*",
                ],
            );
        }
        let head = git(&repo, &["rev-parse", "HEAD"]);
        Clone { base, repo, head }
    }

    /// Runs the binary with a trace of its own, asserts exit 0, and returns
    /// the output and the git processes it started.
    fn traced(&self, step: &str, agent: &str, args: &[&str]) -> (Output, Vec<Call>) {
        let trace = self.base.join(format!("trace-{step}"));
        let _ = fs::remove_file(&trace);
        let out = spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.repo)
            .env("ANK_AGENT", agent)
            .env("GIT_TRACE2_EVENT", &trace)
            .current_dir(scratch::root())
            .output()
            .expect("the binary must have been built");
        assert_eq!(
            out.status.code(),
            Some(0),
            "{args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        (out, calls(&trace))
    }

    /// The refs a bare remote of this clone holds under `refs/ank/`.
    fn held_by(&self, remote: &str) -> Vec<String> {
        let bare = self.base.join(format!("{remote}.git"));
        git(&bare, &["for-each-ref", "--format=%(refname)", "refs/ank/"])
            .lines()
            .map(str::to_string)
            .collect()
    }

    /// claim, log, release, claim again, done, attest --detached and accept,
    /// in that order, each with its network calls.
    fn every_verb(&self) -> Vec<(&'static str, usize, Vec<String>)> {
        let mut seen = Vec::new();
        let mut step = |name: &'static str, agent: &str, args: &[&str]| {
            let (_, calls) = self.traced(name, agent, args);
            seen.push((name, calls.len(), shapes(&calls)));
        };
        step("claim", AGENT, &["claim", T]);
        cross_a_second();
        step("log", AGENT, &["log", "a note"]);
        step("release", AGENT, &["release", "--reason", "handing back"]);
        step("claim2", AGENT, &["claim", T]);
        step(
            "done",
            AGENT,
            &["done", "--proof", &format!("commit:{}", self.head)],
        );
        step(
            "attest",
            "process:ci",
            &["attest", T, "--proof", "test:ci-1", "--detached"],
        );
        step("accept", HUMAN, &["accept", A]);
        seen
    }
}

fn claims() -> String {
    format!("refs/ank/claims/{T}")
}

/// The criterion's first clause, and the routing it states as a whole.
#[test]
fn with_origin_and_ank_claims_travel_on_ank_and_the_rest_on_origin() {
    let c = Clone::new("claims-remote-both", &["origin", "ank"]);
    let claims = claims();
    let proof = format!("refs/ank/proof/{T}");
    let ratify = format!("refs/ank/ratify/{A}");
    let seen = c.every_verb();
    let expected: [(&str, Vec<String>); 7] = [
        (
            "claim",
            vec![
                format!("ls-remote ank {claims}"),
                format!("push ank {claims}"),
            ],
        ),
        ("log", vec![format!("push ank {claims}")]),
        ("release", vec![format!("push ank {claims}")]),
        (
            "claim2",
            vec![
                format!("ls-remote ank {claims}"),
                format!("push ank {claims}"),
            ],
        ),
        ("done", vec![format!("push ank {claims}")]),
        (
            "attest",
            vec![
                format!("ls-remote origin {proof}"),
                format!("push origin {proof}"),
            ],
        ),
        ("accept", vec![format!("push origin {ratify}")]),
    ];
    for ((name, _, got), (want_name, want)) in seen.iter().zip(expected.iter()) {
        assert_eq!(name, want_name);
        assert_eq!(got, want, "{name}: the network calls with origin and ank");
    }

    assert_eq!(
        c.held_by("ank"),
        vec![claims.clone()],
        "ank carries the claim"
    );
    let origin = c.held_by("origin");
    assert!(
        !origin.contains(&claims),
        "origin holds no claim: {origin:?}"
    );
    assert!(
        origin.contains(&proof) && origin.contains(&ratify),
        "origin carries the proof and the ratification: {origin:?}"
    );
    assert!(
        !c.held_by("ank").contains(&proof) && !c.held_by("ank").contains(&ratify),
        "ank carries nothing but claims"
    );
}

/// `status --remote` reads claims off ank, and status names ank as the
/// coordination remote, in text and in `--json`.
#[test]
fn with_ank_status_reads_claims_there_and_names_it() {
    let c = Clone::new("claims-remote-status", &["origin", "ank"]);
    c.traced("claim", AGENT, &["claim", T]);

    let (out, calls) = c.traced(
        "status-remote",
        "other/1.0",
        &["status", "--remote", "--json"],
    );
    let json = text(&out.stdout);
    let net = shapes(&calls);
    assert!(
        net.contains(&"ls-remote ank refs/ank/claims/*".to_string()),
        "status --remote reads claims off ank: {net:?}"
    );
    assert!(
        !net.iter()
            .any(|s| s.starts_with("ls-remote origin refs/ank/claims")),
        "and never off origin: {net:?}"
    );
    assert!(json.contains("\"remote\":true"), "{json}");
    assert!(
        json.contains(&format!("\"id\":\"{T}\"")) && json.contains("\"seen\":\"both\""),
        "the claim ank carries is seen on the remote: {json}"
    );
    assert!(
        json.contains("\"coordination_remote\":\"ank\""),
        "--json names the coordination remote: {json}"
    );

    let (out, _) = c.traced("status-text", AGENT, &["status"]);
    let said = text(&out.stdout);
    assert!(
        said.lines()
            .any(|l| l.starts_with("coordination ") && l.contains("ank")),
        "status names ank as the coordination remote:\n{said}"
    );
}

/// With origin alone, nothing changes: the same processes, the same network
/// calls, and no coordination remote named.
#[test]
fn with_origin_only_the_verbs_run_what_they_ran_before() {
    let c = Clone::new("claims-remote-origin", &["origin"]);
    let claims = claims();
    let proof = format!("refs/ank/proof/{T}");
    let ratify = format!("refs/ank/ratify/{A}");
    let seen = c.every_verb();
    // Measured on 5e2d826a. `log` ran 4 there in the claim's own second, and
    // 8 once a second has passed and its renewal travels.
    let expected: [(&str, usize, Vec<String>); 7] = [
        (
            "claim",
            12,
            vec![
                format!("ls-remote origin {claims}"),
                format!("push origin {claims}"),
            ],
        ),
        ("log", 8, vec![format!("push origin {claims}")]),
        ("release", 8, vec![format!("push origin {claims}")]),
        (
            "claim2",
            12,
            vec![
                format!("ls-remote origin {claims}"),
                format!("push origin {claims}"),
            ],
        ),
        ("done", 14, vec![format!("push origin {claims}")]),
        (
            "attest",
            11,
            vec![
                format!("ls-remote origin {proof}"),
                format!("push origin {proof}"),
            ],
        ),
        ("accept", 11, vec![format!("push origin {ratify}")]),
    ];
    for ((name, count, got), (want_name, want_count, want)) in seen.iter().zip(expected.iter()) {
        assert_eq!(name, want_name);
        assert_eq!(got, want, "{name}: the network calls with origin alone");
        assert_eq!(
            count, want_count,
            "{name}: the git processes with origin alone"
        );
    }

    let (out, calls) = c.traced("status-remote", AGENT, &["status", "--remote", "--json"]);
    assert_eq!(calls.len(), 25, "status --remote --json: the git processes");
    assert_eq!(
        shapes(&calls),
        vec!["ls-remote origin refs/ank/*".to_string()]
    );
    let json = text(&out.stdout);
    assert!(json.contains("\"coordination_remote\":null"), "{json}");
    let (out, _) = c.traced("status-text", AGENT, &["status"]);
    let said = text(&out.stdout);
    assert!(
        !said.lines().any(|l| l.starts_with("coordination ")),
        "no coordination remote is named with origin alone:\n{said}"
    );
}

/// With ank and no origin, claims are arbitrated through ank and every other
/// ref stays local.
#[test]
fn with_ank_only_claim_pushes_to_ank_and_accept_pushes_nothing() {
    let c = Clone::new("claims-remote-ank", &["ank"]);
    let claims = claims();
    let (_, calls) = c.traced("claim", AGENT, &["claim", T]);
    assert_eq!(
        shapes(&calls),
        vec![
            format!("ls-remote ank {claims}"),
            format!("push ank {claims}")
        ]
    );
    assert_eq!(c.held_by("ank"), vec![claims]);

    let (_, calls) = c.traced("accept", HUMAN, &["accept", A]);
    assert_eq!(
        shapes(&calls),
        Vec::<String>::new(),
        "accept pushes nothing"
    );
    assert_eq!(
        git(
            &c.repo,
            &["for-each-ref", "--format=%(refname)", "refs/ank/ratify/"]
        ),
        format!("refs/ank/ratify/{A}"),
        "the ratification stays local, as at level 0"
    );
    assert_eq!(
        c.held_by("ank").len(),
        1,
        "ank carries the claim and nothing else"
    );

    let (out, _) = c.traced("status-text", AGENT, &["status"]);
    let said = text(&out.stdout);
    assert!(
        said.lines()
            .any(|l| l.starts_with("coordination ank ")
                && l.ends_with("everything else stays local")),
        "with no origin, the rest is named local:\n{said}"
    );
}
