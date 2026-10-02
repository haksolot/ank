//! amend adds and drops the ADRs an ADR amends (TASK-0e8f4ed12897, follow-up
//! of TASK-fe548f3dd587, ADR-9ee76b578257, issue #503).
//!
//! Every case through the binary: `--amends` and `--drop-amends` add and
//! remove entries on a proposed ADR and on an accepted one, since the
//! ratification anchor hashes the constraint and the scope and not what the ADR
//! amends; both are refused on a task and a spec; the finding `check` reports
//! on an amends names this verb as its repair; and `help amend` lists both.

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
        let p = scratch::path("amend-amends-gitconfig");
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
        fs::write(r.0.join(".gitignore"), ".ank/index.db\n").unwrap();
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

    fn show(&self, id: &str) -> String {
        self.ok(&["show", id])
    }

    fn check(&self) -> String {
        let out = self.ank(&["check"]);
        assert!(
            [0, 8].contains(&code(&out)),
            "check: {}{}",
            stdout(&out),
            stderr(&out)
        );
        stdout(&out)
    }

    /// The file of an entity, rewritten by hand: the state `new` refuses to
    /// write and `check` exists to report, reached the way a hand edit or a
    /// deletion reaches it.
    fn rewrite(&self, id: &str, from: &str, to: &str) {
        let path = self.0.join(format!(".ank/entities/{id}.md"));
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(from), "{from} not in {text}");
        fs::write(&path, text.replacen(from, to, 1)).unwrap();
    }
}

/// The `amends:` line `show` prints of an ADR, or `None` when it amends nothing.
fn amends_line(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.starts_with("amends:"))
        .map(str::to_string)
}

/// The lines `check` prints about one entity, in order.
fn about(out: &str, id: &str) -> Vec<String> {
    out.lines()
        .filter(|l| l.contains(id))
        .map(str::to_string)
        .collect()
}

#[test]
fn a_proposed_adr_gains_and_loses_what_it_amends() {
    let r = Repo::new("amend-amends-proposed");
    let a = r.adr("The foundation", &[]);
    let b = r.adr("A second foundation", &[]);
    let c = r.adr("The amendment", &[]);
    assert_eq!(amends_line(&r.show(&c)), None);

    let out = r.ok(&["amend", &c, "--amends", &a, "--amends", &b]);
    assert!(out.contains(&format!("+amends {a}")), "{out}");
    assert_eq!(
        amends_line(&r.show(&c)).as_deref(),
        Some(format!("amends: [{a}, {b}]").as_str())
    );

    let out = r.ok(&["amend", &c, "--drop-amends", &a]);
    assert!(out.contains(&format!("-amends {a}")), "{out}");
    assert_eq!(
        amends_line(&r.show(&c)).as_deref(),
        Some(format!("amends: [{b}]").as_str())
    );

    // Emptied out, the field is omitted, as `new` omits it.
    r.ok(&["amend", &c, "--drop-amends", &b]);
    assert_eq!(amends_line(&r.show(&c)), None);

    // Dropping what is not there is refused, not ignored.
    let out = r.ank(&["amend", &c, "--drop-amends", &a]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("does not amend"), "{}", stderr(&out));

    // An ADR does not amend itself.
    let out = r.ank(&["amend", &c, "--amends", &c]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));

    // Only an ADR is amended, in the words `new` and `check` use.
    let spec = r
        .ok(&["new", "spec", "--title", "s", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    let out = r.ank(&["amend", &c, "--amends", &spec]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stderr(&out).contains("is not an ADR"), "{}", stderr(&out));
}

#[test]
fn a_task_and_a_spec_refuse_both_flags() {
    let r = Repo::new("amend-amends-kinds");
    let a = r.adr("The foundation", &[]);
    let task = r
        .ok(&["new", "task", "--title", "t", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    let spec = r
        .ok(&["new", "spec", "--title", "s", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    for target in [&task, &spec] {
        for flag in ["--amends", "--drop-amends"] {
            let out = r.ank(&["amend", target, flag, &a]);
            assert_eq!(code(&out), 1, "{target} {flag}: {}", stderr(&out));
            assert!(
                stderr(&out).contains("amends applies to an ADR"),
                "{target} {flag}: {}",
                stderr(&out)
            );
        }
    }
}

#[test]
fn an_accepted_adr_is_amended_and_its_anchor_still_holds() {
    let r = Repo::new("amend-amends-accepted");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let b = r.adr("The amendment, ratified before it said so", &[]);
    r.accept(&b);
    let before = about(&r.check(), &b);

    // The ratification anchor hashes the constraint and the scope, and
    // `amends` is neither: the edit is let through, and `check` says nothing
    // new about the ADR afterwards, no altered anchor among it.
    r.ok(&["amend", &b, "--amends", &a]);
    assert_eq!(
        amends_line(&r.show(&b)).as_deref(),
        Some(format!("amends: [{a}]").as_str())
    );
    assert!(r.show(&b).contains("\nstatus: accepted\n"));
    r.commit("amend");
    assert_eq!(about(&r.check(), &b), before);
    let amended = r.show(&a);
    assert!(amended.contains("AMENDED BY ("), "{amended}");

    r.ok(&["amend", &b, "--drop-amends", &a]);
    assert_eq!(amends_line(&r.show(&b)), None);
    r.commit("drop");
    assert_eq!(about(&r.check(), &b), before);

    // The scope stays anchored, and stays refused.
    let out = r.ank(&["amend", &b, "--scope", "docs/**"]);
    assert_eq!(code(&out), 6, "{}", stderr(&out));
}

#[test]
fn a_finding_on_amends_names_amend_as_its_repair_and_the_repair_clears_it() {
    let r = Repo::new("amend-amends-repair");
    let lost = r.adr("Lost later", &[]);
    let draft = r.adr("An amendment", &["--amends", &lost]);
    let absent = "ADR-0000000000ff";
    // Reached the way the finding is reached: a target the corpus lost.
    r.rewrite(&draft, &lost, absent);
    let out = r.check();
    let repair = format!("ank amend {draft} --drop-amends {absent}");
    assert!(
        out.lines()
            .any(|l| l.contains("does not exist") && l.contains(&repair)),
        "{out}"
    );

    // And the command it names is one the verb takes: the id is matched
    // against what the ADR stores and never looked up.
    // The machinery entry recording the drop names the id in prose, which is
    // a signal of its own; what is gone is the finding against the amends.
    r.ok(&["amend", &draft, "--drop-amends", absent]);
    let out = r.check();
    assert!(
        !out.lines()
            .any(|l| l.contains(&draft) && l.contains(" amends ")),
        "{out}"
    );
}

#[test]
fn help_amend_lists_both_flags() {
    let out = spawn(ANK)
        .args(["help", "amend"])
        .current_dir(scratch::root())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let page = stdout(&out);
    let flags = page
        .lines()
        .find(|l| l.trim_start().starts_with("flags:"))
        .unwrap_or_else(|| panic!("{page}"));
    for flag in ["--amends", "--drop-amends"] {
        assert!(
            flags.split_whitespace().any(|w| w == flag),
            "{flag}:\n{page}"
        );
    }
}
