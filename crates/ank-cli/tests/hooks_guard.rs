//! `ank hooks run claude pre-tool-use`, through the binary, with the payloads
//! Claude Code writes on its standard input (ADR-43cbb20dcbf1,
//! TASK-582f6758fb0b).
//!
//! Two guards, each read from `.ank/config.yml` and each tightened, never
//! loosened, by the launch environment: `guard.corpus` refuses a file tool
//! aimed under the corpus directory, `guard.edit` speaks about an edit no live
//! claim of the identity covers. What is asserted is what Claude Code reads: a
//! `permissionDecision` of `deny` on standard output, a message that names the
//! path, or nothing at all, always with a 0.
//!
//! The git processes are counted from `GIT_TRACE` at an absolute path
//! (ADR-cc65f1388a71), never timed.

use crate::scratch;

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

const AGENT: &str = "hooks-guard@fixture";

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// The tools that read or search, and the tools that write.
const READERS: &[&str] = &["Read", "Glob", "Grep"];
const EDITORS: &[&str] = &["Edit", "MultiEdit", "Write", "NotebookEdit"];

fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The payload Claude Code hands a PreToolUse hook for `tool` aimed at
/// `target`, with `tool_input` shaped as that tool's own input is.
fn payload(cwd: &Path, tool: &str, target: &str) -> String {
    let t = quoted(target);
    let input = match tool {
        "Read" => format!("{{\"file_path\":{t}}}"),
        "Write" => format!("{{\"file_path\":{t},\"content\":\"x\"}}"),
        "Edit" => format!("{{\"file_path\":{t},\"old_string\":\"a\",\"new_string\":\"b\"}}"),
        "MultiEdit" => format!(
            "{{\"file_path\":{t},\"edits\":[{{\"old_string\":\"a\",\"new_string\":\"b\"}}]}}"
        ),
        "NotebookEdit" => format!("{{\"notebook_path\":{t},\"new_source\":\"x\"}}"),
        "Glob" => format!("{{\"pattern\":\"*.md\",\"path\":{t}}}"),
        "Grep" => format!("{{\"pattern\":\"status\",\"path\":{t}}}"),
        other => panic!("no payload shape for {other}"),
    };
    format!(
        "{{\"session_id\":\"abc123\",\"transcript_path\":\"/tmp/t.jsonl\",\"cwd\":{},\
         \"permission_mode\":\"default\",\"hook_event_name\":\"PreToolUse\",\
         \"tool_name\":\"{tool}\",\"tool_input\":{input}}}",
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
            "[commit]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n",
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
            .env_remove("ANK_REPO")
            .env_remove("ANK_GUARD_CORPUS")
            .env_remove("ANK_GUARD_EDIT")
            .env_remove("MSYS_NO_PATHCONV")
            .env_remove("MSYS2_ARG_CONV_EXCL");
        c
    }

    fn git(&self, dir: &Path, args: &[&str]) {
        let out = self.command("git", dir).args(args).output().unwrap();
        assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
    }

    fn ank(&self, dir: &Path, args: &[&str]) -> Output {
        self.command(ANK, dir).args(args).output().unwrap()
    }

    /// A corpus `ank init` made, committed, with a task scoped to `src/**`
    /// that this identity holds a live claim on, and `extra` more tasks
    /// written beside it so that the corpus has a size.
    fn corpus(&self, name: &str, extra: usize) -> PathBuf {
        let repo = self.root.join(name);
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
        // Written rather than created through `new`: the size is the
        // measurement's, and five hundred invocations of the binary would
        // make building the fixture the experiment.
        let entities = repo.join(".ank").join("entities");
        for i in 0..extra {
            let id = format!("TASK-{:012x}", 0xa000_0000_0000_u64 + i as u64);
            fs::write(
                entities.join(format!("{id}.md")),
                format!(
                    "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
                     created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                     blocked_by: []\ndone_criteria: |\n  It works.\ncriteria_by: creator\n\
                     schema: 1\nversion: 1\n---\n\nFree body.\n"
                ),
            )
            .unwrap();
        }
        self.git(&repo, &["add", "-A"]);
        self.git(&repo, &["commit", "-q", "-m", "corpus"]);
        ok(self.ank(&repo, &["claim", &id]), "claim");
        repo
    }

    fn set(&self, repo: &Path, key: &str, value: &str) {
        ok(self.ank(repo, &["config", key, value]), "config");
    }

    fn hook(&self, dir: &Path, stdin: &[u8], env: &[(&str, &str)], trace: Option<&Path>) -> Output {
        let mut c = self.command(ANK, dir);
        c.args(["hooks", "run", "claude", "pre-tool-use"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in env {
            c.env(k, v);
        }
        if let Some(t) = trace {
            c.env("GIT_TRACE", t);
        }
        let mut child = c.spawn().unwrap();
        let _ = child.stdin.take().unwrap().write_all(stdin);
        child.wait_with_output().unwrap()
    }

    /// The hook for `tool` aimed at `target` from `repo`, its exit asserted 0
    /// and its standard output returned.
    fn ask(&self, repo: &Path, tool: &str, target: &str, env: &[(&str, &str)]) -> String {
        let out = self.hook(repo, payload(repo, tool, target).as_bytes(), env, None);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{tool} {target}: {}",
            stderr(&out)
        );
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

fn denies(stdout: &str) -> bool {
    stdout.contains("\"permissionDecision\":\"deny\"")
}

/// The corpus directory reached three ways: relative to the session's
/// directory, absolute, and through a symbolic link the fixture makes, when the
/// platform lets it make one.
fn corpus_paths(repo: &Path, under: &str) -> Vec<String> {
    let mut paths = vec![
        format!(".ank/{under}"),
        repo.join(".ank").join(under).to_string_lossy().into_owned(),
    ];
    let link = repo.join("link");
    #[cfg(unix)]
    let made = std::os::unix::fs::symlink(repo.join(".ank"), &link).is_ok() || link.exists();
    #[cfg(windows)]
    let made = std::os::windows::fs::symlink_dir(repo.join(".ank"), &link).is_ok() || link.exists();
    if made {
        paths.push(format!("link/{under}"));
    }
    paths
}

/// What each tool is aimed at under the corpus: a file for the tools that take
/// one, a directory for the tools that search.
fn under_corpus(tool: &str) -> &'static str {
    match tool {
        "Glob" | "Grep" => "entities",
        _ => "config.yml",
    }
}

#[test]
fn every_file_tool_aimed_under_the_corpus_is_denied_by_default_and_under_deny() {
    let f = Fixture::new("hooks-guard-corpus-deny");
    let repo = f.corpus("repo", 0);
    for declared in [None, Some("deny")] {
        if let Some(level) = declared {
            f.set(&repo, "guard.corpus", level);
        }
        for tool in READERS.iter().chain(EDITORS) {
            for target in corpus_paths(&repo, under_corpus(tool)) {
                let out = f.ask(&repo, tool, &target, &[]);
                assert!(
                    denies(&out),
                    "guard.corpus {declared:?}: {tool} {target} is not denied: {out:?}"
                );
                assert!(
                    out.contains("ank show") && out.contains("ank find"),
                    "the reason does not name the CLI: {out}"
                );
            }
        }
    }
}

#[test]
fn under_guard_corpus_off_a_file_tool_aimed_under_the_corpus_gets_no_output() {
    let f = Fixture::new("hooks-guard-corpus-off");
    let repo = f.corpus("repo", 0);
    f.set(&repo, "guard.corpus", "off");
    for tool in READERS.iter().chain(EDITORS) {
        for target in corpus_paths(&repo, under_corpus(tool)) {
            let out = f.ask(&repo, tool, &target, &[]);
            assert_eq!(out, "", "guard.corpus off: {tool} {target}");
        }
    }
}

#[test]
fn guard_edit_speaks_only_about_an_edit_no_live_claim_covers() {
    let f = Fixture::new("hooks-guard-edit");
    let repo = f.corpus("repo", 0);
    let covered = "src/lib.rs";
    let uncovered = "docs/page.md";
    let absolute = repo.join("docs").join("page.md");
    let absolute = absolute.to_string_lossy();

    f.set(&repo, "guard.edit", "off");
    for tool in EDITORS {
        for target in [covered, uncovered, &absolute] {
            assert_eq!(f.ask(&repo, tool, target, &[]), "", "off: {tool} {target}");
        }
    }

    for level in [None, Some("warn")] {
        match level {
            Some(l) => f.set(&repo, "guard.edit", l),
            None => {
                ok(f.ank(&repo, &["config", "--unset", "guard.edit"]), "unset");
            }
        }
        for tool in EDITORS {
            for target in [uncovered, &absolute] {
                let out = f.ask(&repo, tool, target, &[]);
                assert!(
                    !denies(&out),
                    "warn {level:?}: {tool} {target} denied: {out}"
                );
                assert!(
                    out.contains("docs/page.md"),
                    "warn {level:?}: {tool} {target} names no path: {out:?}"
                );
            }
            assert_eq!(f.ask(&repo, tool, covered, &[]), "", "warn: {tool} covered");
        }
    }

    f.set(&repo, "guard.edit", "claim");
    for tool in EDITORS {
        for target in [uncovered, &absolute] {
            let out = f.ask(&repo, tool, target, &[]);
            assert!(denies(&out), "claim: {tool} {target} not denied: {out:?}");
            assert!(out.contains("docs/page.md"), "claim: no path: {out}");
        }
        assert_eq!(
            f.ask(&repo, tool, covered, &[]),
            "",
            "claim: {tool} covered"
        );
    }

    // A tool that reads is not an edit, whatever the level.
    for tool in READERS {
        assert_eq!(
            f.ask(&repo, tool, "docs", &[]),
            "",
            "claim: {tool} is no edit"
        );
    }
}

#[test]
fn an_edit_with_no_live_claim_at_all_is_spoken_about_as_uncovered() {
    let f = Fixture::new("hooks-guard-edit-unclaimed");
    let repo = f.corpus("repo", 0);
    ok(f.ank(&repo, &["release", "--reason", "fixture"]), "release");
    f.set(&repo, "guard.edit", "claim");
    let out = f.ask(&repo, "Edit", "src/lib.rs", &[]);
    assert!(denies(&out), "no claim at all, yet not denied: {out:?}");
}

#[test]
fn the_launch_environment_tightens_the_guards_and_a_looser_value_is_ignored_aloud() {
    let f = Fixture::new("hooks-guard-env");
    let repo = f.corpus("repo", 0);

    f.set(&repo, "guard.edit", "warn");
    let out = f.ask(
        &repo,
        "Edit",
        "docs/page.md",
        &[("ANK_GUARD_EDIT", "claim")],
    );
    assert!(denies(&out), "ANK_GUARD_EDIT=claim over warn: {out:?}");

    f.set(&repo, "guard.edit", "claim");
    let out = f.ask(&repo, "Edit", "docs/page.md", &[("ANK_GUARD_EDIT", "off")]);
    assert!(denies(&out), "ANK_GUARD_EDIT=off over claim: {out:?}");
    assert!(
        out.contains("ANK_GUARD_EDIT=off") && out.contains("ignored"),
        "the looser value is not said to be ignored: {out}"
    );
    let out = f.ask(&repo, "Edit", "docs/page.md", &[("ANK_GUARD_EDIT", "warn")]);
    assert!(denies(&out), "ANK_GUARD_EDIT=warn over claim: {out:?}");
    assert!(
        out.contains("ANK_GUARD_EDIT=warn") && out.contains("ignored"),
        "{out}"
    );

    f.set(&repo, "guard.corpus", "off");
    let out = f.ask(
        &repo,
        "Read",
        ".ank/config.yml",
        &[("ANK_GUARD_CORPUS", "deny")],
    );
    assert!(denies(&out), "ANK_GUARD_CORPUS=deny over off: {out:?}");

    f.set(&repo, "guard.corpus", "deny");
    let out = f.ask(
        &repo,
        "Read",
        ".ank/config.yml",
        &[("ANK_GUARD_CORPUS", "off")],
    );
    assert!(denies(&out), "ANK_GUARD_CORPUS=off over deny: {out:?}");
    assert!(
        out.contains("ANK_GUARD_CORPUS=off") && out.contains("ignored"),
        "the looser value is not said to be ignored: {out}"
    );
}

#[test]
fn outside_any_corpus_the_hook_denies_nothing_and_exits_zero() {
    let f = Fixture::new("hooks-guard-outside");
    let bare = f.root.join("bare");
    fs::create_dir_all(bare.join(".ank")).unwrap();
    f.git(&bare, &["init", "-q", "."]);
    let plain = f.root.join("plain");
    fs::create_dir_all(&plain).unwrap();
    for dir in [&bare, &plain] {
        for tool in READERS.iter().chain(EDITORS) {
            let env = [("ANK_GUARD_CORPUS", "deny"), ("ANK_GUARD_EDIT", "claim")];
            let out = f.ask(dir, tool, ".ank/config.yml", &env);
            assert!(!denies(&out), "{tool} in {dir:?}: {out}");
        }
    }
}

#[test]
fn a_payload_the_hook_cannot_read_denies_nothing_and_exits_zero() {
    let f = Fixture::new("hooks-guard-malformed");
    let repo = f.corpus("repo", 0);
    f.set(&repo, "guard.edit", "claim");
    let payloads: [&[u8]; 8] = [
        b"",
        b"not json at all",
        b"{\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\".ank/config.yml\"}",
        b"[1,2,3]",
        b"{\"tool_name\":\"Read\",\"tool_input\":{\"file_path\":\"\xff\xfe\"}}",
        b"{\"tool_name\":\"Read\",\"tool_input\":\"not an object\"}",
        b"{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":42}}",
        b"{\"tool_name\":7,\"tool_input\":{\"file_path\":\"docs/page.md\"}}",
    ];
    for p in payloads {
        let out = f.hook(&repo, p, &[], None);
        let shown = String::from_utf8_lossy(p);
        assert_eq!(
            out.status.code(),
            Some(0),
            "payload {shown:?}: {}",
            stderr(&out)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(!denies(&stdout), "payload {shown:?}: {stdout}");
    }
}

#[test]
fn the_git_processes_of_one_invocation_do_not_grow_with_the_corpus() {
    let f = Fixture::new("hooks-guard-count");
    let count = |trace: &Path| -> usize {
        let text = fs::read_to_string(trace).unwrap_or_default();
        text.lines().filter(|l| l.contains(MARK)).count()
    };
    let asks = [
        ("Read", ".ank/config.yml"),
        ("Edit", "src/lib.rs"),
        ("Edit", "docs/page.md"),
        ("Grep", "docs"),
    ];
    let mut measured = Vec::new();
    for (name, extra) in [("small", 9), ("large", 499)] {
        let repo = f.corpus(name, extra);
        f.set(&repo, "guard.edit", "claim");
        // Warm: the index a first reader builds is not what the hook costs.
        ok(f.ank(&repo, &["status"]), "status");
        let mut counts = Vec::new();
        for (i, (tool, target)) in asks.iter().enumerate() {
            let trace = f.root.join(format!("{name}-{i}.trace"));
            let out = f.hook(
                &repo,
                payload(&repo, tool, target).as_bytes(),
                &[],
                Some(&trace),
            );
            assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
            counts.push(count(&trace));
        }
        eprintln!(
            "git processes per invocation, {} entities: {counts:?}",
            extra + 1
        );
        measured.push(counts);
    }
    // The edit guard reads the claims: a count of nothing would be a trace
    // that recorded nothing, and two equal zeros would prove nothing.
    assert!(
        measured[0][1] > 0,
        "the trace records no git process: {measured:?}"
    );
    assert_eq!(
        measured[0], measured[1],
        "the git processes per invocation grew from 10 entities to 500 ({:?} against {:?})",
        measured[0], measured[1]
    );
}
