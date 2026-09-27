//! `ank config` refuses a peer declared by URL, and names the override
//! (ADR-96fe1f9d619a, TASK-bcdc32d08947).
//!
//! A peer is a checkout on the reader's disk. A URL written as a peer's path is
//! read as a path, the path "is not a corpus", and on a back-declaration the
//! binding then vanishes with no warning at all (measured under
//! TASK-c6d184d238e1). The refusal is taken where the value is written, which
//! is the last place the mistake is still visible to whoever made it, and it
//! names `ank config --user peers.<identity>.<name> <path>`, the per-reader
//! override of ADR-da2819aef598, as what to say instead.
//!
//! **Through the binary**, for both files: `config.yml` and the reader's
//! `corpora.yml`, whose location only the environment of a child process moves.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-url-refusal";

/// Every shape the criterion names as a URL: four schemes and the scp form.
const URLS: &[&str] = &[
    "https://github.com/haksolot/ank",
    "ssh://git@github.com/haksolot/ank",
    "file:///srv/git/b",
    "git://example.org/b.git",
    "git@github.com:haksolot/ank.git",
];

fn run(home: &Path, dir: &Path, args: &[&str]) -> Output {
    let root = scratch::root();
    Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("ANK_AGENT", AGENT)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .output()
        .expect("the binary runs")
}

fn ok(home: &Path, dir: &Path, args: &[&str]) {
    let out = run(home, dir, args);
    assert!(
        out.status.success(),
        "ank {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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

/// A corpus with one commit, so it has the identity an override is keyed on,
/// and one peer already declared, so `peers:` exists and a refusal has a
/// populated block it must leave alone.
fn corpus(home: &Path, dir: &Path) -> String {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    ok(home, dir, &["init"]);
    ok(home, dir, &["config", "peers.bb", "../b"]);
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "init",
        ],
    );
    git(dir, &["rev-list", "--max-parents=0", "HEAD"])
}

/// Asserts one refusal: exit 1, the file byte for byte as it was, and the
/// override named as the command to use instead.
fn refused(home: &Path, dir: &Path, args: &[&str], file: &PathBuf, override_key: &str) {
    let before = fs::read(file).unwrap();
    let out = run(home, dir, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "ank {args:?} must refuse:\n{stderr}"
    );
    assert_eq!(
        fs::read(file).unwrap(),
        before,
        "ank {args:?} changed {}",
        file.display()
    );
    assert!(
        stderr.contains(&format!("ank config --user {override_key} <path>")),
        "ank {args:?} must name the override:\n{stderr}"
    );
}

#[test]
fn config_refuses_a_peer_url_and_names_the_override() {
    let base = scratch::dir("peer-url-refusal");
    let home = base.join("home");
    let a = base.join("a");
    let id = corpus(&home, &a);
    let config = a.join(".ank/config.yml");

    for url in URLS {
        refused(
            &home,
            &a,
            &["config", "peers.bb", url],
            &config,
            &format!("peers.{id}.bb"),
        );
        // A new name as well as an existing one: the refusal is about the
        // value, not about what it would replace.
        refused(
            &home,
            &a,
            &["config", "peers.cc", url],
            &config,
            &format!("peers.{id}.cc"),
        );
    }
}

#[test]
fn config_user_refuses_a_peer_url_and_names_the_override() {
    let base = scratch::dir("peer-url-refusal-user");
    let home = base.join("home");
    let a = base.join("a");
    let id = corpus(&home, &a);
    let key = format!("peers.{id}.bb");
    // A populated corpora.yml, so "unchanged" is measured on real bytes.
    ok(&home, &a, &["config", "--user", &key, "../elsewhere/b"]);
    let corpora = home.join("ank").join("corpora.yml");

    for url in URLS {
        refused(&home, &a, &["config", "--user", &key, url], &corpora, &key);
    }
}

#[test]
fn a_path_is_still_accepted_by_both() {
    let base = scratch::dir("peer-url-accepted");
    let home = base.join("home");
    let a = base.join("a");
    let id = corpus(&home, &a);
    let absolute = base.join("b").to_string_lossy().into_owned();
    let key = format!("peers.{id}.bb");

    for path in [
        "../b",
        "sub/dir/b",
        absolute.as_str(),
        "C:/src/b",
        "C:\\src\\b",
    ] {
        ok(&home, &a, &["config", "peers.bb", path]);
        let out = run(&home, &a, &["config", "peers.bb"]);
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), path);

        ok(&home, &a, &["config", "--user", &key, path]);
        let out = run(&home, &a, &["config", "--user", &key]);
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), path);
    }
}
