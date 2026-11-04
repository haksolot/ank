//! A verifier declares `run`, `timeout` and `inputs`, and nothing else: `env`
//! left the verifier block (ADR-7224dea611c1, TASK-91cefb36af81), after
//! TASK-dc04757dfecb had parsed, validated and hashed it.
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
    assert!(!help.contains("env"), "ank help config names env:\n{help}");
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
