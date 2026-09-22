//! An archived file whose bytes changed stays a fault after `index.db` is
//! deleted (TASK-77df1446260f).
//!
//! `check` held an archived file to the digest the index took when the file
//! arrived, and to nothing else. The index is disposable: deleted, it is rebuilt
//! from the bytes it is meant to verify, and the edited file then matches
//! itself. Measured before the fix, on the corpus built below: exit 8 naming the
//! file, then exit 0 on every run after `rm .ank/index.db`. A committed move is
//! anchored in the tree HEAD records, and that is what judges the file now, so
//! the verdict does not depend on the cache: the edit stays a fault with the
//! index rebuilt, and the file restored with `git checkout` is green again even
//! though the rebuilt index took the edited bytes' digest.
//!
//! Through the binary, because the criterion is about what `check` reports, and
//! with a corpus `ank archive` itself moved and a commit carries.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@archive-digest";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("archive-digest-gitconfig");
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

fn spawn(program: &str, dir: &Path, args: &[&str]) -> Output {
    let config = isolated_git_config();
    let root = scratch::root();
    Command::new(program)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("{program}: {e}"))
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir, args);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// `check`'s exit code, and the faults it names archived files in.
fn check(dir: &Path) -> (i32, Vec<String>) {
    let out = spawn(ANK, dir, &["check"]);
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

fn drop_index(dir: &Path) {
    for entry in fs::read_dir(dir.join(".ank")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with("index.db") {
            fs::remove_file(&path).unwrap();
        }
    }
    assert!(!dir.join(".ank/index.db").exists());
}

#[test]
fn an_edited_archived_file_stays_a_fault_after_the_index_is_deleted() {
    let dir = scratch::dir("archive-digest");
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

    // The entries of a task done on the default branch are cold.
    run(ANK, &dir, &["archive"]);
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "archive"]);
    let cold = dir.join(".ank/archive/entities");
    let mut moved: Vec<PathBuf> = fs::read_dir(&cold)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    moved.sort();
    assert!(moved.len() >= 2, "ank archive moved {moved:?}");
    assert_eq!(
        check(&dir),
        (0, vec![]),
        "the archive as committed is green"
    );

    let edited = &moved[0];
    let name = edited.file_name().unwrap().to_string_lossy().to_string();
    let mut bytes = fs::read(edited).unwrap();
    bytes.extend_from_slice(b"an edit\n");
    fs::write(edited, bytes).unwrap();

    let names = |faults: &[String]| faults.iter().all(|f| f.contains(&name));
    let (code, faults) = check(&dir);
    assert_eq!(code, 8, "with the index: {faults:?}");
    assert_eq!(faults.len(), 1, "{faults:?}");
    assert!(names(&faults), "{faults:?}");

    drop_index(&dir);
    for run in ["first", "second"] {
        let (code, faults) = check(&dir);
        assert_eq!(code, 8, "the {run} run after rm index.db: {faults:?}");
        assert_eq!(faults.len(), 1, "the {run} run: {faults:?}");
        assert!(names(&faults), "the {run} run: {faults:?}");
    }

    // The index now holds the edited bytes' digest; the file restored as the
    // fault says is green all the same, because HEAD judges it.
    run(
        "git",
        &dir,
        &["checkout", "--", &format!(".ank/archive/entities/{name}")],
    );
    assert_eq!(check(&dir), (0, vec![]), "restored with git checkout");
}
