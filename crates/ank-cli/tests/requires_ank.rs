//! An ank older than the corpus's `requires_ank` warns on every verb and is a
//! fault in `check` (TASK-c79bde03e102, ADR-756e8b0fe416).
//!
//! **Through the binary**, because the criterion is about what a reader is
//! handed on stderr and what a parser reads on stdout, and `ank-cli` has no
//! library target a unit test could reach instead.
//!
//! **The running version is read from `ank --version`**, the line `ank update`
//! compares too, and never typed here: a release bumps it, and a requirement
//! written against a literal would stop being above the binary the day it
//! caught up.
//!
//! Every corpus is a git repository with no remote, so `check` runs its
//! coordination half and a verb that wanted the network for this would have had
//! a repository to ask. Every run carries `GIT_TRACE` at an absolute path, and
//! the trace is read for `ls-remote` after it.

use crate::fixture;
use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// A git configuration that signs nothing, so a scratch commit needs no key.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("requires-ank-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

/// The version `ank --version` prints first, as its three numbers.
fn running() -> (u64, u64, u64) {
    let out = Command::new(ANK).arg("--version").output().unwrap();
    let line = String::from_utf8(out.stdout).unwrap();
    let version = line
        .strip_prefix("ank ")
        .and_then(|rest| rest.split_whitespace().next())
        .unwrap_or_else(|| panic!("ank --version printed no version: {line}"));
    let parts: Vec<u64> = version.split('.').map(|p| p.parse().unwrap()).collect();
    assert_eq!(parts.len(), 3, "not MAJOR.MINOR.PATCH: {version}");
    (parts[0], parts[1], parts[2])
}

fn version_text((major, minor, patch): (u64, u64, u64)) -> String {
    format!("{major}.{minor}.{patch}")
}

/// A git repository holding a corpus whose `config.yml` is `requires` under
/// `schema: 2`, or carries no requirement at all, and a file a task can scope.
fn corpus(what: &str, requires: Option<&str>) -> PathBuf {
    let dir = scratch::dir(what);
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&dir)
            .env("GIT_CONFIG_GLOBAL", isolated_git_config())
            .env("GIT_CONFIG_SYSTEM", isolated_git_config())
            .output()
            .expect("git must be on PATH");
        assert!(out.status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    git(&["commit", "-q", "--allow-empty", "-m", "root"]);
    fs::create_dir_all(dir.join(".ank/entities")).unwrap();
    fs::write(dir.join("a.txt"), "a\n").unwrap();
    require(&dir, requires);
    dir
}

/// Rewrites the requirement in place, so a run with it and a run without it
/// answer about the same repository.
fn require(dir: &Path, requires: Option<&str>) {
    let config = match requires {
        Some(v) => format!("schema: 2\nrequires_ank: {v}\n"),
        None => "schema: 2\n".to_string(),
    };
    fs::write(dir.join(".ank/config.yml"), config).unwrap();
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
    /// The argument list of every git the run started.
    gits: Vec<String>,
}

/// `ank <args>` in `dir`, with `GIT_TRACE` at an absolute path.
fn ank(dir: &Path, args: &[&str]) -> Run {
    let trace = scratch::dir("requires-ank-trace").join("trace");
    let out = Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("GIT_TRACE", &trace)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .env("ANK_AGENT", "claude-code/requires-ank-it")
        .stdin(Stdio::null())
        .output()
        .expect("the binary under test must run");
    let gits: Vec<String> = fs::read_to_string(&trace)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(MARK).map(|(_, rest)| rest.to_string()))
        .collect();
    // Every run of the suite, so "no verb reached the network" is a property
    // of the suite rather than of the runs a test remembered to ask about.
    assert!(
        !gits.iter().any(|g: &String| g.starts_with("ls-remote")),
        "ank {args:?} started git ls-remote: {gits:#?}"
    );
    Run {
        code: out.status.code().expect("the binary must not be signalled"),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        gits,
    }
}

impl Run {
    fn said(&self) -> String {
        format!("{}{}", self.stdout, self.stderr)
    }
}

/// The warning a verb prints below the requirement, whole.
fn warning(declared: &str, running: &str) -> String {
    format!(
        "warning: this corpus requires ank {declared} or newer, and this binary is {running}\n  -> ank update\n"
    )
}

/// The fault `check` reports below the requirement, on the line `check`
/// prints a fault on.
fn fault(declared: &str, running: &str) -> String {
    format!(
        "error: requires_ank: this corpus requires ank {declared} or newer, and this binary is {running}"
    )
}

const FIND: &[&str] = &["find", "--status", "open"];
const FIND_JSON: &[&str] = &["find", "--status", "open", "--json"];

fn new_task(json: bool) -> Vec<&'static str> {
    let mut args = vec![
        "new",
        "task",
        "--title",
        "A task",
        "--scope",
        "a.txt",
        "--criteria",
        "it holds",
        "--no-verify",
    ];
    if json {
        args.push("--json");
    }
    args
}

// ---------------------------------------------------------------------------
// Above the running version: every verb warns and goes on
// ---------------------------------------------------------------------------

/// A requirement one minor ahead: for the 0.9.x line this is `0.10.0`, which
/// is lower than `0.9.x` read as a string, so the warning is only printed by a
/// comparison of numbers.
fn above() -> String {
    let (major, minor, _) = running();
    format!("{major}.{}.0", minor + 1)
}

#[test]
fn a_read_verb_warns_once_and_answers_as_without_the_key() {
    let running = version_text(running());
    let declared = above();
    let dir = corpus("requires-ank-find", Some(&declared));

    let with = ank(&dir, FIND);
    assert_eq!(
        with.code,
        0,
        "find goes on below the requirement: {}",
        with.said()
    );
    assert_eq!(
        with.stderr,
        warning(&declared, &running),
        "find prints one warning naming both versions and ank update"
    );

    let with_json = ank(&dir, FIND_JSON);
    assert_eq!(with_json.code, 0, "{}", with_json.said());
    assert_eq!(with_json.stderr, warning(&declared, &running));

    require(&dir, None);
    let without = ank(&dir, FIND);
    assert_eq!(without.code, 0, "{}", without.said());
    assert_eq!(without.stderr, "", "no key, no warning");
    assert_eq!(with.stdout, without.stdout, "the answer is the same");
    let without_json = ank(&dir, FIND_JSON);
    assert_eq!(without_json.code, 0, "{}", without_json.said());
    assert_eq!(
        with_json.stdout, without_json.stdout,
        "the --json document is byte-identical to a run without the key"
    );
}

/// The write verb creates what it would have created, and its document differs
/// from a run without the key only by what every run mints: an identifier and
/// an instant, masked by the redaction the goldens are written under.
#[test]
fn a_write_verb_warns_once_and_writes_as_without_the_key() {
    let running = version_text(running());
    let declared = above();
    let dir = corpus("requires-ank-new", Some(&declared));

    let with = ank(&dir, &new_task(false));
    assert_eq!(
        with.code,
        0,
        "new goes on below the requirement: {}",
        with.said()
    );
    assert_eq!(
        with.stderr,
        warning(&declared, &running),
        "new prints one warning naming both versions and ank update"
    );
    assert!(with.stdout.starts_with("created TASK-"), "{}", with.said());

    let with_json = ank(&dir, &new_task(true));
    assert_eq!(with_json.code, 0, "{}", with_json.said());
    assert_eq!(with_json.stderr, warning(&declared, &running));

    require(&dir, None);
    let without_json = ank(&dir, &new_task(true));
    assert_eq!(without_json.code, 0, "{}", without_json.said());
    assert_eq!(without_json.stderr, "", "no key, no warning");
    assert_eq!(
        fixture::redacted(&with_json.stdout),
        fixture::redacted(&without_json.stdout),
        "the --json document is the one a run without the key prints"
    );
}

#[test]
fn quiet_prints_no_warning() {
    let dir = corpus("requires-ank-quiet", Some(&above()));
    for args in [
        &["find", "--status", "open", "--quiet"][..],
        &["check", "--quiet"],
    ] {
        let run = ank(&dir, args);
        assert!(
            !run.stderr.contains("warning:"),
            "{args:?} --quiet printed a warning: {}",
            run.stderr
        );
    }
    let run = ank(&dir, &["find", "--status", "open", "--quiet"]);
    assert_eq!(run.code, 0, "{}", run.said());
}

#[test]
fn check_exits_8_with_a_fault_naming_both_versions() {
    let running = version_text(running());
    let declared = above();
    let dir = corpus("requires-ank-check", Some(&declared));

    let run = ank(&dir, &["check"]);
    assert!(
        !run.gits.is_empty(),
        "the trace recorded no git at all, so its silence on ls-remote proves nothing"
    );
    assert_eq!(
        run.code,
        8,
        "a binary below the requirement fails check: {}",
        run.said()
    );
    assert!(
        run.stdout.contains(&fault(&declared, &running)),
        "check names both versions in a fault: {}",
        run.said()
    );
    assert!(
        run.stdout
            .contains(&format!("{}\n└── ank update\n", fault(&declared, &running))),
        "the fault names the remedy: {}",
        run.said()
    );

    let json = ank(&dir, &["check", "--json"]);
    assert_eq!(json.code, 8, "{}", json.said());
    let doc: serde_yaml::Value = serde_yaml::from_str(&json.stdout)
        .unwrap_or_else(|e| panic!("not a document ({e}): {}", json.stdout));
    let findings = doc["findings"].as_sequence().expect("findings is an array");
    let ours: Vec<_> = findings
        .iter()
        .filter(|f| f["kind"] == "requires-ank")
        .collect();
    assert_eq!(ours.len(), 1, "one finding of kind requires-ank: {doc:?}");
    assert_eq!(ours[0]["level"], "fault", "{doc:?}");
}

/// Component by component as numbers: the patch is ten past the running one,
/// a string that sorts below it, and it is still above.
#[test]
fn a_patch_ten_past_the_running_one_is_above_it() {
    let (major, minor, patch) = running();
    let running = version_text((major, minor, patch));
    let declared = format!("{major}.{minor}.{}", patch + 10);
    let dir = corpus("requires-ank-numbers", Some(&declared));

    let run = ank(&dir, FIND);
    assert_eq!(run.code, 0, "{}", run.said());
    assert_eq!(run.stderr, warning(&declared, &running));
    let check = ank(&dir, &["check"]);
    assert_eq!(check.code, 8, "{}", check.said());
    assert!(
        check.stdout.contains(&fault(&declared, &running)),
        "{}",
        check.said()
    );
}

// ---------------------------------------------------------------------------
// At or below the running version, or no key: nothing
// ---------------------------------------------------------------------------

/// Equal, below, below by numbers where a string would say above, and absent
/// all answer as a corpus that never heard of the key: `find` silent, `check`
/// green and saying nothing about it.
#[test]
fn at_or_below_the_running_version_and_with_no_key_nothing_is_said() {
    let (major, minor, patch) = running();
    let equal = version_text((major, minor, patch));
    let mut cases: Vec<Option<String>> = vec![None, Some(equal), Some("0.0.0".into())];
    // 0.9.9 sorts above 0.10.0 as a string and is below it as numbers; only
    // a running version from 0.10.0 on can show it, and the patch case above
    // shows the same rule for the version this suite runs today.
    if (major, minor) >= (0, 10) {
        cases.push(Some("0.9.9".into()));
    }
    let baseline = ank(&corpus("requires-ank-baseline", None), &["check"]);
    assert_eq!(baseline.code, 0, "{}", baseline.said());

    for (i, requires) in cases.iter().enumerate() {
        let dir = corpus(&format!("requires-ank-silent-{i}"), requires.as_deref());

        let find = ank(&dir, FIND);
        assert_eq!(find.code, 0, "{requires:?}: {}", find.said());
        assert_eq!(find.stderr, "", "{requires:?}: nothing is printed");

        let created = ank(&dir, &new_task(false));
        assert_eq!(created.code, 0, "{requires:?}: {}", created.said());
        assert_eq!(created.stderr, "", "{requires:?}: nothing is printed");

        let check = ank(&dir, &["check"]);
        assert_eq!(
            check.code,
            0,
            "{requires:?}: check stays green: {}",
            check.said()
        );
        assert!(
            !check.said().contains("requires"),
            "{requires:?}: check says nothing about it: {}",
            check.said()
        );
    }
}
