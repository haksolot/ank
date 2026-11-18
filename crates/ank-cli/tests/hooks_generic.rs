//! `ank hooks run generic <event>`, through the binary: the hook protocol
//! every agent `run` handles translates onto, read and written unchanged
//! (ADR-b108b24202ed, TASK-911fe94dd281).
//!
//! On stdin one JSON document, `event`, `tool`, `paths` and `cwd`; on stdout
//! one decision, `allow`, `deny` with a `reason`, or `context` with the `text`
//! served at session start; always a 0. What `claude` reads and writes is
//! pinned by `hooks_run.rs` and `hooks_guard.rs`, which this change leaves as
//! they were: they passing is the evidence that `claude` is a translation onto
//! this protocol and decides nothing of its own.
//!
//! Declared from `hooks_run.rs` by path, so the suite stays one `mod` line in
//! `main.rs` per file it already had.

use crate::scratch;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

const AGENT: &str = "hooks-generic@fixture";

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The protocol's pre-tool-use document for `tool` reaching `paths` from `cwd`.
fn call(cwd: &Path, tool: &str, paths: &[&str]) -> String {
    let paths: Vec<String> = paths.iter().map(|p| quoted(p)).collect();
    format!(
        "{{\"event\":\"pre-tool-use\",\"tool\":\"{tool}\",\"paths\":[{}],\"cwd\":{}}}",
        paths.join(","),
        quoted(&cwd.to_string_lossy())
    )
}

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(what: &str) -> Fixture {
        let root = scratch::dir(what);
        let config = root.join("config");
        fs::create_dir_all(config.join("ank")).unwrap();
        fs::write(
            config.join("ank").join("corpora.yml"),
            "schema: 1\ncorpora: {}\n",
        )
        .unwrap();
        fs::write(
            root.join("gitconfig"),
            "[commit]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
        )
        .unwrap();
        fs::create_dir_all(root.join("home")).unwrap();
        fs::create_dir_all(root.join("claude")).unwrap();
        Fixture { root }
    }

    fn command(&self, dir: &Path, program: &str) -> Command {
        let mut c = Command::new(program);
        let gitconfig = self.root.join("gitconfig");
        let config = self.root.join("config");
        c.current_dir(dir)
            .env("XDG_CONFIG_HOME", &config)
            .env("APPDATA", &config)
            .env("HOME", self.root.join("home"))
            .env("USERPROFILE", self.root.join("home"))
            .env("CLAUDE_CONFIG_DIR", self.root.join("claude"))
            .env("GIT_CONFIG_GLOBAL", &gitconfig)
            .env("GIT_CONFIG_SYSTEM", &gitconfig)
            .env("NO_COLOR", "1")
            .env("ANK_AGENT", AGENT)
            .env_remove("GIT_TRACE")
            .env_remove("ANK_REPO")
            .env_remove("ANK_GUARD_CORPUS")
            .env_remove("ANK_GUARD_EDIT")
            .env_remove("ANK_PERMISSIONS_ENFORCE");
        c
    }

    fn git(&self, dir: &Path, args: &[&str]) {
        let out = self.command(dir, "git").args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn ank(&self, dir: &Path, args: &[&str]) -> Output {
        self.command(dir, ANK).args(args).output().unwrap()
    }

    /// A committed corpus with a task scoped `src/**` this identity claims.
    fn corpus(&self) -> PathBuf {
        let repo = self.root.join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(repo.join("src").join("lib.rs"), "// seed\n").unwrap();
        fs::write(repo.join("docs").join("page.md"), "# seed\n").unwrap();
        self.git(&repo, &["init", "-q", "-b", "main", "."]);
        ok(self.ank(&repo, &["init"]), "init");
        let made = ok(
            self.ank(
                &repo,
                &[
                    "new",
                    "task",
                    "--title",
                    "a task the agent holds",
                    "--scope",
                    "src/**",
                    "--criteria",
                    "it holds",
                    "--no-verify",
                ],
            ),
            "new task",
        );
        let id = made
            .split_whitespace()
            .find(|w| w.starts_with("TASK-"))
            .unwrap_or_else(|| panic!("new names no task: {made}"))
            .to_string();
        self.git(&repo, &["add", "-A"]);
        self.git(&repo, &["commit", "-q", "-m", "corpus"]);
        ok(self.ank(&repo, &["claim", &id]), "claim");
        repo
    }

    fn set(&self, repo: &Path, key: &str, value: &str) {
        ok(self.ank(repo, &["config", key, value]), "config");
    }

    /// `ank hooks run generic <event>` from `dir` with `stdin`, its exit
    /// asserted 0 and its standard output returned.
    fn hook(&self, dir: &Path, event: &str, stdin: &str) -> String {
        let mut child = self
            .command(dir, ANK)
            .args(["hooks", "run", "generic", event])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let _ = child.stdin.take().unwrap().write_all(stdin.as_bytes());
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0), "{stdin}: {}", stderr(&out));
        String::from_utf8(out.stdout).unwrap()
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn ok(out: Output, what: &str) -> String {
    assert_eq!(out.status.code(), Some(0), "{what}: {}", stderr(&out));
    String::from_utf8(out.stdout).unwrap()
}

/// The decision a document carries, and the one other member it has, if any.
fn decision(stdout: &str) -> (String, Option<(String, String)>) {
    let line = stdout.trim_end();
    assert_eq!(
        stdout.lines().count(),
        1,
        "one document on one line: {stdout:?}"
    );
    let prefix = "{\"decision\":\"";
    assert!(line.starts_with(prefix), "{line}");
    let rest = &line[prefix.len()..];
    let (name, rest) = rest.split_once('"').unwrap();
    if rest == "}" {
        return (name.to_string(), None);
    }
    let rest = rest.strip_prefix(",\"").expect(line);
    let (key, rest) = rest.split_once("\":\"").expect(line);
    let value = rest.strip_suffix("\"}").expect(line);
    (name.to_string(), Some((key.to_string(), value.to_string())))
}

#[test]
fn a_file_under_the_corpus_is_denied_and_one_outside_it_allowed() {
    let f = Fixture::new("hooks-generic-corpus");
    let repo = f.corpus();
    let config = repo.join(".ank").join("config.yml");
    for tool in ["read", "edit", "search"] {
        for target in [
            ".ank/config.yml".to_string(),
            config.to_string_lossy().into(),
        ] {
            let out = f.hook(&repo, "pre-tool-use", &call(&repo, tool, &[&target]));
            let (name, member) = decision(&out);
            assert_eq!(name, "deny", "{tool} {target}: {out}");
            let (key, reason) = member.unwrap();
            assert_eq!(key, "reason");
            assert!(
                reason.contains(".ank/config.yml is under the corpus directory"),
                "{reason}"
            );
        }
    }
    let out = f.hook(
        &repo,
        "pre-tool-use",
        &call(&repo, "read", &["docs/page.md"]),
    );
    assert_eq!(out, "{\"decision\":\"allow\"}\n");
    let out = f.hook(&repo, "pre-tool-use", &call(&repo, "edit", &["src/lib.rs"]));
    assert_eq!(out, "{\"decision\":\"allow\"}\n");

    f.set(&repo, "guard.corpus", "off");
    let out = f.hook(
        &repo,
        "pre-tool-use",
        &call(&repo, "read", &[".ank/config.yml"]),
    );
    assert_eq!(out, "{\"decision\":\"allow\"}\n");
}

#[test]
fn under_guard_edit_claim_an_uncovered_edit_is_denied_and_a_covered_one_allowed() {
    let f = Fixture::new("hooks-generic-edit");
    let repo = f.corpus();
    f.set(&repo, "guard.edit", "claim");
    let out = f.hook(
        &repo,
        "pre-tool-use",
        &call(&repo, "edit", &["docs/page.md"]),
    );
    let (name, member) = decision(&out);
    assert_eq!(name, "deny", "{out}");
    let (key, reason) = member.unwrap();
    assert_eq!(key, "reason");
    assert!(
        reason.contains(&format!("no live claim of {AGENT} covers docs/page.md")),
        "{reason}"
    );
    let out = f.hook(&repo, "pre-tool-use", &call(&repo, "edit", &["src/lib.rs"]));
    assert_eq!(out, "{\"decision\":\"allow\"}\n");
    // Reading is not an edit, at any level.
    let out = f.hook(
        &repo,
        "pre-tool-use",
        &call(&repo, "read", &["docs/page.md"]),
    );
    assert_eq!(out, "{\"decision\":\"allow\"}\n");
}

#[test]
fn under_guard_edit_warn_an_uncovered_edit_is_allowed_with_a_message() {
    let f = Fixture::new("hooks-generic-warn");
    let repo = f.corpus();
    let out = f.hook(
        &repo,
        "pre-tool-use",
        &call(&repo, "edit", &["docs/page.md"]),
    );
    let (name, member) = decision(&out);
    assert_eq!(name, "allow", "{out}");
    let (key, message) = member.unwrap();
    assert_eq!(key, "message");
    assert!(message.contains("covers docs/page.md"), "{message}");
}

#[test]
fn session_start_serves_the_status_as_context() {
    let f = Fixture::new("hooks-generic-session");
    let repo = f.corpus();
    let status = ok(f.ank(&repo, &["status"]), "status");
    let doc = format!(
        "{{\"event\":\"session-start\",\"cwd\":{}}}",
        quoted(&repo.to_string_lossy())
    );
    let out = f.hook(&repo, "session-start", &doc);
    let (name, member) = decision(&out);
    assert_eq!(name, "context", "{out}");
    let (key, text) = member.unwrap();
    assert_eq!(key, "text");
    let first = status.lines().next().unwrap();
    assert!(text.starts_with(first), "{text}");
    assert!(text.contains("ank context <path>"), "{text}");
}

#[test]
fn outside_a_corpus_the_hook_writes_nothing() {
    let f = Fixture::new("hooks-generic-outside");
    let plain = f.root.join("plain");
    fs::create_dir_all(&plain).unwrap();
    let out = f.hook(
        &plain,
        "pre-tool-use",
        &call(&plain, "read", &[".ank/config.yml"]),
    );
    assert_eq!(out, "");
    let doc = format!(
        "{{\"event\":\"session-start\",\"cwd\":{}}}",
        quoted(&plain.to_string_lossy())
    );
    assert_eq!(f.hook(&plain, "session-start", &doc), "");
}

#[test]
fn malformed_input_writes_nothing_and_exits_zero() {
    let f = Fixture::new("hooks-generic-malformed");
    let repo = f.corpus();
    for stdin in [
        "",
        "not json",
        "[]",
        "{\"event\":\"pre-tool-use\"}",
        "{\"event\":\"pre-tool-use\",\"tool\":\"read\",\"paths\":\".ank/config.yml\"}",
        "{\"event\":\"pre-tool-use\",\"tool\":\"read\",\"paths\":[1]}",
        "{\"event\":\"session-start\",\"tool\":\"read\",\"paths\":[\".ank/config.yml\"]}",
    ] {
        assert_eq!(f.hook(&repo, "pre-tool-use", stdin), "", "{stdin}");
    }
    assert_eq!(f.hook(&repo, "no-such-event", "{}"), "");
}

#[test]
fn install_and_remove_refuse_generic_naming_that_it_is_wired_by_hand() {
    let f = Fixture::new("hooks-generic-install");
    let dir = f.root.clone();
    for verb in ["install", "remove"] {
        let out = f.ank(&dir, &["hooks", verb, "generic"]);
        assert_ne!(out.status.code(), Some(0), "{verb}");
        assert!(out.stdout.is_empty(), "{verb}");
        let err = stderr(&out);
        assert!(err.contains("generic"), "{err}");
        assert!(err.contains("by hand"), "{err}");
    }
    assert!(
        !f.root.join("claude").join("settings.json").exists(),
        "nothing written"
    );
}
