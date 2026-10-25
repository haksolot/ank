//! `ank hooks install claude` and `ank hooks remove claude`, through the binary
//! (ADR-24efa7543a16, ADR-43cbb20dcbf1, TASK-f248037e8bab).
//!
//! The file is Claude Code's own `settings.json`, which its owner edits by hand
//! and other tools write into, so what is measured is bytes: every foreign byte
//! survives an install, a second install changes none, and a remove gives back
//! the file exactly as the first install found it. `CLAUDE_CONFIG_DIR` points
//! every run at a fixture, and `HOME` at an empty directory, so no run of this
//! suite can reach the settings of the machine it runs on.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// A settings file as its owner leaves it: keys ank has no business with, and
/// a hook of somebody else's on both events ank installs on.
const FOREIGN: &str = r#"{
  "model": "opus",
  "permissions": {
    "allow": ["Bash(git status)"]
  },
  "hooks": {
    "SessionStart": [
      {
        "hooks": [
          { "type": "command", "command": "herdr integration hook claude" }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "moshi-hook pre" }
        ]
      }
    ],
    "Stop": [
      { "hooks": [ { "type": "command", "command": "moshi-hook stop" } ] }
    ]
  },
  "statusLine": { "type": "command", "command": "echo hi" }
}
"#;

struct Fixture {
    base: PathBuf,
}

impl Fixture {
    fn new() -> Fixture {
        let base = scratch::dir("hooks-install");
        fs::create_dir_all(base.join("config")).unwrap();
        fs::create_dir_all(base.join("home")).unwrap();
        fs::create_dir_all(base.join("cwd")).unwrap();
        Fixture { base }
    }

    fn config(&self) -> PathBuf {
        self.base.join("config")
    }

    fn settings(&self) -> PathBuf {
        self.config().join("settings.json")
    }

    fn ank(&self, args: &[&str]) -> Output {
        Command::new(ANK)
            .args(args)
            .current_dir(self.base.join("cwd"))
            .env("CLAUDE_CONFIG_DIR", self.config())
            .env("HOME", self.base.join("home"))
            .env("USERPROFILE", self.base.join("home"))
            .env("NO_COLOR", "1")
            .env("ANK_AGENT", "tool/1.0")
            .output()
            .expect("the binary must run")
    }

    fn read(&self) -> String {
        fs::read_to_string(self.settings()).expect("settings.json must exist")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

fn ok(out: &Output, what: &str) {
    assert!(
        out.status.success(),
        "{what}: exit {:?}\nstdout: {}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn parse(text: &str) -> serde_yaml::Value {
    serde_yaml::from_str(text).unwrap_or_else(|e| panic!("not readable JSON: {e}\n{text}"))
}

/// The commands every entry of `event` runs, in the file's order.
fn commands(doc: &serde_yaml::Value, event: &str) -> Vec<String> {
    doc["hooks"][event]
        .as_sequence()
        .unwrap_or_else(|| panic!("hooks.{event} is not an array"))
        .iter()
        .flat_map(|entry| {
            entry["hooks"]
                .as_sequence()
                .expect("an entry carries a hooks array")
                .iter()
                .map(|h| h["command"].as_str().unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// What the entry for `event` runs: the installing binary, by absolute path.
fn ours(event: &str) -> String {
    let exe = fs::canonicalize(ANK).unwrap();
    format!("\"{}\" hooks run claude {event}", exe.display())
}

/// Whether `inner` is a subsequence of `outer`: every byte kept, in order.
fn kept(inner: &str, outer: &str) -> bool {
    let mut rest = outer.bytes();
    inner.bytes().all(|b| rest.any(|o| o == b))
}

fn files_under(dir: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(files_under(&path));
        } else {
            found.push(path.display().to_string());
        }
    }
    found.sort();
    found
}

#[test]
fn install_adds_its_two_entries_and_every_foreign_byte_survives() {
    let fx = Fixture::new();
    fs::write(fx.settings(), FOREIGN).unwrap();

    let out = fx.ank(&["hooks", "install", "claude"]);
    ok(&out, "install");
    let after = fx.read();
    assert!(
        kept(FOREIGN, &after),
        "a foreign byte was lost or moved:\n{after}"
    );
    let doc = parse(&after);
    assert_eq!(
        commands(&doc, "SessionStart"),
        [
            "herdr integration hook claude".to_string(),
            ours("session-start")
        ]
    );
    assert_eq!(
        commands(&doc, "PreToolUse"),
        ["moshi-hook pre".to_string(), ours("pre-tool-use")]
    );
    assert_eq!(commands(&doc, "Stop"), ["moshi-hook stop"]);
    assert_eq!(doc["model"].as_str(), Some("opus"));
    assert_eq!(doc["statusLine"]["command"].as_str(), Some("echo hi"));
}

#[test]
fn a_second_install_leaves_the_file_byte_identical() {
    let fx = Fixture::new();
    fs::write(fx.settings(), FOREIGN).unwrap();
    ok(&fx.ank(&["hooks", "install", "claude"]), "install");
    let once = fx.read();
    ok(&fx.ank(&["hooks", "install", "claude"]), "install, again");
    assert_eq!(fx.read(), once, "a second install moved a byte");
}

#[test]
fn remove_gives_back_the_file_as_the_first_install_found_it() {
    let fx = Fixture::new();
    fs::write(fx.settings(), FOREIGN).unwrap();
    ok(&fx.ank(&["hooks", "install", "claude"]), "install");
    ok(&fx.ank(&["hooks", "install", "claude"]), "install, again");
    ok(&fx.ank(&["hooks", "remove", "claude"]), "remove");
    assert_eq!(fx.read(), FOREIGN);
    // Nothing left to take out is not an error, and changes nothing either.
    ok(&fx.ank(&["hooks", "remove", "claude"]), "remove, again");
    assert_eq!(fx.read(), FOREIGN);
}

/// The shapes a hand-kept file takes that the pretty one above does not: an
/// event array that exists and is empty, a `hooks` object with neither event,
/// no `hooks` at all, an empty object, and all of it on one line.
#[test]
fn remove_restores_every_shape_install_had_to_create_into() {
    for before in [
        "{\n  \"hooks\": {\n    \"SessionStart\": [],\n    \"PreToolUse\": [ ]\n  }\n}\n",
        "{\n  \"hooks\": {\n    \"Stop\": []\n  }\n}\n",
        "{\n  \"hooks\": {}\n}\n",
        "{\n  \"model\": \"opus\"\n}\n",
        "{}\n",
        "{}",
        "{\"model\":\"opus\",\"hooks\":{\"Stop\":[{\"hooks\":[{\"type\":\"command\",\"command\":\"x\"}]}]}}",
        "{\r\n\t\"model\": \"opus\"\r\n}\r\n",
    ] {
        let fx = Fixture::new();
        fs::write(fx.settings(), before).unwrap();
        ok(&fx.ank(&["hooks", "install", "claude"]), "install");
        let after = fx.read();
        assert!(kept(before, &after), "{before:?} lost a byte:\n{after}");
        let doc = parse(&after);
        assert_eq!(commands(&doc, "SessionStart"), [ours("session-start")], "{before:?}");
        assert_eq!(commands(&doc, "PreToolUse"), [ours("pre-tool-use")], "{before:?}");
        ok(&fx.ank(&["hooks", "install", "claude"]), "install, again");
        assert_eq!(fx.read(), after, "{before:?}: a second install moved a byte");
        ok(&fx.ank(&["hooks", "remove", "claude"]), "remove");
        assert_eq!(fx.read(), before, "remove did not restore {before:?}");
    }
}

#[test]
fn with_no_settings_install_creates_one_holding_only_its_entries() {
    let fx = Fixture::new();
    ok(&fx.ank(&["hooks", "install", "claude"]), "install");
    let doc = parse(&fx.read());
    let root = doc.as_mapping().expect("an object");
    assert_eq!(root.len(), 1, "only hooks: {doc:?}");
    let hooks = doc["hooks"].as_mapping().expect("hooks is an object");
    assert_eq!(hooks.len(), 2, "only its two events: {doc:?}");
    assert_eq!(commands(&doc, "SessionStart"), [ours("session-start")]);
    assert_eq!(commands(&doc, "PreToolUse"), [ours("pre-tool-use")]);
}

/// A directory that does not exist yet is created, since that is where Claude
/// Code would look; it is the settings file's directory and nothing beside it.
#[test]
fn no_file_other_than_settings_json_is_written() {
    let fx = Fixture::new();
    fs::remove_dir_all(fx.config()).unwrap();
    ok(&fx.ank(&["hooks", "install", "claude"]), "install");
    ok(&fx.ank(&["hooks", "install", "claude"]), "install, again");
    assert_eq!(
        files_under(&fx.base),
        [fx.settings().display().to_string()],
        "install wrote beside the settings file"
    );
    ok(&fx.ank(&["hooks", "remove", "claude"]), "remove");
    assert_eq!(files_under(&fx.base), [fx.settings().display().to_string()]);
}

/// Without `CLAUDE_CONFIG_DIR`, the file is `.claude/settings.json` in the home
/// directory, which is where Claude Code reads it.
#[test]
fn without_claude_config_dir_the_file_is_under_home() {
    let fx = Fixture::new();
    let out = Command::new(ANK)
        .args(["hooks", "install", "claude"])
        .current_dir(fx.base.join("cwd"))
        .env_remove("CLAUDE_CONFIG_DIR")
        .env("HOME", fx.base.join("home"))
        .env("USERPROFILE", fx.base.join("home"))
        .output()
        .unwrap();
    ok(&out, "install");
    let doc = parse(&fs::read_to_string(fx.base.join("home/.claude/settings.json")).unwrap());
    assert_eq!(commands(&doc, "SessionStart"), [ours("session-start")]);
}

#[test]
fn another_agent_is_refused_naming_claude_as_the_only_one() {
    let fx = Fixture::new();
    fs::write(fx.settings(), FOREIGN).unwrap();
    for verb in ["install", "remove"] {
        let out = fx.ank(&["hooks", verb, "codex"]);
        assert_ne!(out.status.code(), Some(0), "{verb} codex must be refused");
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(
            said.contains("codex") && said.contains("claude"),
            "the refusal names the agent and the only one there is: {said}"
        );
        assert_eq!(fx.read(), FOREIGN, "a refusal writes nothing");
    }
    let out = fx.ank(&["hooks", "install"]);
    assert_ne!(out.status.code(), Some(0), "an install names its agent");
    assert!(String::from_utf8_lossy(&out.stderr).contains("claude"));
}

#[test]
fn a_file_that_is_not_an_object_is_refused_and_left_alone() {
    for before in [
        "",
        "[]\n",
        "{\"hooks\": [] }\n",
        "{ \"model\": \n",
        "{\"hooks\":{\"SessionStart\":{}}}",
    ] {
        let fx = Fixture::new();
        fs::write(fx.settings(), before).unwrap();
        let out = fx.ank(&["hooks", "install", "claude"]);
        assert_ne!(out.status.code(), Some(0), "{before:?} must be refused");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("settings.json"),
            "the refusal names the file: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(fx.read(), before, "a refusal writes nothing");
    }
}

#[test]
fn help_lists_hooks_and_declares_it_writes_nothing_in_the_corpus() {
    let fx = Fixture::new();
    let out = fx.ank(&["help"]);
    ok(&out, "help");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("ank hooks <install|remove|run>"),
        "help lists hooks:\n{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let out = fx.ank(&["help", "--json"]);
    ok(&out, "help --json");
    let doc = parse(&String::from_utf8_lossy(&out.stdout));
    let hooks = doc["verbs"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|v| v["name"].as_str() == Some("hooks"))
        .expect("help --json carries hooks");
    assert_eq!(hooks["writes"].as_str(), Some("nothing"));
}

/// The documents `install` and `remove` answer under `--json`, pinned, with the
/// fixture's path named away since it changes every run.
#[test]
fn json_golden_hooks() {
    let fx = Fixture::new();
    fs::write(fx.settings(), FOREIGN).unwrap();
    let settings = fx.settings().display().to_string();
    let escaped = settings.replace('\\', "\\\\");
    for (name, verb) in [
        ("hooks-install", "install"),
        ("hooks-install-again", "install"),
        ("hooks-remove", "remove"),
    ] {
        let out = fx.ank(&["hooks", verb, "claude", "--json"]);
        ok(&out, name);
        let text = String::from_utf8_lossy(&out.stdout).replace(&escaped, "<SETTINGS>");
        crate::fixture::pin(name, &text);
    }
}

/// `run` is what the entries call, and until its events are served it says
/// nothing and lets the call through: a hook that fails blocks the harness.
#[test]
fn run_is_silent_and_exits_zero() {
    let fx = Fixture::new();
    for event in ["session-start", "pre-tool-use", "no-such-event"] {
        let out = fx.ank(&["hooks", "run", "claude", event]);
        assert_eq!(out.status.code(), Some(0), "{event}");
        assert!(out.stdout.is_empty(), "{event}: run printed something");
    }
}
