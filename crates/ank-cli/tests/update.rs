//! `update`: the one verb that asks whether a newer release exists
//! (ADR-64f32c74a0f9, §4).
//!
//! **Through the binary, against a bare repository standing in for the release
//! repository.** The criterion is a statement about a process: what it prints,
//! what it exits with, which processes it starts. `ANK_UPDATE_REPOSITORY` is the
//! seam that points the verb at the stand-in, and it is the seam a mirror uses
//! in production, which is why the verb's help names it.
//!
//! Process counts are read from `GIT_TRACE` at an absolute path and never from
//! a wall clock (CLAUDE.md): a process that did not start leaves no line, on
//! every platform, however loaded the runner is.

mod fixture;
mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The version the binary was built with, which is what `update` reports as
/// running and what `ank --version` prints first.
const RUNNING: &str = env!("CARGO_PKG_VERSION");

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// The route executables `update` could start and must not under `--check`.
const ROUTES: &[&str] = &["npm", "npx", "sh", "curl", "powershell", "pwsh"];

/// git's global and system configuration for every process this suite spawns,
/// so a machine that signs commits by default cannot decide whether a fixture
/// commits.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("update-it-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A bare repository holding one commit and every tag in `tags` pointing at it.
fn release_repository(what: &str, tags: &[&str]) -> PathBuf {
    let bare = scratch::dir(what);
    git(&bare, &["init", "--bare", "-q"]);
    let tree = git(&bare, &["hash-object", "-t", "tree", "-w", "--stdin"]);
    let commit = git(
        &bare,
        &["commit-tree", "--no-gpg-sign", "-m", "release", &tree],
    );
    for tag in tags {
        git(&bare, &["update-ref", &format!("refs/tags/{tag}"), &commit]);
    }
    bare
}

/// A directory of stub route executables, each appending its own name to
/// `$ANK_STUB_RECORD` when started.
fn stubs(what: &str) -> PathBuf {
    let dir = scratch::dir(what);
    for name in ROUTES {
        if cfg!(windows) {
            fs::write(
                dir.join(format!("{name}.cmd")),
                format!("@echo off\r\n>> \"%ANK_STUB_RECORD%\" echo({name}\r\nexit /b 42\r\n"),
            )
            .unwrap();
        } else {
            let path = dir.join(name);
            fs::write(
                &path,
                format!("#!/bin/sh\necho {name} >> \"$ANK_STUB_RECORD\"\nexit 42\n"),
            )
            .unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
    }
    dir
}

/// The stubs first on `PATH`, then whatever found git for this suite.
fn path_with(stubs: &Path) -> std::ffi::OsString {
    let mut dirs = vec![stubs.to_path_buf()];
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(dirs).unwrap()
}

struct Run {
    out: Output,
    trace: String,
    record: Option<String>,
}

impl Run {
    fn code(&self) -> Option<i32> {
        self.out.status.code()
    }
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.out.stdout).to_string()
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.out.stderr).to_string()
    }
    /// Every git command this run started, as its argument list.
    fn gits(&self) -> Vec<String> {
        self.trace
            .lines()
            .filter_map(|l| l.split_once(MARK).map(|(_, rest)| rest.to_string()))
            .collect()
    }
}

/// `ank <args>` from `dir`, with the route stubs on `PATH`, `GIT_TRACE` at an
/// absolute path, and the release repository named by the seam.
fn ank_in(dir: &Path, repository: &str, args: &[&str]) -> Run {
    let bin = stubs("update-stubs");
    let record = scratch::dir("update-record").join("record");
    let trace = scratch::dir("update-trace").join("trace");
    let out = Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("PATH", path_with(&bin))
        .env("ANK_STUB_RECORD", &record)
        .env("ANK_UPDATE_REPOSITORY", repository)
        .env("GIT_TRACE", &trace)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .env("ANK_AGENT", "claude-code/update-it")
        .stdin(Stdio::null())
        .output()
        .expect("the binary must have been built");
    Run {
        out,
        trace: fs::read_to_string(&trace).unwrap_or_default(),
        record: fs::read_to_string(&record).ok(),
    }
}

fn check(repository: &Path, json: bool) -> Run {
    let empty = scratch::dir("update-cwd");
    let mut args = vec!["update", "--check"];
    if json {
        args.push("--json");
    }
    ank_in(&empty, &repository.to_string_lossy(), &args)
}

/// The latest release is the highest tag of the exact form vMAJOR.MINOR.PATCH,
/// compared per component as numbers, and every other tag is ignored however
/// high it reads.
const NEWER_TAGS: &[&str] = &[
    "v9000.9.0",
    "v9000.10.0",
    "v9999.0.0-rc.1",
    "v99999.0",
    "99999.0.0",
    "v99999.0.0.0",
    "latest",
    "release-99999.0.0",
];

#[test]
fn check_names_the_running_version_and_a_newer_release() {
    let bare = release_repository("update-newer", NEWER_TAGS);
    let run = check(&bare, false);
    assert_eq!(
        run.code(),
        Some(0),
        "a newer release is an answer, not a finding:\n{}{}",
        run.stdout(),
        run.stderr()
    );
    let stdout = run.stdout();
    let lines: Vec<&str> = stdout.lines().map(str::trim_end).collect();
    assert!(
        lines
            .iter()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["running", RUNNING]),
        "the running version is not printed:\n{stdout}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["latest", "9000.10.0"]),
        "the latest release is not v9000.10.0:\n{stdout}"
    );
    assert!(
        stdout.contains("a newer release exists"),
        "it does not say a newer release exists:\n{stdout}"
    );
}

#[test]
fn check_says_up_to_date_when_the_running_version_is_the_latest() {
    let tag = format!("v{RUNNING}");
    let bare = release_repository("update-current", &["v0.0.1", &tag, "v99999.0.0-beta"]);
    let run = check(&bare, false);
    assert_eq!(run.code(), Some(0), "{}{}", run.stdout(), run.stderr());
    let stdout = run.stdout();
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["latest", RUNNING]),
        "the latest release is not the running one:\n{stdout}"
    );
    assert!(stdout.contains("up to date"), "{stdout}");
    assert!(!stdout.contains("a newer release exists"), "{stdout}");
}

#[test]
fn check_json_carries_current_latest_and_newer() {
    let bare = release_repository("update-json-newer", NEWER_TAGS);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let doc = document(&run);
    assert_eq!(doc["current"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["latest"].as_str(), Some("9000.10.0"), "{doc:?}");
    assert_eq!(doc["newer"].as_bool(), Some(true), "{doc:?}");

    let tag = format!("v{RUNNING}");
    let bare = release_repository("update-json-current", &[&tag]);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let doc = document(&run);
    assert_eq!(doc["current"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["latest"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["newer"].as_bool(), Some(false), "{doc:?}");
}

/// The `--json` document on stdout, which must be the only thing there.
fn document(run: &Run) -> serde_yaml::Value {
    let stdout = run.stdout();
    assert_eq!(
        stdout.trim().lines().count(),
        1,
        "--json printed more than one document:\n{stdout}"
    );
    serde_yaml::from_str(&stdout).unwrap_or_else(|e| panic!("not a document ({e}):\n{stdout}"))
}

/// **It starts no process but git**, and of git exactly one ls-remote: the
/// route stubs record nothing, and the trace names no other git ank started.
/// `upload-pack` is the far end of a local ls-remote, started by git.
#[test]
fn check_starts_one_git_ls_remote_and_nothing_else() {
    let bare = release_repository("update-processes", NEWER_TAGS);
    let run = check(&bare, false);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert_eq!(
        run.record, None,
        "--check started a route executable, which installs"
    );
    let gits = run.gits();
    let started: Vec<&String> = gits
        .iter()
        .filter(|g| !g.starts_with("upload-pack"))
        .collect();
    assert_eq!(
        started.len(),
        1,
        "ank started other git processes than one ls-remote: {gits:#?}"
    );
    assert!(
        started[0].starts_with("ls-remote --tags --refs "),
        "the one git is not ls-remote --tags --refs: {gits:#?}"
    );
    assert!(
        started[0].contains(&*bare.to_string_lossy()),
        "ls-remote did not ask the repository it was pointed at: {gits:#?}"
    );
}

#[test]
fn an_unreachable_repository_exits_9_naming_it() {
    let missing = scratch::path("update-no-such-repository");
    let run = check(&missing, false);
    assert_eq!(run.code(), Some(9), "{}{}", run.stdout(), run.stderr());
    assert!(
        run.stderr().contains(&*missing.to_string_lossy()),
        "the refusal does not name the repository it asked:\n{}",
        run.stderr()
    );
    assert_eq!(run.record, None);
}

/// **No other verb starts git ls-remote.** Counted over a run of `status` and
/// one of `context` in a corpus whose origin is a reachable repository holding
/// releases, so a verb that wanted to ask could have.
#[test]
fn status_and_context_start_no_ls_remote() {
    let bare = release_repository("update-origin", NEWER_TAGS);
    let tree = scratch::dir("update-corpus");
    git(&tree, &["init", "-q"]);
    git(&tree, &["commit", "-q", "--allow-empty", "-m", "root"]);
    git(&tree, &["remote", "add", "origin", &bare.to_string_lossy()]);
    let init = ank_in(&tree, &bare.to_string_lossy(), &["init"]);
    assert_eq!(init.code(), Some(0), "{}{}", init.stdout(), init.stderr());

    for verb in ["status", "context"] {
        let run = ank_in(&tree, &bare.to_string_lossy(), &[verb]);
        assert_eq!(run.code(), Some(0), "{verb}: {}", run.stderr());
        let gits = run.gits();
        assert!(
            !gits.is_empty(),
            "{verb} left no trace, so the count below would prove nothing"
        );
        let asked: Vec<&String> = gits.iter().filter(|g| g.starts_with("ls-remote")).collect();
        assert!(asked.is_empty(), "{verb} started git ls-remote: {asked:#?}");
        assert_eq!(run.record, None, "{verb} started a route executable");
    }
}

/// The seam is named where a caller reads the verb: a mirror is a real use.
#[test]
fn help_names_the_flags_and_the_repository_seam() {
    let out = Command::new(ANK)
        .args(["help", "update"])
        .output()
        .expect("the binary must have been built");
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{text}");
    for needle in ["--check", "--version", "ANK_UPDATE_REPOSITORY"] {
        assert!(
            text.contains(needle),
            "help update does not name {needle}:\n{text}"
        );
    }
}

/// **The document is pinned by a golden** (ADR-6fd69efb629c), captured from the
/// process. The running version is masked the way `tui.rs` masks what it knows
/// is volatile: it moves at every release and the shape does not, and the test
/// knows the value because the build told it.
#[test]
fn the_check_document_is_pinned_by_a_golden() {
    let bare = release_repository("update-golden", NEWER_TAGS);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let masked = run.stdout().replace(
        &format!("\"current\":\"{RUNNING}\""),
        "\"current\":\"<VERSION>\"",
    );
    assert_ne!(
        masked,
        run.stdout(),
        "the running version was not in the document"
    );
    fixture::pin("update-check", &masked);
}
