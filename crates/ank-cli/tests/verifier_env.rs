//! A verifier declares `run`, `timeout` and `inputs`, and since
//! ADR-ebe85b3aa79d `vars`, `inherit_env` and `cwd`, and never `env`: `env`
//! left the verifier block (ADR-7224dea611c1, TASK-91cefb36af81), after
//! TASK-dc04757dfecb had parsed, validated and hashed it, and stays refused by
//! name. Every run is told its task, verifier, scope and base (GitHub #787).
//!
//! **Through the binary**, because every clause of the criterion is about it:
//! the file is refused by the parser every verb loads it through, the keys are
//! refused by `ank config`, and the hash is the one `check` compares with the
//! proof that anchored it.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The `cargo-test` verifier of this repository as TASK-dc04757dfecb ran it
/// at 6f7ceac, read back from that tree with `ank config`.
const CARGO_TEST_RUN: &str = "command -v cargo-nextest >/dev/null 2>&1 || { echo \"cargo-nextest is not installed: cargo install cargo-nextest --locked\" >&2; exit 127; }; cargo nextest run --workspace";

/// The hash TASK-dc04757dfecb recorded for that definition, `cargo-test@`
/// in its proof: the parser that hashed it still knew `env`, and a verifier
/// that never declared one has to come out of this one unmoved.
const DC04_CARGO_TEST_HASH: &str = "fcf98015afba";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("verifier-env-gitconfig");
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
        .env("ANK_AGENT", "test/verifier-env")
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

/// A corpus with one commit and one verifier, `t`, running `true`.
fn corpus(what: &str) -> PathBuf {
    let dir = scratch::dir(what);
    run("git", &dir, &["init", "-q", "-b", "main"]);
    fs::write(dir.join("f"), "x\n").unwrap();
    run(ANK, &dir, &["init"]);
    run(ANK, &dir, &["config", "default_branch", "main"]);
    run(ANK, &dir, &["config", "verifiers.t.run", "true"]);
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "init"]);
    dir
}

fn config_yml(dir: &Path) -> String {
    fs::read_to_string(dir.join(".ank/config.yml")).unwrap()
}

/// Written by hand, as ADR-d8a328e311b9 had it: no verb writes it any more,
/// and every verb refuses to load it, naming the key it does not know.
#[test]
fn a_config_yml_carrying_an_env_block_fails_to_load_and_names_env() {
    let dir = corpus("verifier-env-refused");
    let before = config_yml(&dir);
    fs::write(
        dir.join(".ank/config.yml"),
        before.replace(
            "    run: \"true\"\n",
            "    run: \"true\"\n    env:\n      platform: linux/amd64\n",
        ),
    )
    .unwrap();
    assert_ne!(config_yml(&dir), before, "the fixture wrote no env block");

    // `ank config` reads the text and not the parse, so that it still answers
    // on a file that does not load: the verbs that load it are the ones asked.
    for verb in [&["check"][..], &["status"], &["context"]] {
        let out = spawn(ANK, &dir, verb);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{verb:?} loaded an env block: {err}");
        assert!(
            err.contains("verifiers.t") && err.contains("unknown field `env`"),
            "{verb:?} must name env as the key it does not know:\n{err}"
        );
    }
}

#[test]
fn ank_config_refuses_every_env_key_by_name_and_lists_none() {
    let dir = corpus("verifier-env-keys");
    let before = config_yml(&dir);
    for (key, value) in [
        ("verifiers.t.env.image", "r@sha256:0123"),
        ("verifiers.t.env.platform", "linux/amd64"),
        ("verifiers.t.env.network", "none"),
        ("verifiers.t.env.vars.LANG", "C"),
    ] {
        for args in [
            &["config", key, value][..],
            &["config", key],
            &["config", "--unset", key],
        ] {
            let out = spawn(ANK, &dir, args);
            let err = String::from_utf8_lossy(&out.stderr);
            assert!(!out.status.success(), "{args:?} was accepted: {err}");
            assert!(
                err.contains(&format!("unknown key '{key}'")),
                "{args:?} must refuse {key} by name:\n{err}"
            );
        }
        assert_eq!(config_yml(&dir), before, "{key} moved config.yml");
    }

    // The help of `ank config` is `ank help config`: `--help` is no flag of
    // the verb, and its refusal lists flags, never keys.
    let help = run(ANK, &dir, &["help", "config"]);
    assert!(help.contains("verifiers.<name>.inputs"), "{help}");
    // `inherit_env` is a key of its own (ADR-ebe85b3aa79d); what stays
    // withdrawn is `env` as a key, a `.env` segment of a path.
    assert!(help.contains("verifiers.<name>.inherit_env"), "{help}");
    assert!(!help.contains(".env"), "ank help config names env:\n{help}");
    let out = spawn(ANK, &dir, &["config", "--help"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("env"), "ank config --help names env:\n{err}");
}

/// What `check` compares a proof's `name@hash` with is the definition in
/// force: a verifier declaring `run` and `timeout` keeps the hash the env
/// parser recorded for it, and `inputs` still moves it.
#[test]
fn the_definition_hash_covers_run_timeout_and_inputs_and_matches_dc04() {
    let dir = corpus("verifier-env-hash");
    let new = run(
        ANK,
        &dir,
        &[
            "new",
            "task",
            "--title",
            "closed by t",
            "--scope",
            "f",
            "--criteria",
            "t passes",
            "--verify",
            "t",
            "--json",
        ],
    );
    let at = new.find("TASK-").expect("new task --json names the id");
    let id = new[at..at + "TASK-".len() + 12].to_string();
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "new"]);
    run(ANK, &dir, &["claim", &id]);
    run(ANK, &dir, &["done"]);

    // The proof `done` wrote anchors `t@<hash of true>`. Rewritten to the
    // reference TASK-dc04757dfecb holds, so the definition compared with it is
    // that task's own: running cargo nextest inside a test is not the point.
    let task = dir.join(format!(".ank/entities/{id}.md"));
    let text = fs::read_to_string(&task).unwrap();
    let at = text.find("verifier: t@").expect("done anchored t");
    let anchored = &text[at..at + "verifier: t@".len() + 12];
    fs::write(
        &task,
        text.replace(
            anchored,
            &format!("verifier: cargo-test@{DC04_CARGO_TEST_HASH}"),
        ),
    )
    .unwrap();
    run(
        ANK,
        &dir,
        &["config", "--", "verifiers.cargo-test.run", CARGO_TEST_RUN],
    );
    run(
        ANK,
        &dir,
        &["config", "verifiers.cargo-test.timeout", "20m"],
    );
    run("git", &dir, &["add", "-A"]);
    run("git", &dir, &["commit", "-q", "-m", "done"]);

    let changed =
        format!("verifier 'cargo-test' changed since the proof ({DC04_CARGO_TEST_HASH} -> ");
    let report = String::from_utf8_lossy(&spawn(ANK, &dir, &["check", "--all"]).stdout).to_string();
    assert!(
        !report.contains(&changed),
        "run and timeout alone must keep the hash TASK-dc04757dfecb recorded:\n{report}"
    );

    run(
        ANK,
        &dir,
        &["config", "verifiers.cargo-test.inputs", r#"["f"]"#],
    );
    let report = String::from_utf8_lossy(&spawn(ANK, &dir, &["check", "--all"]).stdout).to_string();
    assert!(
        report.contains(&changed),
        "inputs no longer moves the hash:\n{report}"
    );
    run(
        ANK,
        &dir,
        &["config", "--unset", "verifiers.cargo-test.inputs"],
    );
    let report = String::from_utf8_lossy(&spawn(ANK, &dir, &["check", "--all"]).stdout).to_string();
    assert!(
        !report.contains(&changed),
        "after --unset inputs:\n{report}"
    );
}

/// The page a reader is handed names no env key and no image: it is generated
/// from the table `ank config` reads (ADR-2b6235b7ff81), and the CI recipe
/// beside it promises no environment the binary never declares.
#[test]
fn the_docs_name_no_env_key_and_no_image() {
    let docs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs");
    let keys = fs::read_to_string(docs.join("config-keys.md")).unwrap();
    assert!(keys.contains("`verifiers.<name>.inputs`"), "{keys}");
    assert!(
        !keys.contains(".env"),
        "docs/config-keys.md carries an env row"
    );
    let ci = fs::read_to_string(docs.join("ci.md")).unwrap();
    assert!(
        !ci.to_lowercase().contains("image"),
        "docs/ci.md names an image"
    );
}

// ---------------------------------------------------------------------------
// vars, inherit_env, cwd and what ank tells a run (ADR-ebe85b3aa79d, GitHub #787)
// ---------------------------------------------------------------------------

/// A path as `sh` takes it on the three platforms: Git for Windows' shell
/// reads `C:/x` and not `C:\x`.
fn sh_path(p: &Path) -> String {
    p.display().to_string().replace('\\', "/")
}

fn spawn_with(dir: &Path, args: &[&str], extra: &[(&str, &str)]) -> Output {
    let config = isolated_git_config();
    let home = scratch::root();
    let mut c = Command::new(ANK);
    c.args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("ANK_AGENT", "test/verifier-env")
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("TMPDIR", home)
        .env("TMP", home)
        .env("TEMP", home)
        .stdin(Stdio::null());
    for (k, v) in extra {
        c.env(k, v);
    }
    c.output().unwrap_or_else(|e| panic!("ank must run: {e}"))
}

fn ok(out: Output, what: &str) -> String {
    assert!(
        out.status.success(),
        "{what}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// `<root>/repo` is the repository, on a branch `work` one commit ahead of
/// `main`, and `<root>` is where a verifier writes what it saw: outside the
/// tree, so nothing it writes moves a content key.
struct Bench {
    root: PathBuf,
    repo: PathBuf,
}

/// The names a dump records, each into `<root>/<name>`, `UNSET` when unset.
const DUMPED: [&str; 8] = [
    "ANK_TASK",
    "ANK_VERIFIER",
    "ANK_SCOPE",
    "ANK_BASE",
    "FOO",
    "SECRET_TOKEN",
    "GIT_CONFIG_GLOBAL",
    "PATH",
];

impl Bench {
    fn new(what: &str) -> Bench {
        let root = scratch::dir(what);
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("sub")).unwrap();
        fs::create_dir_all(repo.join("g")).unwrap();
        fs::write(repo.join("f"), "x\n").unwrap();
        fs::write(repo.join("sub/k"), "k\n").unwrap();
        fs::write(repo.join("g/x"), "g\n").unwrap();
        run("git", &repo, &["init", "-q", "-b", "main"]);
        run(ANK, &repo, &["init"]);
        run(ANK, &repo, &["config", "default_branch", "main"]);
        // Only builtins, so that it runs with nothing of the caller, PATH
        // included.
        let mut dump: Vec<String> = DUMPED
            .iter()
            .map(|n| format!("printf '%s' \"${{{n}-UNSET}}\" > '{}/{n}'", sh_path(&root)))
            .collect();
        dump.push(format!("pwd -P > '{}/pwd'", sh_path(&root)));
        run(
            ANK,
            &repo,
            &["config", "--", "verifiers.dump.run", &dump.join("; ")],
        );
        run(ANK, &repo, &["config", "verifiers.dump.inputs", r#"["f"]"#]);
        run(
            ANK,
            &repo,
            &[
                "config",
                "--",
                "verifiers.count.run",
                &format!("echo x >> '{}/count'", sh_path(&root)),
            ],
        );
        run(
            ANK,
            &repo,
            &["config", "verifiers.count.inputs", r#"["f"]"#],
        );
        run("git", &repo, &["add", "-A"]);
        run("git", &repo, &["commit", "-q", "-m", "init"]);
        run("git", &repo, &["checkout", "-q", "-b", "work"]);
        fs::write(repo.join("f"), "y\n").unwrap();
        run("git", &repo, &["commit", "-q", "-am", "ahead"]);
        Bench { root, repo }
    }

    fn seen(&self, name: &str) -> String {
        fs::read_to_string(self.root.join(name))
            .unwrap_or_else(|e| panic!("the verifier wrote no {name}: {e}"))
    }

    fn count(&self) -> usize {
        fs::read_to_string(self.root.join("count"))
            .map(|t| t.lines().count())
            .unwrap_or(0)
    }

    fn config(&self, key: &str, value: &str) {
        run(ANK, &self.repo, &["config", key, value]);
    }

    /// `ank verify`, with `SECRET_TOKEN` exported by the caller.
    fn verify(&self) {
        ok(
            spawn_with(&self.repo, &["verify"], &[("SECRET_TOKEN", "s3cret")]),
            "ank verify",
        );
    }

    /// A task over `f` and `g/**` declaring `verifier`, committed, claimed and
    /// closed by `ank done` with `SECRET_TOKEN` exported by the caller.
    fn done(&self, verifier: &str, title: &str) -> String {
        let new = run(
            ANK,
            &self.repo,
            &[
                "new",
                "task",
                "--title",
                title,
                "--scope",
                "f",
                "--scope",
                "g/**",
                "--criteria",
                "it runs",
                "--verify",
                verifier,
                "--json",
            ],
        );
        let at = new.find("TASK-").expect("new task --json names the id");
        let id = new[at..at + "TASK-".len() + 12].to_string();
        run("git", &self.repo, &["add", "-A"]);
        run("git", &self.repo, &["commit", "-q", "-m", title]);
        run(ANK, &self.repo, &["claim", &id]);
        ok(
            spawn_with(&self.repo, &["done"], &[("SECRET_TOKEN", "s3cret")]),
            "ank done",
        );
        run("git", &self.repo, &["add", "-A"]);
        run("git", &self.repo, &["commit", "-q", "-m", "done"]);
        id
    }
}

#[test]
fn done_tells_a_run_its_task_verifier_scope_and_base_and_verify_leaves_task_and_scope_empty() {
    let b = Bench::new("verifier-vars-told");
    let main = run("git", &b.repo, &["rev-parse", "main"]);
    let id = b.done("dump", "told");
    assert_eq!(b.seen("ANK_TASK"), id, "the full id");
    assert_eq!(b.seen("ANK_VERIFIER"), "dump");
    assert_eq!(b.seen("ANK_SCOPE"), "f\ng/**", "one glob per line");
    assert_eq!(b.seen("ANK_BASE"), main, "the merge-base with main");
    assert_eq!(b.seen("FOO"), "UNSET");
    assert_eq!(
        b.seen("SECRET_TOKEN"),
        "s3cret",
        "absent inherit_env passes the whole environment, as before"
    );

    b.verify();
    assert_eq!(b.seen("ANK_TASK"), "", "no task under ank verify");
    assert_eq!(b.seen("ANK_SCOPE"), "", "no scope under ank verify");
    assert_eq!(b.seen("ANK_VERIFIER"), "dump");
    assert_eq!(b.seen("ANK_BASE"), main);
}

#[test]
fn vars_reach_the_run_and_inherit_env_decides_what_of_the_caller_does() {
    let b = Bench::new("verifier-vars-inherit");
    b.config("verifiers.dump.vars.FOO", "bar");
    b.verify();
    assert_eq!(b.seen("FOO"), "bar");
    assert_eq!(b.seen("SECRET_TOKEN"), "s3cret");

    b.config("verifiers.dump.inherit_env", "[PATH]");
    b.verify();
    assert_eq!(b.seen("SECRET_TOKEN"), "UNSET", "not listed, not passed");
    assert_eq!(b.seen("GIT_CONFIG_GLOBAL"), "UNSET");
    assert_ne!(b.seen("PATH"), "UNSET", "listed, passed");
    assert_eq!(b.seen("FOO"), "bar", "vars are ank's, not the caller's");

    b.config("verifiers.dump.inherit_env", "false");
    b.verify();
    assert_eq!(b.seen("SECRET_TOKEN"), "UNSET");
    assert_eq!(b.seen("GIT_CONFIG_GLOBAL"), "UNSET");
    assert_ne!(
        b.seen("PATH"),
        std::env::var("PATH").unwrap(),
        "false passes nothing of the caller, PATH included"
    );
    assert_eq!(b.seen("ANK_VERIFIER"), "dump", "ank's own still set");
    assert_eq!(b.seen("FOO"), "bar");

    run(
        ANK,
        &b.repo,
        &["config", "--unset", "verifiers.dump.inherit_env"],
    );
    b.verify();
    assert_eq!(b.seen("SECRET_TOKEN"), "s3cret", "absent again, all again");
}

#[test]
fn a_var_naming_an_ank_variable_is_refused_by_config_and_faulted_by_check() {
    let b = Bench::new("verifier-vars-ank-name");
    let before = config_yml(&b.repo);
    let out = spawn(
        ANK,
        &b.repo,
        &["config", "verifiers.dump.vars.ANK_TASK", "x"],
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "ANK_TASK was accepted: {err}");
    assert!(err.contains("ANK_"), "{err}");
    assert_eq!(config_yml(&b.repo), before, "nothing written");

    fs::write(
        b.repo.join(".ank/config.yml"),
        before.replace(
            "    inputs: [\"f\"]\n",
            "    inputs: [\"f\"]\n    vars:\n      ANK_SCOPE: x\n",
        ),
    )
    .unwrap();
    assert_ne!(config_yml(&b.repo), before, "the fixture wrote no vars");
    let out = spawn(ANK, &b.repo, &["check"]);
    let report = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(8), "{report}");
    assert!(
        report.contains("verifiers.dump.vars.ANK_SCOPE"),
        "check names the key:\n{report}"
    );
}

#[test]
fn cwd_moves_the_run_and_one_absent_or_outside_the_root_is_refused_and_faulted() {
    let b = Bench::new("verifier-vars-cwd");
    b.config("verifiers.dump.cwd", "sub");
    b.verify();
    let pwd = b.seen("pwd");
    assert!(
        pwd.trim_end().ends_with("repo/sub"),
        "the run's directory is <root>/sub: {pwd}"
    );

    let before = config_yml(&b.repo);
    for bad in ["nope", "..", "../repo", "sub/../..", "/tmp"] {
        let out = spawn(ANK, &b.repo, &["config", "verifiers.dump.cwd", bad]);
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "cwd {bad} was accepted: {err}");
        assert!(err.contains("verifiers.dump.cwd"), "{bad}: {err}");
        assert_eq!(config_yml(&b.repo), before, "cwd {bad} moved config.yml");
    }

    for bad in ["nope", "../out"] {
        fs::write(
            b.repo.join(".ank/config.yml"),
            before.replace("    cwd: sub\n", &format!("    cwd: {bad}\n")),
        )
        .unwrap();
        assert_ne!(config_yml(&b.repo), before, "the fixture wrote no cwd");
        let out = spawn(ANK, &b.repo, &["check"]);
        let report = String::from_utf8_lossy(&out.stdout);
        assert_eq!(out.status.code(), Some(8), "cwd {bad}: {report}");
        assert!(
            report.contains("verifiers.dump.cwd"),
            "check names the key for {bad}:\n{report}"
        );
    }
}

#[test]
fn changing_vars_inherit_env_or_cwd_makes_done_run_the_verifier_again() {
    let b = Bench::new("verifier-vars-rerun");
    b.done("count", "first");
    assert_eq!(b.count(), 1);
    b.done("count", "unchanged");
    assert_eq!(b.count(), 1, "the same definition and inputs are reused");

    b.config("verifiers.count.vars.FOO", "bar");
    b.done("count", "vars");
    assert_eq!(b.count(), 2, "vars changed the hash");

    b.config("verifiers.count.inherit_env", "[PATH]");
    b.done("count", "inherit_env");
    assert_eq!(b.count(), 3, "inherit_env changed the hash");

    b.config("verifiers.count.cwd", "sub");
    b.done("count", "cwd");
    assert_eq!(b.count(), 4, "cwd changed the hash");

    b.done("count", "unchanged again");
    assert_eq!(b.count(), 4);
}
