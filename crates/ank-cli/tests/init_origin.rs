//! `ank init` leaves a repository in which `git remote add origin <url>` works,
//! and the remote it adds fetches both branches and `refs/ank/*`
//! (TASK-f067ae7c84ff).
//!
//! Measured before the fix, on git 2.47, in a repository with one commit and no
//! remote: after `ank init`, `.git/config` held `remote.origin.fetch =
//! +refs/ank/*:refs/ank/*` and nothing else under `origin`; `ank status
//! --remote` warned `no remote named origin ... (git remote add origin <url>)`;
//! running that command exited 3 with `error: remote origin already exists.`;
//! and `git remote set-url origin <url>`, which does get past it, left
//! `+refs/ank/*` as the only fetch refspec, so `git fetch origin` brought no
//! branch at all.
//!
//! **Through the binary, and the command is the one status prints.** The
//! repair is read off `ank status --remote`, its `<url>` replaced and nothing
//! else, and run as given: a test that typed `git remote add origin` itself
//! would stay green the day status started naming something else.
//!
//! **Refspecs, not exit codes.** `git remote add` succeeding is half of it; the
//! other half is what `remote.origin.fetch` carries afterwards, and whether a
//! plain `git fetch origin` then brings both a branch and an ank ref.

mod scratch;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const ANK_REFSPEC: &str = "+refs/ank/*:refs/ank/*";
const HEADS_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";

/// A global and system git config of this suite's own: signing off, an author,
/// and nothing a developer's `~/.gitconfig` could add to the remote under test.
fn isolated_git_config() -> &'static Path {
    static CONFIG: OnceLock<PathBuf> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let p = scratch::path("init-origin-gitconfig");
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

/// Every process this suite starts. Git-for-Windows' path conversion switches
/// are dropped for the reason `tests/status.rs` gives: with either set, a
/// Windows path handed to git as a remote is rewritten before git sees it.
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

/// Exit code and both streams, of any process.
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

/// The fetch refspecs of `origin`, as git resolves them: includes followed.
fn fetch_refspecs(dir: &Path) -> Vec<String> {
    let mut c = spawn("git", dir);
    c.args(["config", "--get-all", "remote.origin.fetch"]);
    let (_, said) = run(c);
    said.lines().map(str::to_string).collect()
}

/// A repository with one commit, and an empty bare repository beside it to be
/// its origin.
fn repository(what: &str) -> (PathBuf, PathBuf) {
    let base = scratch::dir(what);
    let repo = base.join("repo");
    let bare = base.join("origin.git");
    fs::create_dir_all(&repo).unwrap();
    fs::create_dir_all(&bare).unwrap();
    git(&repo, &["init", "-q", "."]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "root"]);
    git(&bare, &["init", "-q", "--bare", "."]);
    (repo, bare)
}

/// Puts a branch and an ank ref on origin, removes this clone's copies of
/// both, and fetches with no refspec on the command line: what comes back is
/// what `remote.origin.fetch` asked for.
fn assert_plain_fetch_brings_both(repo: &Path) {
    git(repo, &["update-ref", "refs/ank/claims/fixture", "HEAD"]);
    git(
        repo,
        &[
            "push",
            "-q",
            "origin",
            "HEAD:refs/heads/main",
            "refs/ank/claims/fixture",
        ],
    );
    git(repo, &["update-ref", "-d", "refs/ank/claims/fixture"]);
    let _ = run({
        let mut c = spawn("git", repo);
        c.args(["update-ref", "-d", "refs/remotes/origin/main"]);
        c
    });
    git(repo, &["fetch", "-q", "origin"]);
    let refs = git(repo, &["for-each-ref", "--format=%(refname)"]);
    assert!(
        refs.lines().any(|r| r == "refs/remotes/origin/main"),
        "a plain fetch brought no branch: {refs}"
    );
    assert!(
        refs.lines().any(|r| r == "refs/ank/claims/fixture"),
        "a plain fetch brought no ank ref: {refs}"
    );
}

#[test]
fn the_remote_status_names_can_be_added_after_init_and_fetches_both_planes() {
    let (repo, bare) = repository("init-origin-fresh");
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    assert!(
        said.contains(&format!("refspec added: {ANK_REFSPEC}")),
        "{said}"
    );

    // The repair, read off status and run as it is printed.
    let (_, said) = ank(&repo, &["status", "--remote"]);
    let hint = said
        .lines()
        .find(|l| l.contains("no remote named origin"))
        .and_then(|l| l.rsplit_once('(')?.1.strip_suffix(')'))
        .unwrap_or_else(|| panic!("status named no repair for a missing origin: {said}"))
        .to_string();
    let url = bare.to_string_lossy();
    let words: Vec<String> = hint
        .split_whitespace()
        .map(|w| {
            if w == "<url>" {
                url.to_string()
            } else {
                w.to_string()
            }
        })
        .collect();
    assert_eq!(words.first().map(String::as_str), Some("git"), "{hint}");
    let mut c = spawn("git", &repo);
    c.args(&words[1..]);
    let (code, out) = run(c);
    assert_eq!(code, 0, "`{hint}`, the command status names, failed: {out}");

    let specs = fetch_refspecs(&repo);
    assert!(
        specs.iter().any(|s| s == ANK_REFSPEC),
        "no ank refspec: {specs:?}"
    );
    assert!(
        specs.iter().any(|s| s == HEADS_REFSPEC),
        "no branch refspec: {specs:?}"
    );
    assert_eq!(specs.len(), 2, "{specs:?}");

    // Status now reads the remote plane instead of warning that there is none.
    let (_, said) = ank(&repo, &["status", "--remote"]);
    assert!(!said.contains("no remote named origin"), "{said}");

    assert_plain_fetch_brings_both(&repo);

    // And a second init has nothing left to add.
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "already initialised, nothing to do");
    assert_eq!(fetch_refspecs(&repo).len(), 2);
}

#[test]
fn an_origin_that_already_exists_gets_the_refspec_beside_its_own() {
    let (repo, bare) = repository("init-origin-existing");
    git(&repo, &["remote", "add", "origin", &bare.to_string_lossy()]);
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    assert_plain_fetch_brings_both(&repo);
}

/// Remotes, none of them `origin`: a deferred refspec would be live at once and
/// would configure `origin` by itself, which is the defect. So nothing is
/// written, `git remote add origin` stays open, and init run again once origin
/// exists adds the refspec directly.
#[test]
fn another_remote_leaves_origin_free_and_a_second_init_completes_it() {
    let (repo, bare) = repository("init-origin-other");
    git(
        &repo,
        &["remote", "add", "upstream", &bare.to_string_lossy()],
    );
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    assert!(!said.contains("refspec added"), "{said}");
    assert!(fetch_refspecs(&repo).is_empty());

    git(&repo, &["remote", "add", "origin", &bare.to_string_lossy()]);
    let (code, said) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{said}");
    assert!(
        said.contains(&format!("refspec added: {ANK_REFSPEC}")),
        "{said}"
    );
    assert_eq!(fetch_refspecs(&repo), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    assert_plain_fetch_brings_both(&repo);
}
