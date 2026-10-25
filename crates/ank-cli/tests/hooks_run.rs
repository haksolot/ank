//! `ank hooks run claude session-start`, through the binary, with the payload
//! Claude Code writes on its standard input (ADR-43cbb20dcbf1,
//! TASK-73d26747a04c).
//!
//! What the agent is served is measured against what `ank status` prints for
//! the same identity in the same corpus, byte for byte, rather than against a
//! line this file would have to keep in step with `status`. Silence is measured
//! the same way: an empty standard output and a 0, outside a corpus and on a
//! payload the hook cannot read, because a hook that fails is a harness that
//! blocks.
//!
//! The git processes are counted from `GIT_TRACE` at an absolute path
//! (ADR-cc65f1388a71), never timed.

use crate::scratch;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

const AGENT: &str = "hooks-run@fixture";

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// The payload Claude Code hands a SessionStart hook, as it documents it.
fn payload(cwd: &Path) -> String {
    format!(
        "{{\"session_id\":\"abc123\",\"transcript_path\":\"/tmp/t.jsonl\",\"cwd\":{},\
         \"hook_event_name\":\"SessionStart\",\"source\":\"startup\"}}",
        quoted(&cwd.to_string_lossy())
    )
}

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
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
        Fixture { root }
    }

    fn command(&self, program: &str, dir: &Path) -> Command {
        let mut c = Command::new(program);
        let gitconfig = self.root.join("gitconfig");
        let config = self.root.join("config");
        c.current_dir(dir)
            .env("XDG_CONFIG_HOME", &config)
            .env("APPDATA", &config)
            .env("HOME", self.root.join("home"))
            .env("USERPROFILE", self.root.join("home"))
            .env("GIT_CONFIG_GLOBAL", &gitconfig)
            .env("GIT_CONFIG_SYSTEM", &gitconfig)
            .env("NO_COLOR", "1")
            .env("ANK_AGENT", AGENT)
            .env_remove("GIT_TRACE")
            .env_remove("ANK_REPO");
        c
    }

    fn git(&self, dir: &Path, args: &[&str]) {
        let out = self.command("git", dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn ank(&self, dir: &Path, args: &[&str]) -> Output {
        self.command(ANK, dir).args(args).output().unwrap()
    }

    /// A corpus `ank init` made, committed, with a task this identity holds:
    /// so what `status` prints depends on who is asking.
    fn corpus(&self) -> PathBuf {
        let repo = self.root.join("repo");
        fs::create_dir_all(&repo).unwrap();
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
                    "src",
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

    fn hook(&self, dir: &Path, stdin: &[u8], trace: Option<&Path>) -> Output {
        let mut c = self.command(ANK, dir);
        c.args(["hooks", "run", "claude", "session-start"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(t) = trace {
            c.env("GIT_TRACE", t);
        }
        let mut child = c.spawn().unwrap();
        // A hook that does not read its payload closes the pipe under us:
        // that is not this test's failure, the output is.
        let _ = child.stdin.take().unwrap().write_all(stdin);
        child.wait_with_output().unwrap()
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn ok(out: Output, what: &str) -> String {
    assert_eq!(out.status.code(), Some(0), "{what}: {}", stderr(&out));
    String::from_utf8(out.stdout).unwrap()
}

/// The string value of `key` in a JSON text, decoded, wherever it sits.
fn string_at(json: &str, key: &str) -> Option<String> {
    let start = json.find(&format!("\"{key}\":\""))? + key.len() + 4;
    let mut chars = json[start..].chars();
    let mut out = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                'r' => out.push('\r'),
                'u' => {
                    let hex: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&hex, 16).ok()?)?);
                }
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
}

#[test]
fn inside_a_corpus_the_agent_is_served_its_status_and_the_contract_pointer() {
    let f = Fixture::new("hooks-run-inside");
    let repo = f.corpus();
    let status = ok(f.ank(&repo, &["status"]), "status");
    assert!(status.contains("TASK-"), "status names no claim: {status}");

    let out = f.hook(&repo, payload(&repo).as_bytes(), None);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let doc = stdout.trim_end();
    assert!(
        doc.starts_with('{') && doc.ends_with('}') && !doc.contains('\n'),
        "stdout is not one JSON object on one line: {stdout:?}"
    );
    assert!(
        doc.contains("\"hookSpecificOutput\":{"),
        "no hookSpecificOutput: {doc}"
    );
    assert_eq!(
        string_at(doc, "hookEventName").as_deref(),
        Some("SessionStart")
    );
    let context = string_at(doc, "additionalContext").expect("no additionalContext");
    assert!(
        context.contains(&status),
        "additionalContext does not hold what `ank status` prints.\nstatus:\n{status}\ncontext:\n{context}"
    );
    assert!(context.contains("ank context"), "{context}");
    assert!(context.contains("ank skill"), "{context}");
}

#[test]
fn outside_any_corpus_the_hook_is_silent_and_exits_zero() {
    let f = Fixture::new("hooks-run-outside");
    let bare = f.root.join("bare");
    fs::create_dir_all(&bare).unwrap();
    f.git(&bare, &["init", "-q", "."]);
    let plain = f.root.join("plain");
    fs::create_dir_all(&plain).unwrap();
    for dir in [&bare, &plain] {
        let out = f.hook(dir, payload(dir).as_bytes(), None);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert_eq!(String::from_utf8_lossy(&out.stdout), "", "in {dir:?}");
        assert_eq!(stderr(&out), "", "in {dir:?}");
    }
}

#[test]
fn a_payload_the_hook_cannot_read_lets_the_session_through_silently() {
    let f = Fixture::new("hooks-run-malformed");
    let repo = f.corpus();
    let payloads: [&[u8]; 5] = [
        b"",
        b"not json at all",
        b"{\"hook_event_name\":\"SessionStart\"",
        b"[1,2,3]",
        b"{\"cwd\":\"\xff\xfe\"}",
    ];
    for p in payloads {
        let out = f.hook(&repo, p, None);
        assert_eq!(
            out.status.code(),
            Some(0),
            "payload {:?}: {}",
            String::from_utf8_lossy(p),
            stderr(&out)
        );
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            "",
            "payload {:?}",
            String::from_utf8_lossy(p)
        );
    }
}

#[test]
fn the_hook_starts_no_more_git_processes_than_status() {
    let f = Fixture::new("hooks-run-count");
    let repo = f.corpus();
    // Warm: the index a first reader builds is not what either verb costs.
    ok(f.ank(&repo, &["status"]), "status");

    let count = |trace: &Path| -> usize {
        let text = fs::read_to_string(trace).unwrap_or_default();
        text.lines().filter(|l| l.contains(MARK)).count()
    };
    let status_trace = f.root.join("status.trace");
    let out = f
        .command(ANK, &repo)
        .arg("status")
        .env("GIT_TRACE", &status_trace)
        .output()
        .unwrap();
    ok(out, "status");
    let hook_trace = f.root.join("hook.trace");
    let out = f.hook(&repo, payload(&repo).as_bytes(), Some(&hook_trace));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!out.stdout.is_empty(), "the hook served nothing");

    let (status, hook) = (count(&status_trace), count(&hook_trace));
    assert!(status > 0, "the trace records no git process for status");
    assert!(
        hook <= status,
        "the hook starts {hook} git processes where status starts {status}"
    );
}
