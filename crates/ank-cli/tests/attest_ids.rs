//! The attest job anchors every task `check` reports as finished with nothing
//! external behind it, the typed `test:<ref>` included (TASK-44e6b39c13d6).
//!
//! The selection used to be a jq filter inline in ci.yml, matching
//! `startswith("done with no test proof")`. `check` words that finding two
//! ways, and the second -- `done with no attested test proof: 'test:999' was
//! submitted, not attested` -- is the task somebody closed on a test reference
//! they typed. Measured in a scratch corpus before the fix: two findings, one
//! id selected. The typed task was never anchored and `check` kept reporting it.
//!
//! **Through the binary and through the script, both.** The defect lives in the
//! gap between what the binary prints and what the script expects, so a test
//! of either alone asserts nothing about it: the corpus is built by `ank`, its
//! findings are `ank check --json` as printed, and the ids are what
//! `.github/scripts/unanchored.sh` prints from them.

mod scratch;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("attest-ids-gitconfig");
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

fn spawn(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    let home = scratch::root();
    c.current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", "test/attest-ids")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home);
    c
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// `sh` exists on Linux and macOS, and on a Windows runner it is not on the
/// PATH cargo test sees -- the same skip `adopt.rs` makes for `install.sh`.
/// The script runs on ubuntu in the attest job and nowhere else, so the
/// platform it is skipped on is one it is never executed on.
fn has_sh() -> bool {
    Command::new("sh")
        .args(["-c", "exit 0"])
        .stdin(Stdio::null())
        .output()
        .is_ok_and(|o| o.status.success())
}

/// A task created, claimed and closed on `proof`, returning its id.
fn close_on(dir: &Path, title: &str, proof: &str) -> String {
    let id = run(
        ANK,
        dir,
        &[
            "new",
            "task",
            "--title",
            title,
            "--scope",
            "f",
            "--criteria",
            title,
            "--no-verify",
            "--json",
        ],
    );
    // The first identifier in the document is the task's own: ank-cli carries
    // no JSON parser for its tests, and an id is a fixed shape.
    let at = id.find("TASK-").expect("new task --json names the id");
    let id = id[at..at + "TASK-".len() + 12].to_string();
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", &format!("new {id}")]);
    let proof = proof.replace("<head>", &run("git", dir, &["rev-parse", "HEAD"]));
    run(ANK, dir, &["claim", &id]);
    run(ANK, dir, &["done", "--proof", &proof]);
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", &format!("done {id}")]);
    id
}

#[test]
fn the_attest_job_selects_a_task_closed_on_a_typed_test_proof() {
    if !has_sh() {
        eprintln!("skipped: no sh on PATH");
        return;
    }

    let dir = scratch::dir("attest-ids");
    run("git", &dir, &["init", "-q", "-b", "main"]);
    fs::write(dir.join("f"), "x\n").unwrap();
    run(ANK, &dir, &["init"]);
    run(ANK, &dir, &["config", "default_branch", "main"]);
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "init"]);

    let untested = close_on(&dir, "closed on a commit", "commit:<head>");
    let typed = close_on(&dir, "closed on a typed test reference", "test:999");

    // `check` on the default branch, exit 0: both are signals, never faults.
    let findings = spawn(ANK, &dir)
        .args(["check", "--json"])
        .stdin(Stdio::null())
        .output()
        .expect("ank check must run");
    assert_eq!(
        findings.status.code(),
        Some(0),
        "ank check --json: {}",
        String::from_utf8_lossy(&findings.stderr)
    );

    let mut script = spawn("sh", &root())
        .arg(".github/scripts/unanchored.sh")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("sh must run the script");
    script
        .stdin
        .take()
        .unwrap()
        .write_all(&findings.stdout)
        .unwrap();
    let out = script.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "unanchored.sh (jq must be on PATH): {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Lines and not bytes: jq on Windows ends them in CRLF.
    let mut ids: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect();
    ids.sort();
    let mut want = vec![untested, typed];
    want.sort();
    assert_eq!(
        ids,
        want,
        "the ids the attest job anchors, from:\n{}",
        String::from_utf8_lossy(&findings.stdout)
    );
}

/// The script is what the job runs. A selection tested here and bypassed by an
/// inline filter in ci.yml would leave the defect exactly where it was.
#[test]
fn ci_selects_through_the_script() {
    let ci = fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap();
    assert!(
        ci.contains("sh .github/scripts/unanchored.sh"),
        "ci.yml no longer calls .github/scripts/unanchored.sh"
    );
    assert!(
        !ci.contains("select(.message"),
        "ci.yml matches a finding's wording itself instead of through the script"
    );
}
