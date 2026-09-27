//! `ank help new` says, per kind, what is required and what is refused, and
//! `ank help new <kind>` answers for that kind alone (TASK-af5af0fe9a7f, issue
//! #504).
//!
//! **The help is read, then confronted with the binary.** Every requirement and
//! every refusal is taken from `ank help new --json` and replayed against
//! `ank new` in a fresh repository, so a line added to the help that the verb
//! does not honour, or a refusal the verb changes without the help following,
//! is a red here. A hand-written list of what `new` requires would be a second
//! copy of the table, and the second copy is the one that drifts
//! (ADR-2b62b9a1fe67). The few literals below are the floor that keeps the
//! replay from being vacuous: a help stating nothing would replay green.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_yaml::Value;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@help-new";
const KINDS: [&str; 3] = ["task", "adr", "spec"];

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("help-new-gitconfig");
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
        // With every mandatory flag absent `new` opens an editor; with none
        // set that is a refusal at 9 rather than a hang, whichever case a
        // replay below happens to build.
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) {
    let out = spawn("git").args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", stderr(&out));
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

/// `ank` outside any repository: `help` needs none, and asking it from nowhere
/// is what proves it.
fn help(args: &[&str]) -> Output {
    spawn(ANK)
        .arg("help")
        .args(args)
        .current_dir(scratch::root())
        .output()
        .unwrap()
}

fn help_ok(args: &[&str]) -> String {
    let out = help(args);
    assert_eq!(code(&out), 0, "help {args:?}: {}", stderr(&out));
    stdout(&out)
}

fn yaml(text: &str) -> Value {
    serde_yaml::from_str(text).unwrap_or_else(|e| panic!("{e}: {text}"))
}

fn strs(v: &Value) -> Vec<String> {
    v.as_sequence()
        .unwrap_or_else(|| panic!("not a list: {v:?}"))
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// The one verb `help new [<kind>] --json` describes.
fn new_verb(args: &[&str]) -> Value {
    let mut all = vec!["new"];
    all.extend_from_slice(args);
    all.push("--json");
    let doc = yaml(&help_ok(&all));
    let verbs = doc["verbs"].as_sequence().unwrap();
    assert_eq!(verbs.len(), 1, "{doc:?}");
    verbs[0].clone()
}

fn kind_of<'a>(verb: &'a Value, name: &str) -> &'a Value {
    verb["kinds"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|k| k["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("no kind {name} in {verb:?}"))
}

/// `(flag, code, when)` for each row of a kind's `requires` or `refuses`.
fn rules(kind: &Value, field: &str) -> Vec<(String, i32, String)> {
    kind[field]
        .as_sequence()
        .unwrap_or_else(|| panic!("no {field} on {kind:?}"))
        .iter()
        .map(|r| {
            (
                r["flag"].as_str().unwrap().to_string(),
                r["code"].as_i64().unwrap() as i32,
                r["when"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

/// A repository with one entity of each kind and one verifier, so every flag
/// `new` takes has a value the verb accepts: a `--blocked-by` needs a task to
/// name, a `--supersedes` one of its own kind, a `--verify` a declared name.
struct Repo {
    dir: PathBuf,
    task: String,
    adr: String,
    spec: String,
}

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        fs::write(
            dir.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             verifiers:\n  ok:\n    run: \"true\"\n    timeout: 1m\n",
        )
        .unwrap();
        fs::write(dir.join(".gitignore"), ".ank/index.db\n").unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-qm", "seed"]);
        let mut r = Repo {
            dir,
            task: String::new(),
            adr: String::new(),
            spec: String::new(),
        };
        r.task = r.created(&["task", "--title", "t", "--scope", "src/**"]);
        r.adr = r.created(&[
            "adr",
            "--title",
            "a",
            "--scope",
            "src/**",
            "--constraint",
            "c",
        ]);
        r.spec = r.created(&["spec", "--title", "s", "--scope", "src/**"]);
        r
    }

    fn new_entity(&self, args: &[String]) -> Output {
        spawn(ANK)
            .arg("new")
            .args(args)
            .arg("--repo")
            .arg(&self.dir)
            .current_dir(scratch::root())
            .output()
            .unwrap()
    }

    fn created(&self, args: &[&str]) -> String {
        let mut all: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        all.push("--json".into());
        let out = self.new_entity(&all);
        assert_eq!(code(&out), 0, "new {args:?}: {}", stderr(&out));
        yaml(&stdout(&out))["id"].as_str().unwrap().to_string()
    }

    /// A value `new <kind>` accepts for `flag`, or `None` for a switch.
    fn value(&self, kind: &str, flag: &str) -> Option<String> {
        Some(match flag {
            "--no-verify" => return None,
            "--scope" => "src/**".into(),
            "--blocked-by" => self.task.clone(),
            "--reference" => self.spec.clone(),
            "--supersedes" => match kind {
                "adr" => self.adr.clone(),
                "spec" => self.spec.clone(),
                _ => self.task.clone(),
            },
            "--verify" => "ok".into(),
            "--method" => "tdd".into(),
            _ => "x".into(),
        })
    }

    /// `ank new <kind>` with every flag the kind requires but `without`, plus
    /// `extra`.
    fn replay(&self, kind: &Value, without: Option<&str>, extra: Option<&str>) -> Output {
        let name = kind["name"].as_str().unwrap();
        let mut args = vec![name.to_string()];
        let mut push = |flag: &str| {
            args.push(flag.to_string());
            if let Some(v) = self.value(name, flag) {
                args.push(v);
            }
        };
        for (flag, _, _) in rules(kind, "requires") {
            if Some(flag.as_str()) != without {
                push(&flag);
            }
        }
        if let Some(flag) = extra {
            push(flag);
        }
        self.new_entity(&args)
    }
}

/// The floor under the replay: the three kinds are described, each requires
/// its title and scope, and an adr requires its constraint at 7 -- the rule
/// issue #504 learnt from the refusal because the help did not say it.
#[test]
fn help_new_describes_each_kind_and_names_the_constraint_an_adr_requires() {
    let verb = new_verb(&[]);
    let names: Vec<String> = verb["kinds"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|k| k["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(names, KINDS, "{verb:?}");
    for name in KINDS {
        let required: Vec<String> = rules(kind_of(&verb, name), "requires")
            .into_iter()
            .map(|(f, _, _)| f)
            .collect();
        assert!(
            required.contains(&"--title".into()) && required.contains(&"--scope".into()),
            "{name}: {required:?}"
        );
        assert!(
            !rules(kind_of(&verb, name), "refuses").is_empty(),
            "{name} states no refusal"
        );
    }
    let adr = rules(kind_of(&verb, "adr"), "requires");
    assert!(
        adr.iter().any(|(f, c, _)| f == "--constraint" && *c == 7),
        "{adr:?}"
    );

    // And the verb's own refusals carry it, in both forms: the list a caller
    // reads before calling is where issue #504 looked and found nothing.
    let refusals: Vec<(i64, String)> = verb["refuses"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["code"].as_i64().unwrap(),
                r["when"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert!(
        refusals
            .iter()
            .any(|(c, w)| *c == 7 && w.contains("adr") && w.contains("--constraint")),
        "{refusals:?}"
    );
    let text = help_ok(&["new"]);
    assert!(
        text.lines()
            .any(|l| l.contains("adr") && l.contains("--constraint") && l.ends_with("(7)")),
        "{text}"
    );
}

/// Every requirement the help states is a refusal `new` makes: the code it
/// names, and the reason in the message.
#[test]
fn every_requirement_the_help_states_is_refused_by_new_as_stated() {
    let r = Repo::new("help-new-requires");
    let verb = new_verb(&[]);
    let mut replayed = 0;
    for name in KINDS {
        let kind = kind_of(&verb, name);
        for (flag, want, when) in rules(kind, "requires") {
            let out = r.replay(kind, Some(&flag), None);
            let err = stderr(&out);
            assert_eq!(code(&out), want, "new {name} without {flag}: {err}");
            assert!(err.contains(&when), "new {name} without {flag}: {err}");
            replayed += 1;
        }
        // Everything required and nothing else is a creation: the requirements
        // are sufficient, not only each necessary.
        let out = r.replay(kind, None, None);
        assert_eq!(
            code(&out),
            0,
            "new {name} with its requirements: {}",
            stderr(&out)
        );
    }
    assert!(replayed >= 7, "{replayed}");
}

/// Every refusal the help states is one `new` makes, with the code and the
/// reason; every flag the help says a kind takes, `new` takes.
#[test]
fn every_refusal_and_every_flag_the_help_states_holds_through_new() {
    let r = Repo::new("help-new-refuses");
    let verb = new_verb(&[]);
    for name in KINDS {
        let kind = kind_of(&verb, name);
        for (flag, want, when) in rules(kind, "refuses") {
            let out = r.replay(kind, None, Some(&flag));
            let err = stderr(&out);
            assert_eq!(code(&out), want, "new {name} {flag}: {err}");
            assert!(
                err.contains(&format!("{flag} applies to")),
                "new {name} {flag}: {err}"
            );
            assert!(err.contains(&when), "new {name} {flag}: {err}");
        }
        let required: Vec<String> = rules(kind, "requires")
            .into_iter()
            .map(|(f, _, _)| f)
            .collect();
        for flag in strs(&kind["flags"]) {
            if required.contains(&flag) {
                continue;
            }
            let out = r.replay(kind, None, Some(&flag));
            assert_eq!(code(&out), 0, "new {name} {flag}: {}", stderr(&out));
        }
    }
}

/// Every flag of `new` is, for each kind, either taken or refused, and never
/// both (TASK-1667febee0dd). A flag that is neither is one the help is silent
/// about, which is where `--constraint` on a task, `--criteria` and
/// `--blocked-by` on an ADR were hiding while `new` dropped them at exit 0; the
/// replay above only confronts what the help states, so the silence itself has
/// to be the red.
#[test]
fn every_flag_of_new_is_taken_or_refused_by_each_kind() {
    let verb = new_verb(&[]);
    let every_flag: Vec<String> = verb["flags"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap().to_string())
        .filter(|f| !["--json", "--quiet", "--repo", "--worktree"].contains(&f.as_str()))
        .collect();
    assert!(every_flag.len() >= 11, "{every_flag:?}");
    for name in KINDS {
        let kind = kind_of(&verb, name);
        let taken = strs(&kind["flags"]);
        let refused: Vec<String> = rules(kind, "refuses")
            .into_iter()
            .map(|(f, _, _)| f)
            .collect();
        for flag in &every_flag {
            let (t, r) = (taken.contains(flag), refused.contains(flag));
            assert!(
                t != r,
                "new {name} {flag}: taken {t}, refused {r}; it must be exactly one"
            );
        }
    }
}

/// `help new <kind>` exits 0 and speaks of that kind only: its flags, its
/// requirements, its refusals, and nothing another kind owns.
#[test]
fn help_new_kind_answers_for_that_kind_alone() {
    let whole = new_verb(&[]);
    let every_flag: Vec<String> = whole["flags"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|f| f["name"].as_str().unwrap().to_string())
        .collect();
    for name in KINDS {
        let kind = kind_of(&whole, name);
        let mine: Vec<String> = strs(&kind["flags"])
            .into_iter()
            .chain(rules(kind, "refuses").into_iter().map(|(f, _, _)| f))
            .collect();

        let one = new_verb(&[name]);
        assert_eq!(
            one["usage"].as_str(),
            Some(format!("ank new {name}").as_str())
        );
        let kinds = one["kinds"].as_sequence().unwrap();
        assert_eq!(kinds.len(), 1, "{one:?}");
        assert_eq!(&kinds[0], kind, "help new {name} disagrees with help new");
        let offered: Vec<String> = one["flags"]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|f| f["name"].as_str().unwrap().to_string())
            .filter(|f| {
                every_flag.contains(f)
                    && !["--json", "--quiet", "--repo", "--worktree"].contains(&f.as_str())
            })
            .collect();
        assert_eq!(offered, strs(&kind["flags"]), "help new {name} --json");

        let text = help_ok(&["new", name]);
        assert!(text.starts_with(&format!("ank new {name}\n")), "{text}");
        // A flag of `new` that this kind neither takes nor refuses is none of
        // this page's business, and neither is another kind's usage line.
        for flag in &every_flag {
            if mine.contains(flag)
                || ["--json", "--quiet", "--repo", "--worktree"].contains(&flag.as_str())
            {
                continue;
            }
            assert!(
                !text
                    .split(|c: char| c.is_whitespace() || c == ',' || c == ':')
                    .any(|w| w == flag),
                "help new {name} mentions {flag}:\n{text}"
            );
        }
        for other in KINDS.iter().filter(|k| **k != name) {
            assert!(!text.contains(&format!("new {other}")), "{text}");
        }
        assert_eq!(help_ok(&["new", name, "--quiet"]), "");
    }
}

/// The text form says what the JSON form says, for the whole verb and for
/// each kind: every flag, requirement and refusal, with its code.
#[test]
fn text_and_json_forms_of_help_new_agree() {
    let check = |args: &[&str]| {
        let verb = new_verb(args);
        let mut all = vec!["new"];
        all.extend_from_slice(args);
        let text = help_ok(&all);
        for kind in verb["kinds"].as_sequence().unwrap() {
            let name = kind["name"].as_str().unwrap();
            for flag in strs(&kind["flags"]) {
                assert!(text.contains(&flag), "{args:?} {name} {flag}:\n{text}");
            }
            for field in ["requires", "refuses"] {
                for (flag, c, when) in rules(kind, field) {
                    assert!(
                        text.lines().any(|l| l.contains(&flag)
                            && l.contains(&when)
                            && l.contains(&format!("({c})"))),
                        "{args:?} {name} {field} {flag} {when} ({c}):\n{text}"
                    );
                }
            }
        }
        for r in verb["refuses"].as_sequence().unwrap() {
            let line = format!(
                "{} ({})",
                r["when"].as_str().unwrap(),
                r["code"].as_i64().unwrap()
            );
            assert!(text.contains(&line), "{args:?} {line}:\n{text}");
        }
        for note in strs(&verb["notes"]) {
            assert!(text.contains(&note), "{args:?} {note}:\n{text}");
        }
    };
    check(&[]);
    for name in KINDS {
        check(&[name]);
    }
}

/// A kind `new` does not have, and a second word after a verb that has no
/// kinds, are still refused, and each refusal names the form that works.
#[test]
fn help_refuses_a_kind_that_does_not_exist_naming_the_form_that_works() {
    for args in [
        &["new", "log"][..],
        &["new", "nope"],
        &["new", "nope", "--json"],
    ] {
        let out = help(args);
        assert_ne!(code(&out), 0, "help {args:?}: {}", stdout(&out));
        assert!(
            stdout(&out).is_empty(),
            "help {args:?} answered: {}",
            stdout(&out)
        );
        assert!(
            stderr(&out).contains("ank help new <task|adr|spec>"),
            "help {args:?}: {}",
            stderr(&out)
        );
    }
    for args in [&["claim", "adr"][..], &["claim", "x", "--json"]] {
        let out = help(args);
        assert_ne!(code(&out), 0, "help {args:?}: {}", stdout(&out));
        assert!(
            stdout(&out).is_empty(),
            "help {args:?} answered: {}",
            stdout(&out)
        );
        assert!(
            stderr(&out).contains("-> ank help claim\n"),
            "help {args:?}: {}",
            stderr(&out)
        );
    }
    // And a verb with no kinds says it has none, in both forms.
    let claim = yaml(&help_ok(&["claim", "--json"]));
    assert_eq!(
        claim["verbs"][0]["kinds"].as_sequence().map(|k| k.len()),
        Some(0)
    );
}
