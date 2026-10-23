//! A fetch from origin never writes or prunes the claims a remote named `ank`
//! carries (ADR-b6fe84504ff2, TASK-df475a664d21).
//!
//! origin's `+refs/ank/*:refs/ank/*` (ADR-1b030fe70442) matches
//! `refs/ank/claims/*` too, so once `ank` carries the claims a plain `git fetch
//! origin` would write whatever claims origin still holds, and `git fetch
//! --prune origin` would delete every live claim origin does not. The answer is
//! a negative refspec on origin, `^refs/ank/claims/*`, beside a fetch refspec
//! for the claims on `ank`.
//!
//! Measured on git 2.47.3 before relying on it: in a clone holding a claim
//! locally and on `ank` and none on origin, `git fetch --prune origin` deleted
//! the claim without the negative line and kept it with it, and a claim origin
//! did hold was written without it and not with it, while
//! `refs/ank/ratify/*` and `refs/ank/proof/*` came either way. The assertions
//! below are that measurement, run by the suite on every platform CI has.
//!
//! **Refspecs and refs, not exit codes**: each step asserts on `git config
//! --get-all` and on `git for-each-ref`.

use crate::scratch;

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const ANK_REFSPEC: &str = "+refs/ank/*:refs/ank/*";
const HEADS_REFSPEC: &str = "+refs/heads/*:refs/remotes/origin/*";
const ANK_HEADS_REFSPEC: &str = "+refs/heads/*:refs/remotes/ank/*";
const CLAIMS_REFSPEC: &str = "+refs/ank/claims/*:refs/ank/claims/*";
const NOT_CLAIMS: &str = "^refs/ank/claims/*";
const T: &str = "TASK-0000000000d1";

fn isolated_git_config() -> &'static Path {
    static CONFIG: OnceLock<PathBuf> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let p = scratch::path("claims-refspec-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.invalid\n\
             [init]\n\tdefaultBranch = main\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n",
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
        .env("ANK_AGENT", "refspec@fixture")
        .env_remove("MSYS_NO_PATHCONV")
        .env_remove("MSYS2_ARG_CONV_EXCL");
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git", dir).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The binary, with stdout and stderr apart.
fn ank(dir: &Path, args: &[&str]) -> (i32, String, String) {
    let out = spawn(ANK, dir).args(args).output().unwrap();
    (
        out.status.code().expect("must not be signalled"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn fetch_of(dir: &Path, remote: &str) -> Vec<String> {
    let out = spawn("git", dir)
        .args(["config", "--get-all", &format!("remote.{remote}.fetch")])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn local_refs(dir: &Path) -> Vec<String> {
    let mut v: Vec<String> = git(dir, &["for-each-ref", "--format=%(refname)", "refs/ank"])
        .lines()
        .map(str::to_string)
        .collect();
    v.sort();
    v
}

/// A repository with a commit on `main`, and a bare remote for each of
/// `remotes`, added and holding `main`. Returns the base and the repository.
fn repository(what: &str, remotes: &[&str]) -> (PathBuf, PathBuf) {
    let base = scratch::dir(what);
    let repo = base.join("repo");
    fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q", "-b", "main", "."]);
    fs::write(repo.join("README.md"), "# fixture\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "seed"]);
    for name in remotes {
        add_remote(&base, &repo, name);
    }
    (base, repo)
}

fn add_remote(base: &Path, repo: &Path, name: &str) {
    let bare = base.join(format!("{name}.git"));
    fs::create_dir_all(&bare).unwrap();
    git(&bare, &["init", "-q", "--bare", "-b", "main", "."]);
    git(repo, &["remote", "add", name, &bare.to_string_lossy()]);
    git(repo, &["push", "-q", name, "main"]);
}

/// The criterion's measurement: a claim present locally and on `ank` and none
/// on origin, a ratification and a proof on origin only. `git fetch --prune
/// origin` keeps the claim and brings the other two; a claim origin does
/// carry is not written.
fn origin_fetch_leaves_the_claims_alone(base: &Path, repo: &Path) {
    let claim = format!("refs/ank/claims/{T}");
    let ratify = "refs/ank/ratify/ADR-0000000000a1";
    let proof = format!("refs/ank/proof/{T}");
    git(repo, &["update-ref", &claim, "HEAD"]);
    git(repo, &["push", "-q", "ank", &claim]);
    git(repo, &["update-ref", ratify, "HEAD"]);
    git(repo, &["update-ref", &proof, "HEAD"]);
    git(repo, &["push", "-q", "origin", ratify, &proof]);
    git(repo, &["update-ref", "-d", ratify]);
    git(repo, &["update-ref", "-d", &proof]);
    let origin = base.join("origin.git");
    assert!(
        !git(&origin, &["for-each-ref", "refs/ank/claims"]).contains("claims"),
        "the fixture's origin must carry no claim"
    );

    git(repo, &["fetch", "-q", "--prune", "origin"]);
    let mut want = vec![claim.clone(), proof.clone(), ratify.to_string()];
    want.sort();
    assert_eq!(
        local_refs(repo),
        want,
        "fetch --prune origin deleted a claim ank carries, or did not bring the ratification and the proof"
    );

    // A claim origin still carries, left there before ank took them over.
    let stray = "refs/ank/claims/TASK-0000000000d2";
    git(&origin, &["update-ref", stray, "main"]);
    git(repo, &["fetch", "-q", "--prune", "origin"]);
    assert!(
        !local_refs(repo).iter().any(|r| r == stray),
        "fetch origin wrote a claim origin carries: {:?}",
        local_refs(repo)
    );
    assert!(local_refs(repo).contains(&claim), "{:?}", local_refs(repo));
}

#[test]
fn init_with_origin_and_ank_fetches_claims_from_ank_and_never_from_origin() {
    let (base, repo) = repository("claims-refspec-init", &["origin", "ank"]);

    let (code, out, err) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        fetch_of(&repo, "ank"),
        vec![ANK_HEADS_REFSPEC, CLAIMS_REFSPEC]
    );
    assert_eq!(
        fetch_of(&repo, "origin"),
        vec![HEADS_REFSPEC, ANK_REFSPEC, NOT_CLAIMS]
    );
    assert!(
        out.lines()
            .any(|l| l.contains("remote.ank.fetch") && l.contains(CLAIMS_REFSPEC)),
        "init names the refspec it added to ank: {out}"
    );
    assert!(
        out.lines()
            .any(|l| l.contains("remote.origin.fetch") && l.contains(NOT_CLAIMS)),
        "init names the refspec it added to origin: {out}"
    );

    // Idempotent: a second run adds nothing.
    let (code, out, err) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{out}{err}");
    assert!(!out.contains("refs/ank/claims"), "{out}");
    assert_eq!(fetch_of(&repo, "ank").len(), 2);
    assert_eq!(fetch_of(&repo, "origin").len(), 3);

    origin_fetch_leaves_the_claims_alone(&base, &repo);
}

/// The repair of ADR-1b030fe70442 reaches the same configuration in a clone
/// where `ank` arrived after `init`, from `status` and from `check` alike.
#[test]
fn ank_added_after_init_is_repaired_by_the_first_verb_that_runs_git() {
    for verb in ["status", "check"] {
        let (base, repo) = repository(&format!("claims-refspec-repair-{verb}"), &["origin"]);
        let (code, out, err) = ank(&repo, &["init"]);
        assert_eq!(code, 0, "{out}{err}");
        assert_eq!(fetch_of(&repo, "origin"), vec![HEADS_REFSPEC, ANK_REFSPEC]);

        add_remote(&base, &repo, "ank");
        let (code, out, err) = ank(&repo, &[verb]);
        assert_eq!(code, 0, "{verb}: {out}{err}");
        assert_eq!(
            fetch_of(&repo, "ank"),
            vec![ANK_HEADS_REFSPEC, CLAIMS_REFSPEC],
            "{verb}: {err}"
        );
        assert_eq!(
            fetch_of(&repo, "origin"),
            vec![HEADS_REFSPEC, ANK_REFSPEC, NOT_CLAIMS],
            "{verb}: {err}"
        );
        assert!(
            err.lines()
                .any(|l| l.contains("remote.ank.fetch") && l.contains(CLAIMS_REFSPEC)),
            "{verb} says on stderr what it added to ank: {err}"
        );
        assert!(
            err.lines()
                .any(|l| l.contains("remote.origin.fetch") && l.contains(NOT_CLAIMS)),
            "{verb} says on stderr what it added to origin: {err}"
        );
        assert!(!out.contains("lacks"), "{verb}: {out}");

        // Once repaired, nothing more is said.
        let (code, out, err) = ank(&repo, &[verb]);
        assert_eq!(code, 0, "{verb}: {out}{err}");
        assert!(!err.contains("refs/ank/claims"), "{verb}: {err}");

        origin_fetch_leaves_the_claims_alone(&base, &repo);
    }
}

/// Where the repair cannot write -- `.git/config` locked -- `status` warns and
/// `check` signals, each naming the remote whose refspec is missing.
#[test]
fn status_and_check_name_the_remote_whose_refspec_is_missing() {
    let (base, repo) = repository("claims-refspec-gap", &["origin"]);
    let (code, out, err) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{out}{err}");
    add_remote(&base, &repo, "ank");
    let lock = repo.join(".git/config.lock");
    fs::write(&lock, "").unwrap();

    let gaps = |said: &str| {
        let ank = said
            .lines()
            .any(|l| l.contains("remote.ank.fetch lacks") && l.contains(CLAIMS_REFSPEC));
        let origin = said
            .lines()
            .any(|l| l.contains("remote.origin.fetch lacks") && l.contains(NOT_CLAIMS));
        (ank, origin)
    };
    let (code, out, err) = ank(&repo, &["status"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(gaps(&out), (true, true), "status names both remotes: {out}");
    let (code, out, err) = ank(&repo, &["check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(gaps(&out), (true, true), "check names both remotes: {out}");
    assert!(
        out.lines()
            .any(|l| l.starts_with("signal") && l.contains("remote.ank.fetch lacks")),
        "a signal, never a fault: {out}"
    );

    fs::remove_file(&lock).unwrap();
    let (code, out, err) = ank(&repo, &["check"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(gaps(&out), (false, false), "{out}");
}

/// A clone with origin alone ends where it ends today, and one whose `ank` is
/// removed gets back to exactly that.
#[test]
fn a_clone_with_only_origin_keeps_the_fetch_configuration_it_has_today() {
    let (base, repo) = repository("claims-refspec-origin", &["origin"]);
    let (code, out, err) = ank(&repo, &["init"]);
    assert_eq!(code, 0, "{out}{err}");
    let (code, out, err) = ank(&repo, &["status"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(fetch_of(&repo, "origin"), vec![HEADS_REFSPEC, ANK_REFSPEC]);
    assert!(!out.contains("refs/ank/claims") && !err.contains("refs/ank/claims"));

    add_remote(&base, &repo, "ank");
    let (code, out, err) = ank(&repo, &["status"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        fetch_of(&repo, "origin"),
        vec![HEADS_REFSPEC, ANK_REFSPEC, NOT_CLAIMS]
    );

    git(&repo, &["remote", "remove", "ank"]);
    let (code, out, err) = ank(&repo, &["status"]);
    assert_eq!(code, 0, "{out}{err}");
    assert_eq!(
        fetch_of(&repo, "origin"),
        vec![HEADS_REFSPEC, ANK_REFSPEC],
        "the claims came back to origin, and so does its fetch: {err}"
    );
    assert!(
        err.lines()
            .any(|l| l.contains("remote.origin.fetch") && l.contains(NOT_CLAIMS)),
        "status says what it removed: {err}"
    );
}
