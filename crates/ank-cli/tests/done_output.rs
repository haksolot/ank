//! A failing verifier's output is kept beside the repository, and its last
//! lines are printed (TASK-69006ffa955e).
//!
//! **Through the binary.** `done` used to read what a verifier printed, keep
//! its hash and delete the file, so a red close never said why. Every case
//! here drives `ank done` and reads what it left on disk.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "test/done-output";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("done-output-gitconfig");
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
    let home = scratch::root();
    Command::new(program)
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", AGENT)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home)
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("{program} must run: {e}"))
}

fn run(program: &str, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, dir, args);
    assert!(
        out.status.success(),
        "{program} {args:?}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// `<root>/repo` is the repository; the files steering the verifiers live in
/// `<root>`, outside every work tree, so a sibling worktree at `<root>/wt`
/// reaches the same ones as `../<name>`.
///
/// `fifty` prints fifty numbered lines, each prefixed by `<root>/tag`, and
/// exits 1 unless `<root>/pass` exists. `slow` prints one line and sleeps past
/// its one-second timeout. `fine` always passes.
struct Fixture {
    root: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let root = scratch::dir(what);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/x"), "x\n").unwrap();
        fs::write(root.join("tag"), "first\n").unwrap();
        run("git", &repo, &["init", "-q", "-b", "main"]);
        run(ANK, &repo, &["init"]);
        run(ANK, &repo, &["config", "default_branch", "main"]);
        for (key, value) in [
            (
                "verifiers.fifty.run",
                r#"t=$(cat ../tag); i=1; while [ $i -le 50 ]; do echo "$t-line-$i"; i=$((i+1)); done; test -f ../pass"#,
            ),
            ("verifiers.slow.run", "echo before-the-sleep; sleep 30"),
            ("verifiers.slow.timeout", "1s"),
            ("verifiers.fine.run", "echo fine"),
        ] {
            run(ANK, &repo, &["config", key, value]);
        }
        commit(&repo, "init");
        Fixture { root, repo }
    }

    fn common_dir(&self) -> PathBuf {
        let common = run("git", &self.repo, &["rev-parse", "--git-common-dir"]);
        let common = Path::new(&common);
        let common = if common.is_absolute() {
            common.to_path_buf()
        } else {
            self.repo.join(common)
        };
        fs::canonicalize(common).unwrap()
    }
}

fn commit(dir: &Path, message: &str) {
    run("git", dir, &["add", "-A"]);
    run("git", dir, &["commit", "-q", "-m", message]);
}

/// A task over `src/**` declaring `verify`, committed in `dir`, not claimed.
fn task(dir: &Path, title: &str, verify: &[&str]) -> String {
    let mut args = vec![
        "new",
        "task",
        "--title",
        title,
        "--scope",
        "src/**",
        "--criteria",
        "the verifiers pass",
        "--json",
    ];
    for v in verify {
        args.extend(["--verify", v]);
    }
    let new = run(ANK, dir, &args);
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    commit(dir, title);
    id
}

/// The path `done` named on standard error as the one holding the output.
fn kept_path(stderr: &str) -> PathBuf {
    let line = stderr
        .lines()
        .find(|l| l.contains("output kept:"))
        .unwrap_or_else(|| panic!("done names no kept output:\n{stderr}"));
    let path = line.split("output kept:").nth(1).unwrap().trim();
    PathBuf::from(path)
}

fn numbered(text: &str, tag: &str) -> Vec<usize> {
    let prefix = format!("{tag}-line-");
    text.lines()
        .filter_map(|l| l.trim().strip_prefix(&prefix))
        .map(|n| n.parse().unwrap())
        .collect()
}

#[test]
fn a_failure_prints_its_last_twenty_lines_and_keeps_all_fifty_under_the_common_dir() {
    let f = Fixture::new("done-output-fifty");
    let id = task(&f.repo, "fails loudly", &["fifty"]);
    run(ANK, &f.repo, &["claim", &id]);

    let out = spawn(ANK, &f.repo, &["done"]);
    assert_eq!(out.status.code(), Some(5), "the verifier said no");
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        numbered(&err, "first"),
        (31..=50).collect::<Vec<_>>(),
        "the last twenty lines, in order, and no other:\n{err}"
    );

    let path = kept_path(&err);
    let kept = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: the output is kept: {e}", path.display()));
    assert_eq!(numbered(&kept, "first"), (1..=50).collect::<Vec<_>>());

    let canonical = fs::canonicalize(&path).unwrap();
    assert!(
        canonical.starts_with(f.common_dir()),
        "{} lies under the git common directory {}",
        canonical.display(),
        f.common_dir().display()
    );
    let named = path.to_string_lossy();
    assert!(named.contains(&id), "the path names the task: {named}");
    assert!(
        named.contains("fifty"),
        "the path names the verifier: {named}"
    );
}

#[test]
fn a_second_failure_replaces_the_file_and_a_linked_worktree_names_the_same_path() {
    let f = Fixture::new("done-output-worktree");
    let id = task(&f.repo, "fails twice", &["fifty"]);
    let wt = f.root.join("wt");
    run(
        "git",
        &f.repo,
        &["worktree", "add", "-q", "-b", "other", wt.to_str().unwrap()],
    );

    run(ANK, &wt, &["claim", &id]);
    let first = spawn(ANK, &wt, &["done"]);
    assert_eq!(first.status.code(), Some(5));
    let from_wt = kept_path(&String::from_utf8_lossy(&first.stderr));
    run(
        ANK,
        &wt,
        &["release", "--reason", "moving to the main checkout"],
    );

    fs::write(f.root.join("tag"), "second\n").unwrap();
    run(ANK, &f.repo, &["claim", &id]);
    let second = spawn(ANK, &f.repo, &["done"]);
    assert_eq!(second.status.code(), Some(5));
    let from_repo = kept_path(&String::from_utf8_lossy(&second.stderr));

    assert_eq!(
        fs::canonicalize(&from_wt).unwrap(),
        fs::canonicalize(&from_repo).unwrap(),
        "one path for the task and the verifier, whichever worktree ran it"
    );
    let kept = fs::read_to_string(&from_repo).unwrap();
    assert_eq!(numbered(&kept, "second"), (1..=50).collect::<Vec<_>>());
    assert!(
        numbered(&kept, "first").is_empty(),
        "the second run replaced the first:\n{kept}"
    );
}

#[test]
fn a_timed_out_verifier_keeps_its_output_the_same_way() {
    let f = Fixture::new("done-output-timeout");
    let id = task(&f.repo, "hangs", &["slow"]);
    run(ANK, &f.repo, &["claim", &id]);

    let out = spawn(ANK, &f.repo, &["done"]);
    assert_eq!(out.status.code(), Some(5), "a timeout is a code 5");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("timed out"), "{err}");
    assert!(
        err.lines().any(|l| l.trim() == "before-the-sleep"),
        "the tail is printed:\n{err}"
    );
    let path = kept_path(&err);
    let named = path.to_string_lossy();
    assert!(named.contains(&id) && named.contains("slow"), "{named}");
    assert_eq!(
        fs::read_to_string(&path).unwrap().trim(),
        "before-the-sleep"
    );
}

#[test]
fn a_passing_verifier_leaves_no_file_and_removes_the_one_a_failure_left() {
    let f = Fixture::new("done-output-pass");
    let id = task(&f.repo, "fails then passes", &["fine", "fifty"]);
    run(ANK, &f.repo, &["claim", &id]);

    let failed = spawn(ANK, &f.repo, &["done"]);
    assert_eq!(failed.status.code(), Some(5));
    let err = String::from_utf8_lossy(&failed.stderr);
    let path = kept_path(&err);
    assert!(path.is_file());
    assert_eq!(
        err.matches("output kept:").count(),
        1,
        "only the failure is kept:\n{err}"
    );
    let dir = path.parent().unwrap();
    let names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().all(|n| !n.contains("fine")),
        "a passing verifier leaves no file: {names:?}"
    );

    fs::write(f.root.join("pass"), "").unwrap();
    let passed = spawn(ANK, &f.repo, &["done"]);
    assert!(
        passed.status.success(),
        "{}",
        String::from_utf8_lossy(&passed.stderr)
    );
    assert!(
        !path.exists(),
        "{} outlived the pass that answered it",
        path.display()
    );
}

#[test]
fn done_json_carries_the_kept_path_under_the_declared_key() {
    let f = Fixture::new("done-output-json");
    let id = task(&f.repo, "fails in json", &["fifty"]);
    run(ANK, &f.repo, &["claim", &id]);

    let out = spawn(ANK, &f.repo, &["done", "--json"]);
    assert_eq!(out.status.code(), Some(5));
    let stdout = String::from_utf8_lossy(&out.stdout);
    let doc: serde_yaml::Value = serde_yaml::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("done --json prints one document: {e}\n{stdout}"));
    let output = doc["output"]
        .as_str()
        .unwrap_or_else(|| panic!("no output key:\n{stdout}"));
    assert_eq!(
        Path::new(output),
        kept_path(&String::from_utf8_lossy(&out.stderr))
    );
    assert_eq!(doc["task"].as_str(), Some(id.as_str()));
    assert_eq!(doc["failed"].as_str(), Some("fifty"));
    // The document pinned with the two values a run cannot repeat masked: the
    // task minted here and the path under this fixture's scratch directory.
    let pinned = stdout
        .trim()
        .replace(&output.replace('\\', "\\\\"), "<PATH>")
        .replace(&id, "TASK-000000000002");
    crate::fixture::pin("done-failed", &pinned);

    let spec = ank_contract::spec_of("done").unwrap();
    assert!(
        spec.output
            .iter()
            .any(|shape| shape.fields.iter().any(|field| field.name == "output")),
        "the contract table declares the key done prints"
    );
}
