//! A repository whose origin does not fetch `refs/ank/*` is told so, by `init`
//! when it cannot add the refspec and by `status` and `check` afterwards
//! (TASK-623d80886c2f, issue #499).
//!
//! Measured with ank 0.8.0 in a scratch repository: `git init -b main && ank
//! init` exited 0 and said nothing about the refspec, `git remote add origin
//! <url>` left `remote.origin.fetch` holding `+refs/heads/*:refs/remotes/origin/*`
//! alone, and neither `status` nor `check` printed a line about it. Claims and
//! completion refs were then never fetched, and nothing said so. A fresh clone
//! is the same state by construction: the refspec lives in `.git/config`, which
//! a clone never carries.
//!
//! `init` still writes nothing while origin has no URL -- that is what keeps
//! `git remote add origin` working (TASK-f067ae7c84ff, tests/init_origin.rs).
//! What changes is that the gap is reported where it is visible, and every
//! surface names the same repair, which this suite reads off the output and
//! runs as printed.
//!
//! **Refspecs, not exit codes**: each step asserts on `git config --get-all
//! remote.origin.fetch`.

mod scratch;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const ANK_REFSPEC: &str = "+refs/ank/*:refs/ank/*";
const HEADS_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";

/// A global and system git config of this suite's own, so a developer's
/// `~/.gitconfig` adds nothing to the remote under test.
fn isolated_git_config() -> &'static Path {
    static CONFIG: OnceLock<PathBuf> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let p = scratch::path("init-refspec-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.invalid\n\
             [init]\n\tdefaultBranch = main\n",
        )
        .unwrap();
        p
    })
}

/// Git-for-Windows' path conversion switches are dropped for the reason
/// `tests/status.rs` gives.
fn spawn(program: impl AsRef<OsStr>, dir: &Path) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", "init@fixture")
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("MSYS2_ARG_CONV_EXCL");
    c
}

fn run(mut c: Command) -> (i32, String) {
    let out = c.output().expect("the process must start");
    let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
    said.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().expect("must not be signalled"), said)
}

fn git(dir: &Path, args: &[&str]) -> String {
    let mut c = spawn("git", dir);
    c.args(args);
    let (code, said) = run(c);
    assert_eq!(code, 0, "git {args:?}: {said}");
    said.trim().to_string()
}

fn ank(dir: &Path, args: &[&str]) -> (i32, String) {
    let mut c = spawn(ANK, dir);
    c.args(args);
    run(c)
}

fn fetch_refspecs(dir: &Path) -> Vec<String> {
    let mut c = spawn("git", dir);
    c.args(["config", "--get-all", "remote.origin.fetch"]);
    let (_, said) = run(c);
    said.lines().map(str::to_string).collect()
}

/// The line of `said` that reports the missing refspec, if any.
fn refspec_line(said: &str) -> Option<&str> {
    said.lines()
        .find(|l| l.contains(ANK_REFSPEC) && l.contains("remote.origin.fetch"))
}

/// The command a line names in its closing parenthesis, split into words.
fn named_command(line: &str) -> Vec<String> {
    let inner = line
        .trim_end()
        .strip_suffix(')')
        .and_then(|l| l.rsplit_once('('))
        .map(|(_, c)| c)
        .unwrap_or_else(|| panic!("no command named in: {line}"));
    inner.split_whitespace().map(str::to_string).collect()
}

/// `status` and `check` each report the gap, `check` at exit 0, and both name
/// the same command. Returns it.
fn both_report_the_gap(dir: &Path) -> Vec<String> {
    let (code, said) = ank(dir, &["status"]);
    assert_eq!(code, 0, "{said}");
    let from_status = named_command(
        refspec_line(&said).unwrap_or_else(|| panic!("status is silent on the refspec: {said}")),
    );
    let (code, said) = ank(dir, &["check"]);
    assert_eq!(
        code, 0,
        "a missing refspec is a signal, never a fault: {said}"
    );
    let from_check = named_command(
        refspec_line(&said).unwrap_or_else(|| panic!("check is silent on the refspec: {said}")),
    );
    assert_eq!(from_status, from_check);
    from_status
}

fn neither_reports_the_gap(dir: &Path) {
    let (code, said) = ank(dir, &["status"]);
    assert_eq!(code, 0, "{said}");
    assert!(refspec_line(&said).is_none(), "{said}");
    let (code, said) = ank(dir, &["check"]);
    assert_eq!(code, 0, "{said}");
    assert!(refspec_line(&said).is_none(), "{said}");
}

/// Runs a command read off ank's output, exactly as printed.
fn run_named(dir: &Path, words: &[String]) {
    assert_eq!(words.first().map(String::as_str), Some("ank"), "{words:?}");
    let (code, said) = ank(
        dir,
        &words[1..].iter().map(String::as_str).collect::<Vec<_>>(),
    );
    assert_eq!(code, 0, "`{}` failed: {said}", words.join(" "));
}

#[test]
fn a_refspec_init_could_not_add_is_reported_until_it_is_added() {
    let base = scratch::dir("init-refspec-sequence");
    let repo = base.join("repo");
    let bare = base.join("origin.git");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bare).unwrap();
    git(&repo, &["init", "-q", "-b", "main", "."]);
    git(&bare, &["init", "-q", "--bare", "."]);

    // init says it added nothing, why, and what adds it once origin exists.
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    let line = said
        .lines()
        .find(|l| l.contains(ANK_REFSPEC))
        .unwrap_or_else(|| panic!("init is silent on the refspec: {said}"));
    assert!(line.contains("not added"), "{line}");
    assert!(line.contains("no remote named origin"), "{line}");
    let from_init = named_command(line);
    assert!(fetch_refspecs(&repo).is_empty());
    // Nothing to report yet: there is no origin to fetch from.
    neither_reports_the_gap(&repo);

    // The remote is added, and git still accepts it.
    let mut c = spawn("git", &repo);
    c.args(["remote", "add", "origin", &bare.to_string_lossy()]);
    let (code, out) = run(c);
    assert_eq!(code, 0, "git remote add origin failed: {out}");
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC]);

    let named = both_report_the_gap(&repo);
    assert_eq!(
        named, from_init,
        "init and the reports name different repairs"
    );

    run_named(&repo, &named);
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    neither_reports_the_gap(&repo);

    // A fresh clone of a repository whose origin carries refs/ank/*: the
    // refspec lives in .git/config, so the clone has only git's own.
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "ank init"]);
    git(&repo, &["update-ref", "refs/ank/claims/fixture", "HEAD"]);
    git(
        &repo,
        &["push", "-q", "origin", "main", "refs/ank/claims/fixture"],
    );
    let clone = base.join("clone");
    git(
        &base,
        &[
            "clone",
            "-q",
            &bare.to_string_lossy(),
            &clone.to_string_lossy(),
        ],
    );
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC]);
    let named = both_report_the_gap(&clone);
    assert_eq!(named, from_init);
    run_named(&clone, &named);
    assert_eq!(fetch_refspecs(&clone), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    neither_reports_the_gap(&clone);
    // And what the refspec is for: a plain fetch now brings the ank ref.
    git(&clone, &["fetch", "-q", "origin"]);
    let refs = git(&clone, &["for-each-ref", "--format=%(refname)", "refs/ank"]);
    assert!(
        refs.lines().any(|r| r == "refs/ank/claims/fixture"),
        "{refs}"
    );
}

#[test]
fn help_init_no_longer_says_the_refspec_is_always_added() {
    let dir = scratch::dir("init-refspec-help");
    let (code, said) = ank(&dir, &["help", "init"]);
    assert_eq!(code, 0, "{said}");
    assert!(!said.contains("adds the refs/ank/* refspec;"), "{said}");
    assert!(said.contains("once origin"), "{said}");
}
