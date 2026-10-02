//! `show --json` carries `amends` and `amended_by` over an ADR
//! (TASK-5bb91a4cec00, ADR-9ee76b578257).
//!
//! Every case through the binary: both sides of the relation, both lists
//! present and empty on an ADR with neither, and a spec and a log entry
//! answering the shape they answered before.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("adr-amends-gitconfig");
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

fn spawn(program: &str) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

struct Repo(PathBuf);

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        let r = Repo(dir);
        r.git(&["init", "-q", "-b", "main"]);
        fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
        r.commit("seed");
        r
    }

    fn git(&self, args: &[&str]) {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.0)
            .current_dir(scratch::root())
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank(args);
        assert_eq!(
            code(&out),
            0,
            "ank {args:?}: {}{}",
            stdout(&out),
            stderr(&out)
        );
        stdout(&out)
    }

    /// `ank new adr` with `extra`, and the id it printed.
    fn adr(&self, title: &str, extra: &[&str]) -> String {
        let mut args = vec![
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            "A binding rule.",
        ];
        args.extend_from_slice(extra);
        self.ok(&args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    /// Committed, then ratified: `accept` works on the default branch and on
    /// what is committed there.
    fn accept(&self, id: &str) {
        self.commit(&format!("propose {id}"));
        self.ok(&["accept", id]);
    }
}

/// `ank show <id> --json`, parsed.
fn show_json(r: &Repo, id: &str) -> serde_yaml::Mapping {
    let said = r.ok(&["show", id, "--json"]);
    match serde_yaml::from_str(&said) {
        Ok(serde_yaml::Value::Mapping(m)) => m,
        other => panic!("show {id} --json is not an object: {other:?}\n{said}"),
    }
}

fn keys(doc: &serde_yaml::Mapping) -> Vec<String> {
    doc.keys()
        .map(|k| k.as_str().expect("a key is a string").to_string())
        .collect()
}

/// The ids of the edge list under `field`, which must be present.
fn ids(doc: &serde_yaml::Mapping, field: &str) -> Vec<String> {
    let list = doc
        .get(field)
        .unwrap_or_else(|| panic!("no `{field}` in {doc:?}"))
        .as_sequence()
        .unwrap_or_else(|| panic!("`{field}` is not a list: {doc:?}"));
    list.iter()
        .map(|e| {
            e.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("an edge of `{field}` carries no id: {e:?}"))
                .to_string()
        })
        .collect()
}

#[test]
fn show_json_carries_amends_on_both_sides_and_only_accepted_amendments() {
    let r = Repo::new("amends-json");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let other = r.adr("Another foundation", &[]);
    r.accept(&other);
    let b = r.adr("The first amendment", &["--amends", &a, "--amends", &other]);
    r.accept(&b);
    let c = r.adr("The second amendment", &["--amends", &a]);
    r.accept(&c);
    let pending = r.adr("A proposed amendment", &["--amends", &a]);

    // The amending side: every id it amends, in the order it declares them.
    let doc = show_json(&r, &b);
    assert_eq!(
        ids(&doc, "amends"),
        vec![a.clone(), other.clone()],
        "{doc:?}"
    );
    assert_eq!(ids(&doc, "amended_by"), Vec::<String>::new(), "{doc:?}");

    // The amended side: every accepted ADR amending it, and not the proposal.
    let doc = show_json(&r, &a);
    assert_eq!(ids(&doc, "amends"), Vec::<String>::new(), "{doc:?}");
    let mut by = vec![b.clone(), c.clone()];
    by.sort();
    assert_eq!(ids(&doc, "amended_by"), by, "{doc:?}");
    assert!(!ids(&doc, "amended_by").contains(&pending), "{doc:?}");

    // A proposal amends, and says so on its own page.
    let doc = show_json(&r, &pending);
    assert_eq!(ids(&doc, "amends"), vec![a.clone()], "{doc:?}");
}

#[test]
fn show_json_on_an_adr_with_neither_carries_both_empty() {
    let r = Repo::new("amends-json-neither");
    let a = r.adr("A lone decision", &[]);
    r.accept(&a);
    let doc = show_json(&r, &a);
    assert_eq!(
        keys(&doc),
        [
            "contract",
            "id",
            "coordination",
            "amends",
            "amended_by",
            "detached_proofs",
            "log_total",
            "log_shown",
            "log",
            "machinery",
            "content",
        ],
        "a consumer never tests for the key: {doc:?}"
    );
    assert_eq!(ids(&doc, "amends"), Vec::<String>::new());
    assert_eq!(ids(&doc, "amended_by"), Vec::<String>::new());
}

/// The shape a spec and a log entry answered before `amends` reached the
/// document, written out rather than read off the table it is checking.
const SHOW_OTHER_KEYS: [&str; 9] = [
    "contract",
    "id",
    "coordination",
    "detached_proofs",
    "log_total",
    "log_shown",
    "log",
    "machinery",
    "content",
];

#[test]
fn show_json_on_a_spec_and_a_log_entry_keeps_its_shape() {
    let r = Repo::new("amends-json-other");
    let a = r.adr("A decision", &[]);
    r.accept(&a);
    let b = r.adr("Its amendment", &["--amends", &a]);
    r.accept(&b);

    let spec = r
        .ok(&["new", "spec", "--title", "s", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .expect("created <id> <slug>")
        .to_string();
    let doc = show_json(&r, &spec);
    assert_eq!(keys(&doc), SHOW_OTHER_KEYS, "{doc:?}");

    let logged = r.ok(&["log", &a, "a reading of the decision"]);
    let entry = logged
        .split_whitespace()
        .find(|w| w.starts_with("LOG-"))
        .unwrap_or_else(|| panic!("log names the entry it wrote: {logged}"))
        .to_string();
    let doc = show_json(&r, &entry);
    assert_eq!(keys(&doc), SHOW_OTHER_KEYS, "{doc:?}");
}
