//! Every verb says whether it writes, and the ones that say nothing are held to
//! it (ADR-3c8d94a0bbe0).
//!
//! The decision it replaces enumerated the reading verbs and the writing verbs
//! inside its constraint, and the enumeration went stale once per new route:
//! `archive` and `migrate` wrote `.ank/` and were on neither list. So the list
//! is gone and each `CommandSpec` declares `writes` instead, which
//! `help --json` carries. A declaration nobody confronts is a list by another
//! name, so this suite runs every verb declaring `nothing` on a scratch corpus
//! and counts what it leaves behind, the measurement once made by hand: the
//! paths `git status --porcelain` reports under `.ank/`, with `index.db`
//! ignored, and the refs under `refs/ank/` with what each points at. Counting
//! paths rather than hashing files is deliberate: a measurement of the rule
//! should not be the thing the rule forbids.
//!
//! **What `writes` declares is the verb's own act, and a lease renewal is not
//! one.** `show` on the task the caller holds moves its claim ref, and `renews`
//! already declares that per verb (ADR-0bb7ea8991bc). So the corpus is measured
//! with no claim held by the caller, and with a claim held by somebody else and
//! an orphan claim ref beside it, so a verb that pruned or renewed what is not
//! its own would show.
//!
//! Through the binary, because the criterion is a statement about what the
//! binary dispatches: the declarations are read back from `help --json`, never
//! from the contract crate, so a verb added to the table and missing from the
//! rendering is the same red as a verb missing a declaration.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "tool/1.0";
const OTHER: &str = "other/1.0";
const HELD: &str = "TASK-000000000001";
const OPEN: &str = "TASK-000000000002";
const ORPHAN: &str = "refs/ank/claims/TASK-0000000000ff";

/// The four values `writes` takes, and nothing else.
const VALUES: &[&str] = &["nothing", "files", "refs", "both"];

/// How each verb declaring `nothing` is run. Several lines for a verb that has
/// several forms: a reader's `--json` path is a second route through the code.
///
/// **A verb declaring `nothing` with no line here is a red**, and so is a line
/// naming a verb that declares something else: the table is the measurement,
/// and a verb added to the reading side on a later day is measured on that day.
const READERS: &[(&str, &[&str])] = &[
    ("context", &["context"]),
    ("context", &["context", "src/**", "--json"]),
    ("show", &["show", HELD]),
    ("show", &["show", OPEN, "--json"]),
    ("find", &["find", "task"]),
    ("find", &["find", "--status", "open", "--json"]),
    ("status", &["status"]),
    ("status", &["status", "--json"]),
    ("review", &["review"]),
    ("review", &["review", "--json"]),
    ("graph", &["graph"]),
    ("graph", &["graph", "--json"]),
    ("scope", &["scope", "src/main.rs"]),
    ("scope", &["scope", "src/main.rs", "--json"]),
    ("skills", &["skills"]),
    ("skills", &["skills", "--json"]),
    // What it writes is the agent's settings, under the scratch `HOME` and
    // `CLAUDE_CONFIG_DIR` [`ank`] sets, and never the corpus it runs in.
    ("hooks", &["hooks", "install", "claude"]),
    ("hooks", &["hooks", "install", "claude", "--json"]),
    ("hooks", &["hooks", "remove", "claude"]),
    ("hooks", &["hooks", "run", "claude", "session-start"]),
    // No verifier is declared here, so the measurement is of the verb and its
    // selection alone; what it records lives under the git common directory.
    ("verify", &["verify"]),
    ("verify", &["verify", "--json"]),
    ("help", &["help"]),
    ("help", &["help", "--json"]),
];

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("writes-gitconfig");
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
        .env("GIT_CONFIG_SYSTEM", config);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn ank(dir: &Path, agent: &str, args: &[&str]) -> Output {
    let home = dir.parent().unwrap().join("home");
    spawn(ANK)
        .args(args)
        .current_dir(dir)
        .env("ANK_AGENT", agent)
        .env("NO_COLOR", "1")
        // `skills` reads where the skills are installed, `hooks` writes the
        // agent's settings, and `update --check` asks a release repository:
        // none may reach this machine's own.
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .env("CLAUDE_CONFIG_DIR", home.join(".claude"))
        .env(
            "ANK_UPDATE_REPOSITORY",
            dir.parent().unwrap().join("no-release.git"),
        )
        .output()
        .expect("the binary must run")
}

fn task(id: &str, title: &str) -> String {
    format!(
        "---\nid: {id}\ntype: task\nslug: example\ntitle: {title}\n\
         created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
         blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
         criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
    )
}

/// Two tasks, one claimed by another agent, and an orphan claim ref copied from
/// that claim onto an id no branch carries.
fn corpus() -> PathBuf {
    let base = scratch::dir("writes");
    let repo = base.join("repo");
    fs::create_dir_all(repo.join(".ank/entities")).unwrap();
    fs::create_dir_all(repo.join("src")).unwrap();
    fs::create_dir_all(base.join("home")).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    fs::write(
        repo.join(".ank/config.yml"),
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
    )
    .unwrap();
    fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
    fs::write(repo.join("src/main.rs"), "fn main() {}\n").unwrap();
    for (id, title) in [(HELD, "The held task"), (OPEN, "The open task")] {
        fs::write(
            repo.join(".ank/entities").join(format!("{id}.md")),
            task(id, title),
        )
        .unwrap();
    }
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    let out = ank(&repo, OTHER, &["claim", HELD]);
    assert!(
        out.status.success(),
        "the other agent's claim: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The claim moves the task's version; committed, so the tree a reader is
    // measured against is clean and putting it back is exact.
    git(&repo, &["commit", "-qam", "claimed"]);
    let claim = git(&repo, &["rev-parse", &format!("refs/ank/claims/{HELD}")]);
    git(&repo, &["update-ref", ORPHAN, &claim]);
    repo
}

/// What a verb could have left: the paths git reports under `.ank/`, and every
/// ref under `refs/ank/` with the object it names.
fn snapshot(repo: &Path) -> (String, String) {
    (
        git(
            repo,
            &[
                "status",
                "--porcelain",
                "--untracked-files=all",
                "--",
                ".ank",
            ],
        ),
        git(
            repo,
            &[
                "for-each-ref",
                "--format=%(refname) %(objectname)",
                "refs/ank/",
            ],
        ),
    )
}

/// Each verb and what it declares, read back from the binary.
fn declarations(repo: &Path) -> Vec<(String, Option<String>)> {
    let out = ank(repo, AGENT, &["help", "--json"]);
    assert!(out.status.success(), "help --json must answer");
    // JSON is YAML, and this suite parses it the way the rest of it does.
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    doc["verbs"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["name"].as_str().unwrap().to_string(),
                v["writes"].as_str().map(str::to_string),
            )
        })
        .collect()
}

#[test]
fn help_json_declares_writes_on_every_verb() {
    let repo = corpus();
    let declared = declarations(&repo);
    assert!(declared.len() > 20, "the listing is the whole table");
    let wrong: Vec<String> = declared
        .iter()
        .filter(|(_, w)| !w.as_deref().is_some_and(|w| VALUES.contains(&w)))
        .map(|(name, w)| format!("{name}: {w:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "every verb declares writes, one of {VALUES:?}; these do not: {wrong:?}"
    );
    // The writing side is not empty either: a table where everything reads
    // would pass the measurement below by declaring nothing about anything.
    for (verb, side) in [
        ("new", "files"),
        ("claim", "both"),
        ("done", "both"),
        ("check", "refs"),
        ("archive", "files"),
        ("migrate", "files"),
    ] {
        let found = declared.iter().find(|(n, _)| n == verb).unwrap();
        assert_eq!(found.1.as_deref(), Some(side), "{verb} declares {side}");
    }
}

#[test]
fn a_verb_declaring_nothing_leaves_nothing_under_ank_or_refs_ank() {
    let repo = corpus();
    let declared = declarations(&repo);
    let readers: Vec<&str> = declared
        .iter()
        .filter(|(_, w)| w.as_deref() == Some("nothing"))
        .map(|(n, _)| n.as_str())
        .collect();
    let unmeasured: Vec<&&str> = readers
        .iter()
        .filter(|v| !READERS.iter().any(|(r, _)| r == *v))
        .collect();
    assert!(
        unmeasured.is_empty(),
        "these verbs declare nothing and READERS does not run them: {unmeasured:?}"
    );
    let mut stale: Vec<&str> = READERS
        .iter()
        .map(|(r, _)| *r)
        .filter(|r| !readers.contains(r))
        .collect();
    stale.dedup();
    assert!(
        stale.is_empty(),
        "READERS runs verbs that no longer declare nothing: {stale:?}"
    );

    // Warm the index first: building it is a cache under `.ank/`, ignored by
    // git, and the first verb of any session would otherwise be the one
    // measured doing it.
    ank(&repo, AGENT, &["find", "task"]);
    let before = snapshot(&repo);
    assert_eq!(before.0, "", "the tree starts clean");
    assert!(
        before.1.contains(ORPHAN),
        "the orphan is in place to be pruned: {}",
        before.1
    );
    let mut wrote = Vec::new();
    for (verb, args) in READERS {
        ank(&repo, AGENT, args);
        let after = snapshot(&repo);
        if after != before {
            wrote.push(format!(
                "{verb} ({args:?})\n  .ank/ was {:?}, is {:?}\n  refs/ank/ was {:?}, is {:?}",
                before.0, after.0, before.1, after.1
            ));
            // Put the corpus back, so one offender does not stand in for all.
            git(&repo, &["checkout", "-q", "--", "."]);
            git(&repo, &["clean", "-qfd", "--", ".ank"]);
            for line in before.1.lines() {
                let (name, sha) = line.split_once(' ').unwrap();
                git(&repo, &["update-ref", name, sha]);
            }
        }
    }
    // A ref a reader created would survive the restore above; it is already
    // reported, so it is enough that the loop named it.
    assert!(
        wrote.is_empty(),
        "verbs declaring writes: nothing changed the corpus:\n{}",
        wrote.join("\n")
    );
}
