//! The index is one file per schema, and the archived digests carry over
//! (TASK-a8d3219e84fb, ADR-3db9735a7036).
//!
//! Measured during the parallel run of 2026-10-02: a binary at index schema 10
//! on PATH and the tree's binary at 11 shared `.ank/index.db`, each found the
//! other's version in it, and each dropped every table and rebuilt, in a loop.
//! A binary now reads and writes `.ank/index.db.<N>` alone, `N` its own
//! schema, and leaves every other file exactly as it found it. The one thing
//! the index holds that the files cannot rebuild, the digest an archived file
//! arrived with, is taken over from the newest older file when the binary
//! creates its own.
//!
//! Through the binary, because every clause is about what a verb leaves on
//! disk and what `check` then reports.

mod index_file;
mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@index-per-schema";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("index-per-schema-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn spawn(program: &str, dir: &Path, args: &[&str], envs: &[(&str, &Path)]) -> Output {
    let config = isolated_git_config();
    let root = scratch::root();
    let mut cmd = Command::new(program);
    cmd.args(args)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .current_dir(dir);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().unwrap_or_else(|e| panic!("{program}: {e}"))
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir, args, &[]);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// A repository with `ank init` run in it and one task committed.
fn corpus(what: &str) -> (PathBuf, String) {
    let dir = scratch::dir(what);
    run("git", &dir, &["init", "-q", "-b", "main"]);
    run(
        "git",
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "init"],
    );
    run(ANK, &dir, &["init"]);
    run(ANK, &dir, &["config", "default_branch", "main"]);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), "").unwrap();
    let out = run(
        ANK,
        &dir,
        &[
            "new",
            "task",
            "--title",
            "finished",
            "--scope",
            "src/**",
            "--criteria",
            "a criterion",
            "--no-verify",
        ],
    );
    let id = out
        .split_whitespace()
        .find(|w| w.starts_with("TASK-"))
        .unwrap_or_else(|| panic!("new task names no id: {out}"))
        .to_string();
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "seed"]);
    (dir, id)
}

/// Every file under `.ank/` whose name starts `index.db`, sorted.
fn index_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.join(".ank"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("index.db"))
        .collect();
    names.sort();
    names
}

/// `check`'s exit code, and the lines naming an archived file changed.
fn check(dir: &Path) -> (i32, Vec<String>) {
    let out = spawn(ANK, dir, &["check"], &[]);
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let faults = text
        .lines()
        .filter(|l| l.contains("archived file changed"))
        .map(str::to_string)
        .collect();
    (out.status.code().unwrap_or(-1), faults)
}

#[test]
fn a_verb_writes_its_own_schema_file_and_git_status_never_offers_it() {
    let (dir, _) = corpus("index-per-schema-own");
    index_file::remove_all(&dir.join(".ank"));

    run(ANK, &dir, &["find", "--status", "open"]);

    let own = index_file::name();
    assert!(
        index_files(&dir).contains(&own),
        "the verb did not write {own}: {:?}",
        index_files(&dir)
    );
    assert!(
        !dir.join(".ank/index.db").exists(),
        "the verb created the unsuffixed .ank/index.db: {:?}",
        index_files(&dir)
    );

    let status = run("git", &dir, &["status", "--porcelain", "-uall"]);
    assert!(
        !status.contains("index.db"),
        "git status offers an index file:\n{status}"
    );
}

#[test]
fn other_schemas_files_are_left_byte_identical_and_the_own_one_is_reused() {
    let (dir, id) = corpus("index-per-schema-others");
    let ank = dir.join(".ank");
    let n = index_file::schema();

    // Real index bytes for the files an older binary would have left: the
    // one this binary builds, copied under their names before it is removed.
    run(ANK, &dir, &["find", "--status", "open"]);
    let built = fs::read(index_file::path(&ank)).unwrap();
    let others = [
        "index.db".to_string(),
        format!("index.db.{}", n - 1),
        format!("index.db.{}", n + 1),
    ];
    fs::write(ank.join(&others[0]), &built).unwrap();
    fs::write(ank.join(&others[1]), &built).unwrap();
    fs::write(
        ank.join(&others[2]),
        b"a newer schema this binary cannot read",
    )
    .unwrap();
    for name in index_files(&dir) {
        if !others.contains(&name) {
            fs::remove_file(ank.join(name)).unwrap();
        }
    }
    let before: Vec<Vec<u8>> = others
        .iter()
        .map(|o| fs::read(ank.join(o)).unwrap())
        .collect();

    // Rebuilt once, then reused: the first run indexes the corpus, and none
    // of the three after it indexes anything.
    let mut indexed = Vec::new();
    for i in 0..4 {
        let counted = scratch::path(&format!("index-per-schema-refreshed-{i}"));
        let _ = fs::remove_file(&counted);
        let out = spawn(
            ANK,
            &dir,
            &["find", "--status", "open"],
            &[("ANK_INDEX_REFRESHED", &counted)],
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let lines = fs::read_to_string(&counted).unwrap_or_default();
        let total: u64 = lines
            .split_whitespace()
            .filter_map(|kv| kv.strip_prefix("indexed="))
            .map(|v| v.parse::<u64>().unwrap())
            .sum();
        indexed.push(total);
    }
    assert!(indexed[0] > 0, "the first run rebuilt nothing: {indexed:?}");
    assert_eq!(
        &indexed[1..],
        &[0, 0, 0],
        "the binary's own index was not reused: indexed per run {indexed:?}"
    );

    for args in [
        vec!["show", id.as_str()],
        vec!["log", id.as_str()],
        vec!["context", "src"],
        vec!["status"],
        vec!["find", "--all"],
        vec!["check"],
    ] {
        let _ = spawn(ANK, &dir, &args, &[]);
    }

    for (name, bytes) in others.iter().zip(&before) {
        assert!(
            fs::read(ank.join(name)).unwrap() == *bytes,
            "{name} was rewritten by a verb of schema {n}"
        );
    }
}

#[test]
fn an_archived_digest_carries_over_from_an_older_schema_file() {
    carries_over(&format!("index.db.{}", index_file::schema() - 1));
}

/// The file every release before the split wrote is an older schema's too, and
/// the one an upgrade actually finds.
#[test]
fn an_archived_digest_carries_over_from_the_unsuffixed_file() {
    carries_over("index.db");
}

fn carries_over(older: &str) {
    let (dir, id) = corpus(&format!("index-per-schema-carry-{older}"));
    let ank = dir.join(".ank");
    run(ANK, &dir, &["claim", &id]);
    run(ANK, &dir, &["log", "a note"]);
    let head = run("git", &dir, &["rev-parse", "HEAD"]);
    run(
        ANK,
        &dir,
        &["done", "--proof", &format!("commit:{}", head.trim())],
    );
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "done"]);

    // Moved and not committed: HEAD carries no blob for the archived files,
    // so the digest the index took when they arrived is the only thing they
    // are held to (TASK-77df1446260f judges a committed move by HEAD).
    run(ANK, &dir, &["archive"]);
    let cold = ank.join("archive/entities");
    let mut moved: Vec<PathBuf> = fs::read_dir(&cold)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    moved.sort();
    assert!(!moved.is_empty(), "ank archive moved nothing");
    assert_eq!(check(&dir), (0, vec![]), "the archive as moved is green");

    // The upgrade: the index this binary wrote becomes the file of the
    // schema before it, and this binary has none of its own yet.
    for name in index_files(&dir) {
        if name == index_file::name() {
            fs::rename(ank.join(&name), ank.join(older)).unwrap();
        } else {
            fs::remove_file(ank.join(&name)).unwrap();
        }
    }
    let older_bytes = fs::read(ank.join(older)).unwrap();

    // Edited before the upgrade's first verb.
    let edited = &moved[0];
    let name = edited.file_name().unwrap().to_string_lossy().to_string();
    let mut bytes = fs::read(edited).unwrap();
    bytes.extend_from_slice(b"an edit\n");
    fs::write(edited, bytes).unwrap();

    for run in ["first", "second"] {
        let (code, faults) = check(&dir);
        assert_eq!(
            code, 8,
            "the {run} check after the upgrade lost the archived digest: {faults:?}"
        );
        assert!(
            faults.len() == 1 && faults[0].contains(&name),
            "the {run} run: {faults:?}"
        );
    }
    assert!(
        index_files(&dir).contains(&index_file::name()),
        "{:?}",
        index_files(&dir)
    );
    assert!(
        fs::read(ank.join(older)).unwrap() == older_bytes,
        "{older} was rewritten"
    );

    // Restored, it is green, and its row is indexed from the restored bytes.
    let original = fs::read(edited).unwrap();
    fs::write(edited, &original[..original.len() - b"an edit\n".len()]).unwrap();
    assert_eq!(
        check(&dir),
        (0, vec![]),
        "restored to the bytes it arrived with"
    );
    let shown = run(ANK, &dir, &["find", "--all"]);
    assert!(shown.contains("(archived)"), "{shown}");
}
