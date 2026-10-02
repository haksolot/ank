//! The ADRs that amend another are found through the index, and `show` stops
//! parsing every accepted ADR (TASK-ea86d1cc4af4, ADR-f3d1dea65d84).
//!
//! Through the binary: `show` names an ADR's accepted amendments once one is
//! added, once it is accepted, and after the index is deleted or left behind
//! by an older build; and what `show` on an ADR and `archive --dry-run` parse
//! does not grow with the number of accepted ADRs.

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
        let p = scratch::path("amends-cost-gitconfig");
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
}

/// The shortest form a listing prints an id in.
fn short(id: &str) -> &str {
    &id[..8]
}

/// The lines of a `show` section, from its heading to the next blank line, or
/// none when the section is absent.
fn section<'a>(text: &'a str, heading: &str) -> Vec<&'a str> {
    let mut lines = text.lines();
    if !lines
        .by_ref()
        .any(|l| l.starts_with(&format!("{heading} (")))
    {
        return Vec::new();
    }
    lines.take_while(|l| !l.is_empty()).collect()
}

impl Repo {
    fn index(&self) -> PathBuf {
        self.0.join(".ank/index.db")
    }

    /// The entity files a verb parsed, read off `ANK_TRACE_READS`.
    fn entity_reads(&self, args: &[&str]) -> Vec<String> {
        let trace = self.0.join(".git/ank-read-trace");
        let _ = fs::remove_file(&trace);
        let out = spawn(ANK)
            .args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_TRACE_READS", &trace)
            .current_dir(scratch::root())
            .output()
            .unwrap();
        assert_eq!(
            code(&out),
            0,
            "ank {args:?}: {}{}",
            stdout(&out),
            stderr(&out)
        );
        fs::read_to_string(&trace)
            .unwrap_or_default()
            .lines()
            .filter(|l| l.starts_with("entity "))
            .map(str::to_string)
            .collect()
    }

    /// The index rewritten into the shape a build before the relation left
    /// it: no `amends` column on its rows, and the schema version that build
    /// stamped. A binary that trusted it would find no amendment anywhere.
    fn age_index(&self) {
        // Built first, so there is an index to age.
        self.ok(&["status"]);
        let db = rusqlite::Connection::open(self.index()).unwrap();
        let has_column: bool = db
            .query_row(
                "SELECT count(*) FROM pragma_table_info('entities') WHERE name = 'amends'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .unwrap()
            > 0;
        if has_column {
            db.execute("ALTER TABLE entities DROP COLUMN amends", [])
                .unwrap();
        }
        db.execute(
            "UPDATE meta SET value = '10' WHERE key = 'schema_version'",
            [],
        )
        .unwrap();
    }
}

#[test]
fn show_names_the_amendment_once_added_once_accepted_and_after_the_index_is_deleted() {
    let r = Repo::new("amends-cost-show");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);

    let b = r.adr("The amendment", &["--amends", &a]);
    // Added and not ratified: it amends nothing yet.
    let shown = r.show(&a);
    assert!(
        section(&shown, "AMENDED BY").is_empty(),
        "a proposal is not listed: {shown}"
    );

    r.accept(&b);
    let shown = r.show(&a);
    let lines = section(&shown, "AMENDED BY");
    assert_eq!(lines.len(), 1, "accepted, it is listed: {shown}");
    assert!(
        lines[0].contains(short(&b)) && lines[0].contains("The amendment"),
        "{shown}"
    );

    // The index is a cache: gone, it is rebuilt and answers the same.
    fs::remove_file(r.index()).unwrap();
    let rebuilt = r.show(&a);
    assert_eq!(section(&rebuilt, "AMENDED BY"), lines, "{rebuilt}");
}

#[test]
fn an_index_left_by_an_older_build_is_rebuilt_rather_than_trusted() {
    let r = Repo::new("amends-cost-older");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let b = r.adr("The amendment", &["--amends", &a]);
    r.accept(&b);

    r.age_index();
    let shown = r.show(&a);
    let lines = section(&shown, "AMENDED BY");
    assert_eq!(lines.len(), 1, "{shown}");
    assert!(lines[0].contains(short(&b)), "{shown}");

    // And the cold list keeps the amended ADR hot from the same aged index.
    let c = r.adr("Replaced", &[]);
    r.accept(&c);
    let d = r.adr("Its successor", &["--supersedes", &c, "--amends", &a]);
    r.accept(&d);
    r.age_index();
    let dry = r.ok(&["archive", "--dry-run", "--json"]);
    assert!(dry.contains(&c), "the superseded one is cold: {dry}");
    assert!(!dry.contains(&a), "the amended one is not: {dry}");

    // A row written into this schema by a build that did not record the
    // relation -- one that opened the index before this one rebuilt it --
    // says nothing about it, and is read from its file.
    r.ok(&["status"]);
    let db = rusqlite::Connection::open(r.index()).unwrap();
    db.execute("UPDATE entities SET amends = NULL", []).unwrap();
    drop(db);
    let shown = r.show(&a);
    assert_eq!(section(&shown, "AMENDED BY").len(), 2, "{shown}");
    let dry = r.ok(&["archive", "--dry-run", "--json"]);
    assert!(dry.contains(&c) && !dry.contains(&a), "{dry}");
}

#[test]
fn show_on_an_adr_and_archive_dry_run_parse_no_accepted_adr_to_find_amendments() {
    let r = Repo::new("amends-cost-reads");
    let a = r.adr("The foundation", &[]);
    r.accept(&a);
    let b = r.adr("The amendment", &["--amends", &a]);
    r.accept(&b);
    // Accepted ADRs that amend nothing: the population the reverse lookup
    // used to parse one by one.
    for i in 0..8 {
        let id = r.adr(&format!("Bystander {i}"), &[]);
        r.accept(&id);
    }
    // Warm, so the counts below are the verbs' and not a first build's.
    r.ok(&["status"]);

    let shown = r.show(&a);
    assert_eq!(section(&shown, "AMENDED BY").len(), 1, "{shown}");
    for id in [&a, &b] {
        let reads = r.entity_reads(&["show", id]);
        assert!(
            reads.len() <= 3,
            "show {id} parsed {} entity files, where the measure before the relation \
             was 3: {reads:#?}",
            reads.len()
        );
    }
    let reads = r.entity_reads(&["archive", "--dry-run"]);
    assert!(
        reads.len() <= 1,
        "archive --dry-run parsed {} entity files, where the measure before the relation \
         was 1: {reads:#?}",
        reads.len()
    );
}
