//! `ank hooks status`, through the binary (ADR-89c1e7146fe5, ADR-43cbb20dcbf1,
//! ADR-b108b24202ed, TASK-e74920bd2d0f).
//!
//! What it answers: for every agent `install` knows, whether its entries are in
//! that agent's user-level settings, the binary they call and whether that is
//! the binary answering; then `guard.corpus` and `guard.edit` as the corpus
//! declares them and as they are in force once `ANK_GUARD_*` is applied. What is
//! measured beside the answer is that it writes nothing: the settings file and
//! every file under `.ank/` are compared byte for byte around each run.
//! `CLAUDE_CONFIG_DIR` points every run at a fixture, and `HOME` at an empty
//! directory, so no run can read the settings of the machine it runs on.

use crate::scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

const AGENT: &str = "claude-code/1.0.0";

struct Fixture {
    base: PathBuf,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let base = scratch::dir(what);
        for d in ["claude", "home", "outside", "xdg/ank"] {
            fs::create_dir_all(base.join(d)).unwrap();
        }
        fs::write(base.join("xdg/ank/corpora.yml"), "schema: 1\ncorpora: {}\n").unwrap();
        fs::write(
            base.join("gitconfig"),
            "[commit]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n",
        )
        .unwrap();
        Fixture { base }
    }

    fn settings(&self) -> PathBuf {
        self.base.join("claude").join("settings.json")
    }

    fn outside(&self) -> PathBuf {
        self.base.join("outside")
    }

    fn command(&self, program: &str, dir: &Path, env: &[(&str, &str)]) -> Command {
        let mut c = Command::new(program);
        let gitconfig = self.base.join("gitconfig");
        c.current_dir(dir)
            .env("CLAUDE_CONFIG_DIR", self.base.join("claude"))
            .env("XDG_CONFIG_HOME", self.base.join("xdg"))
            .env("APPDATA", self.base.join("xdg"))
            .env("HOME", self.base.join("home"))
            .env("USERPROFILE", self.base.join("home"))
            .env("GIT_CONFIG_GLOBAL", &gitconfig)
            .env("GIT_CONFIG_SYSTEM", &gitconfig)
            .env("NO_COLOR", "1")
            .env("ANK_AGENT", AGENT)
            .env_remove("ANK_REPO")
            .env_remove("ANK_GUARD_CORPUS")
            .env_remove("ANK_GUARD_EDIT");
        for (k, v) in env {
            c.env(k, v);
        }
        c
    }

    fn ank(&self, dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Output {
        self.command(ANK, dir, env).args(args).output().unwrap()
    }

    /// A corpus `ank init` made, with `guard.edit` declared `warn` and
    /// `guard.corpus` left at its default.
    fn corpus(&self) -> PathBuf {
        let repo = self.base.join("repo");
        fs::create_dir_all(&repo).unwrap();
        let out = self
            .command("git", &repo, &[])
            .args(["init", "-q", "-b", "main", "."])
            .output()
            .unwrap();
        assert!(out.status.success(), "git init: {}", stderr(&out));
        ok(&self.ank(&repo, &["init"], &[]), "init");
        ok(
            &self.ank(&repo, &["config", "guard.edit", "warn"], &[]),
            "config guard.edit",
        );
        repo
    }
}

fn ok(out: &Output, what: &str) -> String {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{what}: {}\n{}",
        stderr(out),
        stdout(out)
    );
    stdout(out)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn parse(text: &str) -> serde_yaml::Value {
    serde_yaml::from_str(text).unwrap_or_else(|e| panic!("not JSON ({e}): {text}"))
}

/// Every file under `dir`, by relative path, with its bytes.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut all = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                all.insert(
                    p.strip_prefix(dir).unwrap().to_path_buf(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    all
}

fn this_binary() -> String {
    fs::canonicalize(ANK).unwrap().display().to_string()
}

fn claude(doc: &serde_yaml::Value) -> &serde_yaml::Value {
    doc["agents"]
        .as_sequence()
        .expect("agents is an array")
        .iter()
        .find(|a| a["agent"].as_str() == Some("claude"))
        .expect("claude is listed")
}

fn guard<'a>(doc: &'a serde_yaml::Value, key: &str) -> &'a serde_yaml::Value {
    doc["guards"]
        .as_sequence()
        .expect("guards is an array")
        .iter()
        .find(|g| g["key"].as_str() == Some(key))
        .unwrap_or_else(|| panic!("{key} is listed"))
}

/// Outside a corpus, with nothing installed: claude is listed as not
/// installed, and no corpus is in force, so no guard is listed.
#[test]
fn outside_a_corpus_it_lists_the_agents_and_says_no_corpus_is_in_force() {
    let fx = Fixture::new("hooks-status-outside");
    let out = fx.ank(&fx.outside(), &["hooks", "status"], &[]);
    let text = ok(&out, "status");
    assert!(text.contains("claude"), "{text}");
    assert!(text.contains("not installed"), "{text}");
    assert!(text.contains("no corpus is in force"), "{text}");
    assert!(!fx.settings().exists(), "status created the settings file");

    let doc = parse(&ok(
        &fx.ank(&fx.outside(), &["hooks", "status", "--json"], &[]),
        "status --json",
    ));
    let c = claude(&doc);
    assert_eq!(c["installed"].as_bool(), Some(false));
    assert!(c["binary"].is_null(), "{doc:?}");
    assert_eq!(c["current"].as_bool(), Some(false));
    assert!(doc["corpus"].is_null(), "{doc:?}");
    assert_eq!(doc["guards"].as_sequence().map(|g| g.len()), Some(0));
    // Every agent install knows, and generic is not one of them.
    let names: Vec<&str> = doc["agents"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|a| a["agent"].as_str())
        .collect();
    assert_eq!(names, vec!["claude"]);
}

/// Installed through the binary, the entries call this binary: status says
/// so, and neither the settings file nor `.ank/` moves a byte.
#[test]
fn installed_entries_name_this_binary_and_status_writes_nothing() {
    let fx = Fixture::new("hooks-status-installed");
    let repo = fx.corpus();
    ok(
        &fx.ank(&repo, &["hooks", "install", "claude"], &[]),
        "install",
    );
    let settings = fs::read(fx.settings()).unwrap();
    let corpus = snapshot(&repo.join(".ank"));

    let text = ok(&fx.ank(&repo, &["hooks", "status"], &[]), "status");
    assert!(text.contains("claude installed"), "{text}");
    assert!(text.contains(&this_binary()), "{text}");
    assert!(text.contains("this binary"), "{text}");
    let doc = parse(&ok(
        &fx.ank(&repo, &["hooks", "status", "--json"], &[]),
        "status --json",
    ));
    let c = claude(&doc);
    assert_eq!(c["installed"].as_bool(), Some(true));
    assert_eq!(c["binary"].as_str(), Some(this_binary().as_str()));
    assert_eq!(c["current"].as_bool(), Some(true));
    assert_eq!(
        c["events"].as_sequence().map(|e| e.len()),
        Some(2),
        "{doc:?}"
    );

    assert_eq!(fs::read(fx.settings()).unwrap(), settings, "settings moved");
    assert_eq!(snapshot(&repo.join(".ank")), corpus, ".ank/ moved");
}

/// Entries an install from another binary left: the path they call is named,
/// and it is not this binary.
#[test]
fn entries_calling_another_binary_are_named_and_not_this_one() {
    let fx = Fixture::new("hooks-status-moved");
    ok(
        &fx.ank(&fx.outside(), &["hooks", "install", "claude"], &[]),
        "install",
    );
    let quoted = format!("\\\"{}\\\"", this_binary().replace('\\', "\\\\"));
    let text = fs::read_to_string(fx.settings()).unwrap();
    assert!(text.contains(&quoted), "{text}");
    let moved = text.replace(&quoted, "\\\"/nowhere/old/ank\\\"");
    fs::write(fx.settings(), &moved).unwrap();

    let out = ok(&fx.ank(&fx.outside(), &["hooks", "status"], &[]), "status");
    assert!(out.contains("/nowhere/old/ank"), "{out}");
    assert!(out.contains("not this binary"), "{out}");
    let doc = parse(&ok(
        &fx.ank(&fx.outside(), &["hooks", "status", "--json"], &[]),
        "status --json",
    ));
    let c = claude(&doc);
    assert_eq!(c["installed"].as_bool(), Some(true));
    assert_eq!(c["binary"].as_str(), Some("/nowhere/old/ank"));
    assert_eq!(c["current"].as_bool(), Some(false));
    assert_eq!(fs::read_to_string(fx.settings()).unwrap(), moved);
}

/// The levels the corpus declares, and the ones in force: a tighter override
/// wins, a looser one is ignored and said to be.
#[test]
fn guards_are_listed_as_declared_and_as_in_force() {
    let fx = Fixture::new("hooks-status-guards");
    let repo = fx.corpus();
    let corpus = snapshot(&repo.join(".ank"));

    let doc = parse(&ok(
        &fx.ank(&repo, &["hooks", "status", "--json"], &[]),
        "status --json",
    ));
    assert!(doc["corpus"].as_str().is_some(), "{doc:?}");
    let edit = guard(&doc, "guard.edit");
    assert_eq!(edit["declared"].as_str(), Some("warn"));
    assert_eq!(edit["in_force"].as_str(), Some("warn"));
    assert!(
        edit["set"].is_null() && edit["ignored"].is_null(),
        "{doc:?}"
    );
    let corpus_guard = guard(&doc, "guard.corpus");
    assert_eq!(corpus_guard["declared"].as_str(), Some("deny"));
    assert_eq!(corpus_guard["in_force"].as_str(), Some("deny"));

    let env = [("ANK_GUARD_EDIT", "claim"), ("ANK_GUARD_CORPUS", "off")];
    let doc = parse(&ok(
        &fx.ank(&repo, &["hooks", "status", "--json"], &env),
        "status --json, overridden",
    ));
    let edit = guard(&doc, "guard.edit");
    assert_eq!(edit["declared"].as_str(), Some("warn"));
    assert_eq!(edit["in_force"].as_str(), Some("claim"));
    assert_eq!(edit["set"].as_str(), Some("claim"));
    assert!(edit["ignored"].is_null(), "{doc:?}");
    let corpus_guard = guard(&doc, "guard.corpus");
    assert_eq!(corpus_guard["in_force"].as_str(), Some("deny"));
    assert_eq!(corpus_guard["set"].as_str(), Some("off"));
    let why = corpus_guard["ignored"]
        .as_str()
        .expect("the override is ignored");
    assert!(why.contains("ANK_GUARD_CORPUS=off was ignored"), "{why}");

    let text = ok(&fx.ank(&repo, &["hooks", "status"], &env), "status");
    assert!(text.contains("guard.edit"), "{text}");
    assert!(text.contains("claim"), "{text}");
    assert!(text.contains("ANK_GUARD_CORPUS=off was ignored"), "{text}");
    assert_eq!(snapshot(&repo.join(".ank")), corpus, ".ank/ moved");
}

/// Status takes no agent: it answers for every one.
#[test]
fn status_refuses_an_argument() {
    let fx = Fixture::new("hooks-status-extra");
    let out = fx.ank(&fx.outside(), &["hooks", "status", "claude"], &[]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("ank hooks status"),
        "{}",
        stderr(&out)
    );
}

/// `ank help` lists the subcommand, and the help of hooks no longer says
/// claude is the only agent there is.
#[test]
fn help_lists_status_and_names_generic() {
    let fx = Fixture::new("hooks-status-help");
    let text = ok(&fx.ank(&fx.outside(), &["help", "hooks"], &[]), "help");
    assert!(text.contains("status"), "{text}");
    assert!(text.contains("generic"), "{text}");
    assert!(!text.contains("the only one this version"), "{text}");
    let doc = parse(&ok(
        &fx.ank(&fx.outside(), &["help", "--json"], &[]),
        "help --json",
    ));
    let hooks = doc["verbs"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|v| v["name"].as_str() == Some("hooks"))
        .expect("help --json carries hooks");
    let usage = hooks["usage"].as_str().expect("hooks has a usage");
    assert!(usage.contains("status"), "{usage}");
}

/// The document status answers under `--json`, pinned from inside a corpus
/// with the entries installed and one override of each kind, so that every
/// field is filled; the fixture's paths are named away.
#[test]
fn json_golden_hooks_status() {
    let fx = Fixture::new("hooks-status-golden");
    let repo = fx.corpus();
    ok(
        &fx.ank(&repo, &["hooks", "install", "claude"], &[]),
        "install",
    );
    let env = [("ANK_GUARD_EDIT", "claim"), ("ANK_GUARD_CORPUS", "off")];
    let out = fx.ank(&repo, &["hooks", "status", "--json"], &env);
    let mut text = ok(&out, "status --json");
    let doc = parse(&text);
    let corpus = doc["corpus"].as_str().unwrap().to_string();
    for (path, name) in [
        (fx.settings().display().to_string(), "<SETTINGS>"),
        (this_binary(), "<BINARY>"),
        (corpus, "<CORPUS>"),
    ] {
        text = text.replace(&path.replace('\\', "\\\\"), name);
    }
    crate::fixture::pin("hooks-status", &text);
}
