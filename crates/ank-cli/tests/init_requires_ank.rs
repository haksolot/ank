//! `ank init` writes `requires_ank` as the running version, and only a corpus
//! it creates (ADR-756e8b0fe416, TASK-96fa594bdb15).
//!
//! **Through the binary, and compared with what the binary says it is.** The
//! version is read from `ank --version` and never from `CARGO_PKG_VERSION`: the
//! criterion is that the two agree, and a test that read the same constant the
//! code writes would agree with itself whatever the binary printed.
//!
//! **The opening of the file is compared byte for byte**, because `ank config
//! requires_ank` inserts the key beneath `schema:` and a corpus born under
//! `init` should read the same as one raised by `config` afterwards. The rest
//! of the file is answered by the reader: `ank config requires_ank` reads the
//! key back, which it does only when the whole file parses.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A run of the binary in `dir`, with the reader's home pointed at `home` on
/// every platform (`XDG_CONFIG_HOME`, and `APPDATA` on Windows), so a
/// contributor's own `corpora.yml` is never the one read.
fn ank(dir: &Path, home: &Path, args: &[&str]) -> (i32, String, String) {
    let home = home.to_string_lossy().into_owned();
    let out = Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("XDG_CONFIG_HOME", &home)
        .env("APPDATA", &home)
        .env("HOME", "")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("ANK_AGENT", "init-requires@fixture")
        .output()
        .expect("the binary under test must run");
    (
        out.status.code().expect("the binary must not be signalled"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// An empty repository: `git init` and nothing else, no commit.
fn empty_repo(dir: &Path) {
    fs::create_dir_all(dir).expect("the repository must be creatable");
    git(dir, &["init", "-q", "."]);
    git(dir, &["config", "commit.gpgsign", "false"]);
}

/// A repository with one commit, which is what a detached corpus is keyed on
/// (ADR-621a7fd96ce1: the identity is the root commit).
fn committed_repo(dir: &Path) {
    empty_repo(dir);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "root",
        ],
    );
}

/// A reader's home holding an empty `corpora.yml`.
fn home(what: &str) -> PathBuf {
    let home = scratch::dir(what).join("config");
    fs::create_dir_all(home.join("ank")).expect("the reader's home must be creatable");
    fs::write(
        home.join("ank").join("corpora.yml"),
        "schema: 1\ncorpora: {}\n",
    )
    .expect("corpora.yml must be writable");
    home
}

/// The version `ank --version` prints first: `ank <version> (...)`.
fn printed_version(home: &Path) -> String {
    let (code, out, err) = ank(Path::new("."), home, &["--version"]);
    assert_eq!(code, 0, "ank --version: {err}");
    let version = out
        .split_whitespace()
        .nth(1)
        .unwrap_or_else(|| panic!("ank --version names no version: {out:?}"))
        .to_string();
    let parts: Vec<&str> = version.split('.').collect();
    assert!(
        parts.len() == 3 && parts.iter().all(|p| p.parse::<u64>().is_ok()),
        "ank --version printed {version:?}, not MAJOR.MINOR.PATCH"
    );
    version
}

/// The criterion's file content: a corpus born at schema 2, with the running
/// version beneath the schema, and a file the reader takes whole.
fn assert_born_with(corpus: &Path, home: &Path, version: &str) {
    let cfg = corpus.join(".ank").join("config.yml");
    let text = fs::read_to_string(&cfg).unwrap_or_else(|e| panic!("{}: {e}", cfg.display()));
    assert!(
        text.starts_with(&format!("schema: 2\nrequires_ank: {version}\n")),
        "config.yml opens on schema 2 and the running version: {text}"
    );
    assert_eq!(
        text.matches("requires_ank").count(),
        1,
        "and carries the key once: {text}"
    );
    let (code, out, err) = ank(corpus, home, &["config", "requires_ank"]);
    assert_eq!(code, 0, "the reader takes the file whole: {err}");
    assert_eq!(out.trim(), version, "and reads the version back");
}

#[test]
fn init_in_an_empty_repository_writes_the_running_version_at_schema_2() {
    let home = home("init-requires-empty");
    let repo = scratch::dir("init-requires-empty").join("repo");
    empty_repo(&repo);
    let version = printed_version(&home);

    let (code, _, err) = ank(&repo, &home, &["init"]);

    assert_eq!(code, 0, "ank init: {err}");
    assert_born_with(&repo, &home, &version);
}

#[test]
fn init_at_writes_the_running_version_in_the_declared_corpus() {
    let home = home("init-requires-at");
    let base = scratch::dir("init-requires-at");
    let (source, detached) = (base.join("repo"), base.join("detached"));
    committed_repo(&source);
    empty_repo(&detached);
    let version = printed_version(&home);

    let (code, _, err) = ank(
        &source,
        &home,
        &["init", "--at", &detached.to_string_lossy()],
    );

    assert_eq!(code, 0, "ank init --at: {err}");
    assert!(
        !source.join(".ank").exists(),
        "the corpus is the declared one, not the work tree's"
    );
    assert_born_with(&detached, &home, &version);
    // Read from the source repository too: the declaration is what routes the
    // reader there, so this is the corpus a verb in the work tree reads.
    let (code, out, err) = ank(&source, &home, &["config", "requires_ank"]);
    assert_eq!(code, 0, "the work tree reads the declared corpus: {err}");
    assert_eq!(out.trim(), version);
}

#[test]
fn check_on_a_fresh_corpus_exits_0_with_no_fault() {
    let home = home("init-requires-check");
    let repo = scratch::dir("init-requires-check").join("repo");
    committed_repo(&repo);
    let (code, _, err) = ank(&repo, &home, &["init"]);
    assert_eq!(code, 0, "ank init: {err}");

    let (code, out, err) = ank(&repo, &home, &["check", "--json"]);

    assert_eq!(code, 0, "check on a fresh corpus: {out}{err}");
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&out).unwrap_or_else(|e| panic!("check --json: {e}: {out}"));
    assert_eq!(doc["faults"].as_u64(), Some(0), "no fault counted: {out}");
    let findings = doc["findings"]
        .as_sequence()
        .unwrap_or_else(|| panic!("check --json carries its findings: {out}"));
    assert!(
        findings
            .iter()
            .all(|f| f["level"].as_str() != Some("fault")),
        "and none listed: {out}"
    );
}

/// Written before `init` runs, and left byte for byte: `init` on an existing
/// corpus is the refspec repair `status` and `check` name, and it is not one of
/// the two writes the ADR lets move the requirement.
#[test]
fn init_over_an_existing_corpus_leaves_requires_ank_as_it_was() {
    let home = home("init-requires-existing");
    let base = scratch::dir("init-requires-existing");
    let cases = [
        (
            "declared",
            "schema: 2\nrequires_ank: 0.0.1\nverifiers: {}\n",
        ),
        ("absent", "schema: 1\nverifiers: {}\n"),
    ];
    for (what, before) in cases {
        let repo = base.join(what);
        empty_repo(&repo);
        fs::create_dir_all(repo.join(".ank")).unwrap();
        let cfg = repo.join(".ank").join("config.yml");
        fs::write(&cfg, before).unwrap();

        let (code, _, err) = ank(&repo, &home, &["init"]);

        assert_eq!(code, 0, "{what}: ank init over a corpus: {err}");
        assert_eq!(
            fs::read_to_string(&cfg).unwrap(),
            before,
            "{what}: config.yml is left byte for byte"
        );
    }
}
