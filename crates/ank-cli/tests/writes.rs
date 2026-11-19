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
        // Every directory a verb could write the reader's state into, under
        // the same scratch home: the `outside` measurement counts what lands
        // here, and a directory left pointing at the machine's would let a
        // write escape the count rather than fail it.
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("XDG_DATA_HOME", home.join(".local/share"))
        .env("XDG_STATE_HOME", home.join(".local/state"))
        .env("XDG_CACHE_HOME", home.join(".cache"))
        .env("APPDATA", home.join("AppData/Roaming"))
        .env("LOCALAPPDATA", home.join("AppData/Local"))
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

/// Each verb and what it declares writing outside the corpus, read back from
/// the binary as [`declarations`] reads `writes`.
fn outside_declarations(repo: &Path) -> Vec<(String, Option<String>)> {
    let out = ank(repo, AGENT, &["help", "--json"]);
    assert!(out.status.success(), "help --json must answer");
    let doc: serde_yaml::Value =
        serde_yaml::from_str(&String::from_utf8_lossy(&out.stdout)).unwrap();
    doc["verbs"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|v| {
            (
                v["name"].as_str().unwrap().to_string(),
                v["outside"].as_str().map(str::to_string),
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

/// The two values `outside` takes.
const OUTSIDE_VALUES: &[&str] = &["nothing", "files"];

/// How each verb declaring `outside: nothing` is run, in order, on one corpus:
/// the corpus writers too, since `outside` is a second question asked of every
/// verb and `writes` answering it for the corpus says nothing about the home.
///
/// **A verb declaring `outside: nothing` with no line here is a red**, as in
/// [`READERS`]. A line is measured whatever the verb answers: a refusal that
/// wrote into the home is as much a write as a success that did.
const QUIET_OUTSIDE: &[(&str, &[&str])] = &[
    ("context", &["context", "--json"]),
    ("find", &["find", "task"]),
    ("status", &["status", "--json"]),
    ("review", &["review"]),
    ("graph", &["graph"]),
    ("scope", &["scope", "src/main.rs"]),
    ("verify", &["verify"]),
    ("help", &["help", "--json"]),
    ("check", &["check"]),
    ("claim", &["claim", OPEN]),
    ("show", &["show", OPEN]),
    ("log", &["log", "measured outside the corpus"]),
    ("amend", &["amend", OPEN, "--scope", "docs/**"]),
    ("edit", &["edit", OPEN, "--title", "The open task, renamed"]),
    ("read", &["read", OPEN]),
    ("attest", &["attest", OPEN, "--proof", "test:1"]),
    ("done", &["done", "--proof", "commit:0000000"]),
    ("release", &["release", OPEN, "--reason", "measured"]),
    (
        "new",
        &[
            "new",
            "adr",
            "--title",
            "A decision",
            "--scope",
            "src/**",
            "--constraint",
            "A rule.",
        ],
    ),
    ("accept", &["accept", "ADR-000000000000"]),
    (
        "reject",
        &["reject", "ADR-000000000000", "--reason", "measured"],
    ),
    ("close", &["close", OPEN, "--reason", "measured"]),
    (
        "signers",
        &["signers", "add", "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5"],
    ),
    ("migrate", &["migrate"]),
    ("archive", &["archive"]),
];

/// The verbs declaring `outside: files`, each with the call that shows it or
/// the reason no test can run one.
///
/// A call here must leave something under the scratch home: the positive side
/// of the measurement, so that `files` is not a word a verb can wear for free
/// while writing nothing anywhere. `None` is a verb whose write cannot be made
/// on a scratch machine, with the reason beside it.
const LOUD_OUTSIDE: &[(&str, Option<&[&str]>, &str)] = &[
    (
        "hooks",
        Some(&["hooks", "install", "claude"]),
        "the harness's settings.json",
    ),
    (
        "config",
        Some(&[
            "config",
            "--user",
            "corpora.0000000000000000000000000000000000000000",
            "/nowhere",
        ]),
        "the reader's corpora.yml",
    ),
    (
        "init",
        None,
        "a detached init declares the corpus in corpora.yml, and needs a second repository to detach into",
    ),
    (
        "watch",
        None,
        "it writes events.jsonl beside watch.yml while it runs, and runs until stopped",
    ),
    (
        "skills",
        None,
        "--install hands a directory to npx skills add, which installs from the network",
    ),
    (
        "update",
        None,
        "it replaces this binary with a release it downloads",
    ),
    (
        "tui",
        None,
        "it composes config --user among the verbs it spells, and needs a terminal to run at all",
    ),
    (
        "mcp",
        None,
        "it is the server, and runs any verb a client calls, the writers above included",
    ),
];

/// Every file under the scratch home, as a sorted list of paths.
fn home_files(home: &Path) -> Vec<String> {
    fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            match path.is_dir() {
                true => walk(&path, base, out),
                false => out.push(
                    path.strip_prefix(base)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                ),
            }
        }
    }
    let mut out = Vec::new();
    walk(home, home, &mut out);
    out.sort();
    out
}

/// Every verb says what it writes outside the corpus, and both answers are
/// measured under a scratch home and scratch XDG directories (GitHub #781).
///
/// **`writes` stays the corpus, as ADR-3c8d94a0bbe0 gives it**, and `outside`
/// is the second field that answers for everything else: the harness's
/// settings, the reader's declarations, a skill directory, the binary itself.
/// A verb declaring `nothing` is run and must leave the home as it found it; a
/// verb declaring `files` is run where a scratch machine can run it and must
/// leave something, so neither word can be worn for free.
#[test]
fn a_verb_declaring_nothing_outside_leaves_the_home_untouched() {
    let repo = corpus();
    let home = repo.parent().unwrap().join("home");
    let declared = outside_declarations(&repo);
    let wrong: Vec<String> = declared
        .iter()
        .filter(|(_, o)| !o.as_deref().is_some_and(|o| OUTSIDE_VALUES.contains(&o)))
        .map(|(name, o)| format!("{name}: {o:?}"))
        .collect();
    assert!(
        wrong.is_empty(),
        "every verb declares outside, one of {OUTSIDE_VALUES:?}; these do not: {wrong:?}"
    );
    let side = |value: &str| -> Vec<&str> {
        declared
            .iter()
            .filter(|(_, o)| o.as_deref() == Some(value))
            .map(|(n, _)| n.as_str())
            .collect()
    };
    let (quiet, loud) = (side("nothing"), side("files"));
    let missing: Vec<&&str> = quiet
        .iter()
        .filter(|v| !QUIET_OUTSIDE.iter().any(|(q, _)| q == *v))
        .collect();
    assert!(
        missing.is_empty(),
        "these verbs declare outside: nothing and QUIET_OUTSIDE does not run them: {missing:?}"
    );
    let mut stale: Vec<&str> = QUIET_OUTSIDE
        .iter()
        .map(|(q, _)| *q)
        .filter(|q| !quiet.contains(q))
        .collect();
    stale.dedup();
    assert!(
        stale.is_empty(),
        "QUIET_OUTSIDE runs verbs that declare files: {stale:?}"
    );
    let listed: Vec<&str> = LOUD_OUTSIDE.iter().map(|(v, _, _)| *v).collect();
    let mut a = loud.clone();
    let mut b = listed.clone();
    a.sort();
    b.sort();
    assert_eq!(
        a, b,
        "the verbs declaring outside: files are exactly the ones LOUD_OUTSIDE accounts for"
    );

    let before = home_files(&home);
    let mut wrote = Vec::new();
    for (verb, args) in QUIET_OUTSIDE {
        ank(&repo, AGENT, args);
        let after = home_files(&home);
        if after != before {
            wrote.push(format!("{verb} ({args:?}): {before:?} -> {after:?}"));
            let _ = fs::remove_dir_all(&home);
            fs::create_dir_all(&home).unwrap();
        }
    }
    assert!(
        wrote.is_empty(),
        "verbs declaring outside: nothing wrote under the home:\n{}",
        wrote.join("\n")
    );

    for (verb, args, what) in LOUD_OUTSIDE {
        let Some(args) = args else { continue };
        let before = home_files(&home);
        let out = ank(&repo, AGENT, args);
        let after = home_files(&home);
        assert_ne!(
            before,
            after,
            "{verb} declares outside: files ({what}) and {args:?} left the home as it was: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
