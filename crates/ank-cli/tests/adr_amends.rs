//! An ADR declares the ADRs it amends in part, and the amended one names what
//! amends it (TASK-fe548f3dd587, ADR-9ee76b578257, issue #503).
//!
//! Every case through the binary: `new` writes the relation and refuses it on
//! the two kinds that do not carry it, `show` prints it on both sides, `check`
//! resolves it the way it resolves a spec's `references`, `archive` keeps an
//! amended ADR hot, and `help new` names the flag under `adr` only.

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

/// The shortest form a listing prints an id in: the kind and four digits,
/// which is what a corpus this small disambiguates on.
fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines of a `show` section, from its heading to the next blank line.
fn section<'a>(text: &'a str, heading: &str) -> Vec<&'a str> {
    let mut lines = text.lines();
    let found = lines
        .by_ref()
        .any(|l| l.starts_with(&format!("{heading} (")));
    assert!(found, "no {heading} section in:\n{text}");
    lines.take_while(|l| !l.is_empty()).collect()
}

#[test]
fn new_adr_writes_every_amends_it_is_given_and_the_other_kinds_refuse_it() {
    let r = Repo::new("amends-new");
    let a = r.adr("The foundation", &[]);
    let b = r.adr("A second foundation", &[]);
    let c = r.adr("The amendment", &["--amends", &a, "--amends", &b]);

    let text = r.show(&c);
    assert!(
        text.contains(&format!("\namends: [{a}, {b}]\n")),
        "both, in the order given: {text}"
    );

    // Anything but an ADR is refused at the write: an amendment changes a
    // decision, and a task or a spec is not one.
    let spec = r
        .ok(&["new", "spec", "--title", "s", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    let out = r.ank(&[
        "new",
        "adr",
        "--title",
        "t",
        "--scope",
        "src/**",
        "--constraint",
        "c",
        "--amends",
        &spec,
    ]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stderr(&out).contains(&spec), "{}", stderr(&out));

    for kind in [
        &["task", "--title", "t", "--scope", "src/**"][..],
        &["spec", "--title", "s", "--scope", "src/**"],
    ] {
        let mut args = vec!["new"];
        args.extend_from_slice(kind);
        args.extend_from_slice(&["--amends", &a]);
        let out = r.ank(&args);
        assert_eq!(code(&out), 1, "new {}: {}", kind[0], stderr(&out));
        assert!(
            stderr(&out).contains("--amends applies to an ADR"),
            "new {}: {}",
            kind[0],
            stderr(&out)
        );
    }
}

#[test]
fn show_prints_the_relation_on_both_sides_and_only_accepted_amendments() {
    let r = Repo::new("amends-show");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let b = r.adr("The accepted amendment", &["--amends", &a]);
    r.accept(&b);
    let pending = r.adr("A proposed amendment", &["--amends", &a]);

    let amending = r.show(&b);
    let lines = section(&amending, "AMENDS");
    assert_eq!(lines.len(), 1, "{amending}");
    assert!(
        lines[0].contains(short(&a)) && lines[0].contains("The foundation"),
        "{amending}"
    );

    // The amended ADR stays accepted, and names what amends it; the proposal
    // amends nothing yet, so it is not among them.
    let amended = r.show(&a);
    assert!(amended.contains("\nstatus: accepted\n"), "{amended}");
    let lines = section(&amended, "AMENDED BY");
    assert_eq!(lines.len(), 1, "{amended}");
    assert!(
        lines[0].contains(short(&b)) && lines[0].contains("The accepted amendment"),
        "{amended}"
    );
    assert!(!amended.contains(short(&pending)), "{amended}");
}

#[test]
fn check_resolves_amends_through_succession_and_reports_what_does_not_resolve() {
    let r = Repo::new("amends-check");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let amendment = r.adr("The amendment", &["--amends", &a]);
    r.accept(&amendment);
    // Every ADR here is ratified by its own author, which is a signal of its
    // own: what is asserted is that nothing is said about the relation.
    let about_amends = |out: &str| {
        out.lines()
            .any(|l| l.contains(&amendment) && l.contains(" amends "))
    };
    let clean = r.check();
    assert!(!about_amends(&clean), "{clean}");

    // Succession followed to its end: the target was replaced by an accepted
    // ADR, so the amendment still resolves and nothing is reported.
    let successor = r.adr("The replacement", &["--supersedes", &a]);
    r.accept(&successor);
    let followed = r.check();
    assert!(!about_amends(&followed), "{followed}");

    // Not accepted: the proposed one it names is reported, naming it.
    let proposed = r.adr("A proposal", &[]);
    let draft = r.adr("An amendment of a proposal", &["--amends", &proposed]);
    let out = r.check();
    assert!(
        out.lines()
            .any(|l| l.contains(&draft) && l.contains(&proposed) && l.contains("not accepted")),
        "{out}"
    );

    // Absent: an id nothing in the corpus holds.
    let absent = "ADR-0000000000ff";
    r.rewrite(&draft, &proposed, absent);
    let out = r.check();
    assert!(
        out.lines()
            .any(|l| l.contains(&draft) && l.contains(absent) && l.contains("does not exist")),
        "{out}"
    );

    // Not an ADR: a spec, which `new` refuses and a hand edit can still write.
    let spec = r
        .ok(&["new", "spec", "--title", "s", "--scope", "src/**"])
        .split_whitespace()
        .nth(1)
        .unwrap()
        .to_string();
    r.rewrite(&draft, absent, &spec);
    let out = r.check();
    assert!(
        out.lines()
            .any(|l| l.contains(&draft) && l.contains(&spec) && l.contains("not an ADR")),
        "{out}"
    );
}

#[test]
fn an_amended_adr_is_never_cold_where_a_superseded_one_is() {
    let r = Repo::new("amends-archive");
    let amended = r.adr("Amended in part", &[]);
    r.accept(&amended);
    let replaced = r.adr("Replaced whole", &[]);
    r.accept(&replaced);
    let amendment = r.adr("The amendment", &["--amends", &amended]);
    r.accept(&amendment);
    let successor = r.adr("The successor", &["--supersedes", &replaced]);
    r.accept(&successor);

    assert!(r.show(&amended).contains("\nstatus: accepted\n"));
    let dry = r.ok(&["archive", "--dry-run", "--json"]);
    assert!(dry.contains(&replaced), "the superseded one is cold: {dry}");
    assert!(!dry.contains(&amended), "the amended one is not: {dry}");

    // And an ADR both superseded and still named by an accepted amendment
    // stays hot: the amendment names it, so moving it would leave the
    // amendment naming a file the hot corpus no longer holds.
    let both = r.adr("Amended, then replaced", &[]);
    r.accept(&both);
    let second = r.adr("A second amendment", &["--amends", &both]);
    r.accept(&second);
    let third = r.adr("Its successor", &["--supersedes", &both]);
    r.accept(&third);
    let dry = r.ok(&["archive", "--dry-run", "--json"]);
    assert!(dry.contains(&replaced), "{dry}");
    assert!(
        !dry.contains(&both),
        "named by an accepted amendment: {dry}"
    );
}

#[test]
fn help_new_names_amends_under_adr_only() {
    let out = spawn(ANK)
        .args(["help", "new"])
        .current_dir(scratch::root())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    for kind in ["task", "adr", "spec"] {
        let out = spawn(ANK)
            .args(["help", "new", kind])
            .current_dir(scratch::root())
            .output()
            .unwrap();
        let page = stdout(&out);
        let flags = page
            .lines()
            .find(|l| l.trim_start().starts_with("flags:"))
            .unwrap_or_else(|| panic!("help new {kind}:\n{page}"));
        let taken = flags.split_whitespace().any(|w| w == "--amends");
        assert_eq!(taken, kind == "adr", "help new {kind}:\n{page}");
        // Refused by name on the two other kinds, never dropped in silence.
        let refused = page
            .lines()
            .skip_while(|l| !l.trim_start().starts_with("refuses:"))
            .any(|l| l.contains("--amends ") && l.ends_with("(1)"));
        assert_eq!(refused, kind != "adr", "help new {kind}:\n{page}");
    }
    assert!(text.contains("--amends"), "{text}");
}
