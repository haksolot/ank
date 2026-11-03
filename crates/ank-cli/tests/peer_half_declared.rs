//! A binding declared at one end only is named by `check`, from the end that
//! can see the other (TASK-971e568f4d7f).
//!
//! A scope entry `web:src/**` in `api` binds `web` only when both ends declare
//! each other: `api` resolves `web` to know where the files are, and `web`
//! reads `api` through its own `peers.api` to be served the rule at all
//! (LOG-9a80124cc6ad). Forgetting one of the two used to be silent: `context`
//! in `web` answered as though nothing bound it. Each end that resolves its own
//! declaration can read the other's `config.yml` at its default branch, so each
//! end can say what the other is missing — a signal, never a fault, since the
//! repair lives in another repository, and nothing is written there.
//!
//! **Through the binary, in a scratch pair `api`/`web`**, with a reader whose
//! home is the test's own so that no override on the machine running it takes
//! part.

use crate::scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-half-declared";
const KIND: &str = "peer-half-declared";

struct Reader {
    home: PathBuf,
}

impl Reader {
    fn run(&self, dir: &Path, args: &[&str]) -> Output {
        let root = scratch::root();
        Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", &self.home)
            .env("APPDATA", &self.home)
            .output()
            .expect("the binary runs")
    }

    fn ok(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.run(dir, args);
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// `check` in `dir`: the lines of the finding kind under test, after
    /// asserting the exit code is 0 — a signal, and nothing else in these
    /// corpora that could turn it red.
    fn half_declared(&self, dir: &Path) -> Vec<String> {
        let out = self.run(dir, &["check"]);
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(
            out.status.code(),
            Some(0),
            "check in {}:\n{stdout}{stderr}",
            dir.display()
        );
        let kinded = self.run(dir, &["check", "--kind", KIND]);
        assert_eq!(kinded.status.code(), Some(0));
        let json = self.run(dir, &["check", "--json"]);
        let json = String::from_utf8_lossy(&json.stdout).to_string();
        let tagged = json.matches(&format!("\"kind\":\"{KIND}\"")).count();
        let lines: Vec<String> = String::from_utf8_lossy(&kinded.stdout)
            .lines()
            .filter(|l| l.contains("peer"))
            .map(str::to_string)
            .collect();
        assert_eq!(
            tagged,
            lines.len(),
            "--json and the page agree:\n{json}\n{lines:?}"
        );
        lines
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            message,
        ],
    );
}

/// A corpus with one commit, since a peer is read at its default branch
/// (ADR-09b985a5627e).
fn corpus(reader: &Reader, dir: &Path, peers: &[(&str, &str)], adrs: &[&str]) -> Vec<String> {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src").join("x.rs"), "").unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    reader.ok(dir, &["init"]);
    reader.ok(dir, &["config", "default_branch", "main"]);
    for (name, path) in peers {
        reader.ok(dir, &["config", &format!("peers.{name}"), path]);
    }
    let mut ids = Vec::new();
    for glob in adrs {
        let out = reader.ok(
            dir,
            &[
                "new",
                "adr",
                "--title",
                &format!("Scoped {glob}"),
                "--scope",
                glob,
                "--constraint",
                "A rule.",
                "--body",
                "Why.",
            ],
        );
        let id = out
            .split_whitespace()
            .find(|w| w.starts_with("ADR-"))
            .expect("new names the ADR")
            .to_string();
        ids.push(id);
    }
    commit(dir, "init");
    ids
}

/// Every file under `dir`, `.git` included, with its bytes: what "no file
/// under the other repository is modified" is measured against.
fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.insert(p.clone(), fs::read(&p).unwrap());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, &mut out);
    out
}

#[test]
fn check_names_a_binding_declared_at_one_end_from_either_end() {
    let base = scratch::dir("peer-half-declared");
    let reader = Reader {
        home: base.join("home"),
    };
    let api = base.join("api");
    let web = base.join("web");

    // `api` declares `web` and scopes a rule into it; `web` declares nobody.
    corpus(&reader, &web, &[], &[]);
    let bound = corpus(&reader, &api, &[("web", "../web")], &["web:src/**"]).remove(0);
    let before = snapshot(&web);
    let lines = reader.half_declared(&api);
    assert_eq!(lines.len(), 1, "one signal in api: {lines:?}");
    let line = &lines[0];
    assert!(line.contains(&bound), "names the ADR: {line}");
    assert!(
        !line.contains("LOG-"),
        "names the rules and never the entries recording them: {line}"
    );
    assert!(line.contains("'web'"), "names the peer: {line}");
    assert!(
        line.contains("ank --repo ../web config peers.api ../api"),
        "names the command that completes the binding in web: {line}"
    );
    assert_eq!(snapshot(&web), before, "nothing under web was written");

    // `web` declares `api`; `api` declares nobody that resolves to `web`.
    let _ = fs::remove_dir_all(&api);
    let _ = fs::remove_dir_all(&web);
    corpus(&reader, &api, &[], &["web:src/**"]);
    corpus(&reader, &web, &[("api", "../api")], &[]);
    let before = snapshot(&api);
    let lines = reader.half_declared(&web);
    assert_eq!(lines.len(), 1, "one signal in web: {lines:?}");
    let line = &lines[0];
    assert!(line.contains("'api'"), "names the peer: {line}");
    assert!(
        line.contains("declares no peer resolving back to this corpus"),
        "names the missing back-declaration: {line}"
    );
    assert!(
        line.contains("ank --repo ../api config peers.web ../web"),
        "names the command that completes it: {line}"
    );
    assert_eq!(snapshot(&api), before, "nothing under api was written");

    // A declaration resolving somewhere else is not a back-declaration.
    let third = base.join("third");
    corpus(&reader, &third, &[], &[]);
    reader.ok(&api, &["config", "peers.web", "../third"]);
    commit(&api, "web names the wrong repository");
    assert_eq!(reader.half_declared(&web).len(), 1);

    // Both declarations in place: neither end says anything about the binding.
    reader.ok(&api, &["config", "peers.web", "../web"]);
    commit(&api, "web declared");
    assert!(reader.half_declared(&web).is_empty());
    assert!(reader.half_declared(&api).is_empty());

    let _ = fs::remove_dir_all(&base);
}

/// The consequence agrees its verb with the number of decisions it names
/// (TASK-2dc2d8411e12): one ADR `binds`, several `bind`.
#[test]
fn the_signal_says_binds_for_one_decision_and_bind_for_several() {
    let base = scratch::dir("peer-half-declared-verb");
    let reader = Reader {
        home: base.join("home"),
    };
    let api = base.join("api");
    let web = base.join("web");

    corpus(&reader, &web, &[], &[]);
    let one = corpus(&reader, &api, &[("web", "../web")], &["web:src/**"]).remove(0);
    let lines = reader.half_declared(&api);
    assert_eq!(lines.len(), 1, "one signal in api: {lines:?}");
    assert!(
        lines[0].contains(&format!("so {one} binds nothing there (")),
        "one decision binds: {}",
        lines[0]
    );

    let _ = fs::remove_dir_all(&api);
    let _ = fs::remove_dir_all(&web);
    corpus(&reader, &web, &[], &[]);
    let mut two = corpus(
        &reader,
        &api,
        &[("web", "../web")],
        &["web:src/**", "web:src/x.rs"],
    );
    two.sort();
    let lines = reader.half_declared(&api);
    assert_eq!(lines.len(), 1, "one signal in api: {lines:?}");
    assert!(
        lines[0].contains(&format!("so {}, {} bind nothing there (", two[0], two[1])),
        "several decisions bind: {}",
        lines[0]
    );

    let _ = fs::remove_dir_all(&base);
}
