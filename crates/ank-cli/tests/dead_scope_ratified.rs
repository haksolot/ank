//! A ratified entity's scope git records as deleted owes nothing while another
//! of its scopes lives (TASK-93add59aa1de, ADR-cb68e20acbad).
//!
//! Every case through the binary, on an accepted ADR and an accepted spec that
//! each scope two paths: one that stays, and one an act of git kills. The kill
//! is what each case varies -- one path deleted, every path deleted, one path
//! renamed -- and only the first stops owing anything. The two others are
//! pinned to the words `check` printed before the decision, so a rule that
//! silenced more than it was given would turn them red; the rename's repair is
//! the one later change, a rescope where it was a supersession
//! (ADR-bb9ccdba9be7).
//!
//! None of it writes: the decision changes what `check` says, never what the
//! corpus holds, so every case compares `git status` and the refs under
//! `refs/ank` before and after the run.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

/// A file long enough that git's similarity index recognises it after a move,
/// which a one-line file does not reliably give.
const BODY: &str = "# Old notes\n\nOne line of prose.\nA second line of prose.\n\
                    A third line of prose.\nA fourth line of prose.\nA fifth line.\n";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("dead-scope-ratified-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.com\n\tsigningkey =\n\
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

/// One finding `check` printed: its level, its sentence and its note.
#[derive(Debug, PartialEq)]
struct Finding {
    level: String,
    message: String,
    note: Vec<String>,
}

struct Repo(PathBuf);

impl Repo {
    fn new(what: &str) -> Repo {
        let dir = scratch::dir(what);
        fs::create_dir_all(dir.join(".ank/entities")).unwrap();
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("docs")).unwrap();
        fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(dir.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        fs::write(dir.join("docs/old.md"), BODY).unwrap();
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

    fn git(&self, args: &[&str]) -> String {
        let out = spawn("git")
            .args(args)
            .current_dir(&self.0)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
        text(&out.stdout)
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
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    fn created(&self, args: &[&str]) -> String {
        self.ok(args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    /// An accepted ADR and an accepted spec, each scoping `scopes`, committed
    /// and ratified: the state ADR-cb68e20acbad decides something about.
    fn ratified(&self, scopes: &[&str]) -> [String; 2] {
        let mut adr = vec![
            "new",
            "adr",
            "--title",
            "A rule",
            "--constraint",
            "A binding rule.",
        ];
        let mut spec = vec!["new", "spec", "--title", "A document"];
        for s in scopes {
            adr.extend(["--scope", s]);
            spec.extend(["--scope", s]);
        }
        let ids = [self.created(&adr), self.created(&spec)];
        self.commit("propose");
        for id in &ids {
            self.ok(&["accept", id]);
        }
        self.commit("ratify");
        ids
    }

    /// What `check` could have written: the working tree as git sees it, and
    /// every ref under `refs/ank` with what it points at.
    fn written(&self) -> (String, String) {
        (
            self.git(&["status", "--porcelain", "--untracked-files=all"]),
            self.git(&["for-each-ref", "refs/ank"]),
        )
    }

    /// `check`, asserted to have written nothing: its exit code, and the
    /// dead-scope findings about `id` as `check --kind dead-scope` prints them.
    fn check(&self, id: &str) -> (i32, Vec<Finding>) {
        let before = self.written();
        let whole = self.ank(&["check"]);
        let out = self.ank(&["check", "--kind", "dead-scope"]);
        for out in [&whole, &out] {
            assert!(
                [0, 8].contains(&code(out)),
                "check: {}{}",
                text(&out.stdout),
                text(&out.stderr)
            );
        }
        assert_eq!(
            self.written(),
            before,
            "check wrote to the tree or to refs/ank"
        );
        (code(&whole), findings(&text(&out.stdout), id))
    }

    /// The commit at HEAD, in the twelve characters a note names it by.
    fn head(&self) -> String {
        self.git(&["rev-parse", "--short=12", "HEAD"])
            .trim()
            .to_string()
    }
}

/// The findings `check` printed about `id`: a `level: subject: message` line,
/// and the note hung beneath it, one entry per `└── ` line and the lines
/// indented under it.
fn findings(printed: &str, id: &str) -> Vec<Finding> {
    let mut found: Vec<Finding> = Vec::new();
    let mut ours = false;
    for line in printed.lines() {
        if let Some(note) = line
            .strip_prefix("└── ")
            .or_else(|| line.strip_prefix("    "))
        {
            if ours {
                found.last_mut().unwrap().note.push(note.to_string());
            }
            continue;
        }
        let mut parts = line.splitn(3, ": ");
        let (Some(level), Some(subject), Some(message)) =
            (parts.next(), parts.next(), parts.next())
        else {
            ours = false;
            continue;
        };
        ours = subject == id && matches!(level, "signal" | "fault");
        if ours {
            found.push(Finding {
                level: level.to_string(),
                message: message.to_string(),
                note: Vec::new(),
            });
        }
    }
    found
}

#[test]
fn a_deleted_scope_beside_a_live_one_owes_nothing_and_names_the_commit() {
    let r = Repo::new("dead-scope-ratified-deleted");
    let ids = r.ratified(&["src/main.rs", "docs/old.md"]);
    r.git(&["rm", "-q", "docs/old.md"]);
    r.commit("drop the notes");
    let sha = r.head();

    for id in &ids {
        let (exit, findings) = r.check(id);
        assert_eq!(exit, 0, "{id}: nothing about this corpus is a fault");
        assert_eq!(
            findings,
            vec![Finding {
                level: "signal".into(),
                message: "scope 'docs/old.md' matches no file, git records it deleted, \
                          and another scope still matches: nothing is owed"
                    .into(),
                note: vec![format!("git records docs/old.md deleted in {sha}")],
            }],
            "{id}"
        );
    }
}

#[test]
fn every_scope_deleted_is_reported_as_it_was() {
    let r = Repo::new("dead-scope-ratified-all");
    let ids = r.ratified(&["src/lib.rs", "docs/old.md"]);
    r.git(&["rm", "-q", "docs/old.md", "src/lib.rs"]);
    r.commit("drop both");
    let sha = r.head();

    for id in &ids {
        let (_, findings) = r.check(id);
        let dead = |glob: &str| Finding {
            level: "signal".into(),
            message: format!("dead scope '{glob}': no file matches it"),
            note: vec![format!("git records {glob} deleted in {sha}")],
        };
        assert_eq!(
            findings,
            vec![dead("docs/old.md"), dead("src/lib.rs")],
            "{id}: an entity that governs nothing says so"
        );
    }
}

#[test]
fn a_renamed_scope_beside_a_live_one_is_reported_as_it_was() {
    let r = Repo::new("dead-scope-ratified-renamed");
    let ids = r.ratified(&["src/main.rs", "docs/old.md"]);
    r.git(&["mv", "docs/old.md", "docs/new.md"]);
    r.commit("move the notes");
    let sha = r.head();

    for id in &ids {
        let (exit, findings) = r.check(id);
        assert_eq!(exit, 0, "{id}: a rename git records is no fault");
        // The perimeter follows the file by a rescope, and no longer by a
        // supersession (ADR-bb9ccdba9be7), for either kind.
        let rescope = format!(
            "ank amend {id} --drop-scope \"docs/old.md\" --scope \"docs/new.md\", \
             then ank accept {id} --rescope"
        );
        assert_eq!(
            findings,
            vec![Finding {
                level: "signal".into(),
                message: "dead scope 'docs/old.md': no file matches it".into(),
                note: vec![
                    format!("git records docs/old.md renamed to docs/new.md in {sha}"),
                    rescope,
                ],
            }],
            "{id}: the perimeter ought to follow the file, so the finding stays"
        );
    }
}
