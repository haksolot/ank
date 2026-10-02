//! `context` marks an accepted ADR that an accepted amendment changes, and
//! names the amendment (TASK-1d8e9d7f853d, ADR-9ee76b578257).
//!
//! The amended ADR stays binding and is served as it always was; what changes
//! is the mark beside it, because `context` is the verb an agent reads before
//! touching a perimeter, and an agent that reads the amended constraint and
//! not its amendment applies the clause the amendment replaced.
//!
//! Every case through the binary: the orientation listing, the full
//! constraints served under a claim, `--json` in both modes, a proposed
//! amendment that marks nothing, and a budget that pays for the mark.

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
        let p = scratch::path("amends-context-gitconfig");
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
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\n").unwrap();
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

    /// The id `new` printed, from `created <id> <slug>`.
    fn created(&self, args: &[&str]) -> String {
        self.ok(args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    fn adr(&self, title: &str, constraint: &str, extra: &[&str]) -> String {
        let mut args = vec![
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            constraint,
        ];
        args.extend_from_slice(extra);
        self.created(&args)
    }

    fn task(&self, title: &str) -> String {
        self.created(&[
            "new",
            "task",
            "--title",
            title,
            "--scope",
            "src/**",
            "--criteria",
            "It holds.",
            "--no-verify",
        ])
    }

    /// Committed, then ratified: `accept` works on the default branch and on
    /// what is committed there.
    fn accept(&self, id: &str) {
        self.commit(&format!("propose {id}"));
        self.ok(&["accept", id]);
    }

    fn context(&self, extra: &[&str]) -> String {
        let mut args = vec!["context", "src/main.rs"];
        args.extend_from_slice(extra);
        self.ok(&args)
    }

    /// `context` with no path, which under a claim is the execution page.
    fn held(&self, extra: &[&str]) -> String {
        let mut args = vec!["context"];
        args.extend_from_slice(extra);
        self.ok(&args)
    }
}

/// The shortest form a listing prints an id in: the kind and four digits,
/// which is what a corpus this small disambiguates on.
fn short(id: &str) -> &str {
    &id[..8]
}

/// The line of a page that lists `id`, as the listing prints it.
fn line_of<'a>(page: &'a str, id: &str) -> &'a str {
    page.lines()
        .find(|l| l.trim_start().starts_with(short(id)))
        .unwrap_or_else(|| panic!("{id} is not listed:\n{page}"))
}

/// The value of `"amended_by"` on the constraint `id` of a `context --json`
/// document, as its raw array text.
fn amended_by_json<'a>(doc: &'a str, id: &str) -> &'a str {
    let at = doc
        .find(&format!("{{\"id\":\"{id}\""))
        .unwrap_or_else(|| panic!("{id} not in {doc}"));
    let object = &doc[at..at + doc[at..].find('}').expect("an object closes")];
    let key = "\"amended_by\":";
    let from = object
        .find(key)
        .unwrap_or_else(|| panic!("no amended_by on {id}: {object}"))
        + key.len();
    let rest = &object[from..];
    &rest[..=rest.find(']').expect("an array closes")]
}

/// The corpus every case starts from: an ADR, an accepted amendment of it,
/// and an ADR a proposal only amends, all three binding `src/**`.
struct Amended {
    repo: Repo,
    amended: String,
    amendment: String,
    untouched: String,
    proposal: String,
}

fn amended(what: &str) -> Amended {
    let repo = Repo::new(what);
    let amended = repo.adr("The foundation", "Every module logs.", &[]);
    repo.accept(&amended);
    let untouched = repo.adr("The other foundation", "Every module tests.", &[]);
    repo.accept(&untouched);
    let amendment = repo.adr(
        "The amendment",
        "A module under src/net logs nothing.",
        &["--amends", &amended],
    );
    repo.accept(&amendment);
    let proposal = repo.adr(
        "A proposed amendment",
        "A module under src/io tests nothing.",
        &["--amends", &untouched],
    );
    repo.commit("propose an amendment");
    Amended {
        repo,
        amended,
        amendment,
        untouched,
        proposal,
    }
}

#[test]
fn the_listing_marks_the_amended_adr_and_names_its_amendment() {
    let c = amended("amends-context-listing");
    let page = c.repo.context(&[]);

    let line = line_of(&page, &c.amended);
    assert!(
        line.contains(&format!("amended by {}", short(&c.amendment))),
        "the amended ADR names its amendment: {line:?}\n{page}"
    );
    // The amendment itself amends something and is amended by nothing, and the
    // ADR only a proposal names has changed in no way anyone ratified.
    for id in [&c.amendment, &c.untouched] {
        let line = line_of(&page, id);
        assert!(!line.contains("amended by"), "{line:?}\n{page}");
    }
    assert!(
        page.contains(short(&c.proposal)),
        "the proposal is still listed as one: {page}"
    );
}

#[test]
fn the_json_document_carries_the_relation_in_both_modes() {
    let c = amended("amends-context-json");
    let doc = c.repo.context(&["--json"]);
    assert_eq!(
        amended_by_json(&doc, &c.amended),
        format!("[\"{}\"]", c.amendment),
        "{doc}"
    );
    assert_eq!(amended_by_json(&doc, &c.amendment), "[]", "{doc}");
    assert_eq!(amended_by_json(&doc, &c.untouched), "[]", "{doc}");

    let task = c.repo.task("Touch src");
    c.repo.commit("a task");
    c.repo.ok(&["claim", &task]);
    let doc = c.repo.held(&["--json"]);
    assert!(doc.contains("\"mode\":\"execution\""), "{doc}");
    assert_eq!(
        amended_by_json(&doc, &c.amended),
        format!("[\"{}\"]", c.amendment),
        "{doc}"
    );
    assert_eq!(amended_by_json(&doc, &c.untouched), "[]", "{doc}");
}

#[test]
fn the_full_constraints_under_a_claim_mark_the_amended_adr() {
    let c = amended("amends-context-claim");
    let task = c.repo.task("Touch src");
    c.repo.commit("a task");
    c.repo.ok(&["claim", &task]);
    let page = c.repo.held(&[]);

    // The amended rule is still served whole, and the mark follows it, before
    // the next rule begins.
    let at = page
        .find("Every module logs.")
        .unwrap_or_else(|| panic!("the amended rule is served in full:\n{page}"));
    let after = &page[at..];
    let mark = after
        .lines()
        .nth(1)
        .unwrap_or_else(|| panic!("nothing follows the rule:\n{page}"));
    assert!(
        mark.contains(&format!("amended by {}", short(&c.amendment)))
            && mark.contains("The amendment"),
        "the line after the amended rule names the amendment: {mark:?}\n{page}"
    );
    // Only the one: neither the amendment nor the ADR a proposal names.
    assert_eq!(page.matches("amended by").count(), 1, "{page}");
}

#[test]
fn the_budget_pays_for_the_mark() {
    // Enough tasks that the page is long and the constraints' third is wide,
    // so the cut that has to happen falls on a task and not on the mark.
    let c = amended("amends-context-budget");
    for n in 0..12 {
        c.repo.task(&format!(
            "A task with a title long enough to cost, number {n}"
        ));
    }
    c.repo.commit("tasks");
    let full = c.repo.context(&[]);
    assert!(full.contains("amended by"), "{full}");
    assert!(!full.contains("more tasks"), "nothing is cut yet: {full}");

    // One character short of the page with its mark: a budget that priced the
    // page without the mark would find it fitting and print it whole, over the
    // budget by the mark less one.
    let budget = full.chars().count() - 1;
    c.repo
        .ok(&["config", "context_budget", &budget.to_string()]);
    let page = c.repo.context(&[]);
    assert!(
        page.chars().count() <= budget,
        "{} characters against a budget of {budget}:\n{page}",
        page.chars().count()
    );
    assert!(
        page.contains("more tasks") && page.contains("amended by"),
        "a task is cut and the mark survives:\n{page}"
    );
}
