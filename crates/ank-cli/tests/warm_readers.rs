//! A reader of a warm index never meets "database is locked"
//! (TASK-f0dd3c98a6bf).
//!
//! Measured on ci run 37067325662 (ubuntu-latest, main at 7d07cc5d) and then
//! outside the suite, pinned to one core: one reader in twelve refused with
//! "index: another process is writing the index (database is locked)" on a
//! corpus that was warm, about one burst in a hundred and fifty. Every refusal
//! carried SQLite's extended code 261, `SQLITE_BUSY_RECOVERY`, from the first
//! read of the connection. The process that warmed the index closed the last
//! connection, which checkpoints the WAL and deletes `-wal` and `-shm`; the next
//! readers then race to rebuild the shared-memory index, and the losers are told
//! the database is busy while one of them recovers. No writer is involved, and
//! with `ANK_INDEX_BUSY_MS=0` the busy handler gave them no time at all.
//!
//! The race is arranged here rather than waited for: a large WAL that was never
//! checkpointed and no `-shm` beside it make the recovery long enough that the
//! twelve readers started together are certain to meet it.

mod index_file;
mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@warm-readers";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("warm-readers-gitconfig");
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

fn command(program: &str, dir: &Path, args: &[&str]) -> Command {
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
    cmd
}

fn run(program: &str, dir: &Path, args: &[&str]) {
    let out: Output = command(program, dir, args)
        .output()
        .unwrap_or_else(|e| panic!("{program}: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A repository with twelve tasks, and an index the binary has built.
fn warm_corpus() -> PathBuf {
    let dir = scratch::dir("warm-readers");
    run("git", &dir, &["init", "-q", "-b", "main"]);
    run(
        "git",
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "init"],
    );
    run(ANK, &dir, &["init"]);
    for i in 0..12 {
        run(
            ANK,
            &dir,
            &[
                "new",
                "task",
                "--title",
                &format!("Task {i}"),
                "--scope",
                "src/**",
                "--no-verify",
            ],
        );
    }
    run(ANK, &dir, &["find", "Task"]);
    dir
}

/// Leaves the index in the state the readers met on CI, made long: a WAL that
/// holds frames no checkpoint has applied, and no `-shm` beside it, so the next
/// connection to read must rebuild the shared-memory index from the whole WAL.
fn leave_a_wal_to_recover(ank: &Path) {
    let db = index_file::path(ank);
    {
        let conn = rusqlite::Connection::open(&db).expect("the index opens");
        conn.set_db_config(
            rusqlite::config::DbConfig::SQLITE_DBCONFIG_NO_CKPT_ON_CLOSE,
            true,
        )
        .expect("checkpoint on close can be turned off");
        conn.execute_batch(
            "PRAGMA wal_autocheckpoint = 0;
             CREATE TABLE IF NOT EXISTS ballast (b BLOB);",
        )
        .expect("ballast table");
        let tx = conn.unchecked_transaction().unwrap();
        for _ in 0..4000 {
            tx.execute("INSERT INTO ballast (b) VALUES (randomblob(4096))", [])
                .unwrap();
        }
        tx.commit().unwrap();
    }
    let wal = ank.join(format!("{}-wal", index_file::name()));
    let size = fs::metadata(&wal).map(|m| m.len()).unwrap_or(0);
    assert!(
        size > 8_000_000,
        "the arrangement needs a large WAL left unapplied, found {size} bytes at {}",
        wal.display()
    );
    let _ = fs::remove_file(ank.join(format!("{}-shm", index_file::name())));
}

#[test]
fn readers_meeting_a_wal_recovery_are_not_refused() {
    let dir = warm_corpus();
    leave_a_wal_to_recover(&dir.join(".ank"));

    let verbs: [&[&str]; 3] = [&["find", "Task"], &["context"], &["scope", "src"]];
    let running: Vec<_> = (0..12)
        .map(|i| {
            command(ANK, &dir, verbs[i % verbs.len()])
                .env("ANK_AGENT", format!("agent-{i}@host"))
                .env("ANK_INDEX_BUSY_MS", "0")
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("the binary must have been built")
        })
        .collect();

    let refused: Vec<String> = running
        .into_iter()
        .enumerate()
        .filter_map(|(i, child)| {
            let out = child.wait_with_output().expect("a spawned ank must finish");
            (!out.status.success()).then(|| {
                format!(
                    "#{i} exited {:?}: {}",
                    out.status.code(),
                    String::from_utf8_lossy(&out.stderr).trim()
                )
            })
        })
        .collect();
    assert!(
        refused.is_empty(),
        "a reader was refused while another reader rebuilt the WAL index; nothing \
         was being written, so there was no writer to wait for:\n{}",
        refused.join("\n")
    );
}
