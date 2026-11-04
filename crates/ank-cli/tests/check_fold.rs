//! A finding carries a kind, and a kind firing past the threshold is reported
//! once for the corpus (TASK-9e7a652aa4f9, ADR-df1035934387).
//!
//! Every case through the binary, each on a corpus of its own. The threshold is
//! the smaller of 20 and five percent of the hot corpus, so every fixture is
//! padded with 190 tasks a human wrote and nothing reports per entity: 200 hot
//! files hold the threshold at 10 with ten entities firing, and 220 hold it at
//! 11 with thirty. Ten stays per entity and thirty folds, on the same terms for
//! the three kinds the criterion names: read by no human, a verifier changed
//! since the proof, and a dead scope. A fault is never folded, and `review` and
//! `status` count a folded signal as the one finding `check` prints.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

/// The tasks nothing reports per entity, holding the hot corpus at a size
/// where the threshold is ten rather than zero.
const PADDING: usize = 190;

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("check-fold-gitconfig");
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
        .env_remove("NO_COLOR")
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

fn task_id(n: usize) -> String {
    format!("TASK-{n:012x}")
}

fn adr_id(n: usize) -> String {
    format!("ADR-{n:012x}")
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
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             verifiers:\n  t:\n    run: \"true\"\n",
        )
        .unwrap();
        fs::write(r.0.join(".gitignore"), ".ank/index.db*\nbuild/\n").unwrap();
        for n in 0..PADDING {
            r.task(0x100 + n, "human:marie", "open", "");
        }
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

    fn commit(&self) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "seed", "--allow-empty"]);
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

    /// `check`'s human output, which exits 0 on signals alone.
    fn check(&self, args: &[&str]) -> String {
        let mut all = vec!["check"];
        all.extend_from_slice(args);
        let out = self.ank(&all);
        assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
        stdout(&out)
    }

    /// A task in canonical form, with what a fixture varies: who wrote it, its
    /// status, and whatever frontmatter the status needs.
    fn task(&self, n: usize, author: &str, status: &str, proof: &str) {
        let id = task_id(n);
        fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example\n\
                 created: 2026-07-28T00:00:00Z\nauthor: {author}\nstatus: {status}\n\
                 scope:\n  - src/**\nblocked_by: []\n\
                 done_criteria: |\n  A verifiable criterion.\ncriteria_by: creator\n\
                 {proof}schema: 4\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// A task blocked by a task this corpus does not hold: a fault, one each.
    fn blocked_by_nothing(&self, n: usize) {
        let id = task_id(n);
        let missing = task_id(0xfff000 + n);
        fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example\n\
                 created: 2026-07-28T00:00:00Z\nauthor: human:marie\nstatus: open\n\
                 scope:\n  - src/**\nblocked_by: [{missing}]\n\
                 done_criteria: |\n  A verifiable criterion.\ncriteria_by: creator\n\
                 schema: 4\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// A proposed ADR whose scope matches only a file git ignores: a dead
    /// scope, and a signal rather than a fault (ADR-3abc).
    fn ignored_adr(&self, n: usize) {
        let id = adr_id(n);
        let dir = self.0.join(format!("build/x{n}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("out.txt"), "built\n").unwrap();
        fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                 created: 2026-07-20T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
                 scope:\n  - build/x{n}/**\nconstraint: |\n  A rule.\n\
                 schema: 4\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }
}

/// Lines of `check` naming one entity with this message.
fn per_entity(said: &str, message: &str) -> usize {
    said.lines()
        .filter(|l| {
            (l.starts_with("signal: TASK-") || l.starts_with("signal: ADR-")) && l.contains(message)
        })
        .count()
}

/// The one corpus line of a folded kind, and the note under it.
fn folded<'a>(said: &'a str, kind: &str) -> Vec<&'a str> {
    let lines: Vec<&str> = said.lines().collect();
    let Some(at) = lines
        .iter()
        .position(|l| l.starts_with("signal: corpus:") && l.contains(&format!("{kind} ")))
    else {
        return Vec::new();
    };
    let note = lines[at + 1..]
        .iter()
        .take_while(|l| l.starts_with("└── ") || l.starts_with("    "));
    std::iter::once(&lines[at]).chain(note).copied().collect()
}

/// What the fold owes the reader: the kind, the count, five identifiers and the
/// command that lists the rest.
fn assert_folded(said: &str, kind: &str, count: usize, ids: &[String]) {
    let block = folded(said, kind).join("\n");
    assert!(!block.is_empty(), "no corpus line for {kind}:\n{said}");
    assert!(
        block.contains(&format!("{count} entities")),
        "the count:\n{block}"
    );
    let named = ids.iter().filter(|id| block.contains(id.as_str())).count();
    assert_eq!(named, 5, "five identifiers, and only five:\n{block}");
    assert!(
        block.contains(&format!("ank check --kind {kind}")),
        "the command that lists the rest:\n{block}"
    );
    assert_eq!(
        said.lines()
            .filter(|l| l.starts_with("signal: corpus:") && l.contains(&format!("{kind} ")))
            .count(),
        1,
        "one signal for the corpus:\n{said}"
    );
}

fn agent_corpus(what: &str, unread: usize) -> (Repo, Vec<String>) {
    let r = Repo::new(what);
    let ids: Vec<String> = (0..unread)
        .map(|n| {
            r.task(0x900 + n, "claude-code/opus-5", "open", "");
            task_id(0x900 + n)
        })
        .collect();
    r.commit();
    (r, ids)
}

#[test]
fn thirty_entities_read_by_no_human_are_one_corpus_signal() {
    let (r, ids) = agent_corpus("fold-unread-30", 30);
    let said = r.check(&[]);
    assert_eq!(
        per_entity(&said, "written by an agent and read by no human"),
        0,
        "no per-entity line of a folded kind:\n{said}"
    );
    assert_folded(&said, "read-by-no-human", 30, &ids);

    // The command the fold names lists every one of them, per entity.
    let listed = r.check(&["--kind", "read-by-no-human"]);
    assert_eq!(
        per_entity(&listed, "written by an agent and read by no human"),
        30,
        "{listed}"
    );
    assert!(
        !listed.contains("matches no file"),
        "--kind lists that kind and no other:\n{listed}"
    );
}

#[test]
fn ten_entities_read_by_no_human_stay_per_entity() {
    let (r, _) = agent_corpus("fold-unread-10", 10);
    let said = r.check(&[]);
    assert_eq!(
        per_entity(&said, "written by an agent and read by no human"),
        10,
        "{said}"
    );
    assert!(folded(&said, "read-by-no-human").is_empty(), "{said}");
}

#[test]
fn a_verifier_changed_since_thirty_proofs_is_one_corpus_signal() {
    let proof = "proof:\n  - type: test\n    ref: local/aaa@bbb\n    \
                 verifier: t@000000000000\n    via: verifier\n";
    for (what, n) in [("fold-verifier-30", 30), ("fold-verifier-10", 10)] {
        let r = Repo::new(what);
        let ids: Vec<String> = (0..n)
            .map(|k| {
                r.task(0x900 + k, "human:marie", "done", proof);
                task_id(0x900 + k)
            })
            .collect();
        r.commit();
        let said = r.check(&["--all"]);
        let lines = per_entity(&said, "verifier 't' changed since the proof");
        if n == 30 {
            assert_eq!(lines, 0, "{said}");
            assert_folded(&said, "verifier-changed", 30, &ids);
        } else {
            assert_eq!(lines, 10, "{said}");
            assert!(folded(&said, "verifier-changed").is_empty(), "{said}");
        }
    }
}

#[test]
fn thirty_dead_scopes_are_one_corpus_signal() {
    for (what, n) in [("fold-dead-30", 30), ("fold-dead-10", 10)] {
        let r = Repo::new(what);
        let ids: Vec<String> = (0..n)
            .map(|k| {
                r.ignored_adr(0x900 + k);
                adr_id(0x900 + k)
            })
            .collect();
        r.commit();
        let said = r.check(&[]);
        let lines = per_entity(&said, "dead scope");
        if n == 30 {
            assert_eq!(lines, 0, "{said}");
            assert_folded(&said, "dead-scope", 30, &ids);
        } else {
            assert_eq!(lines, 10, "{said}");
            assert!(folded(&said, "dead-scope").is_empty(), "{said}");
        }
    }
}

#[test]
fn thirty_faults_of_one_kind_are_printed_one_by_one() {
    let r = Repo::new("fold-faults");
    for n in 0..30 {
        r.blocked_by_nothing(0x900 + n);
    }
    r.commit();
    let out = r.ank(&["check"]);
    assert_eq!(code(&out), 8, "{}", stderr(&out));
    let said = stdout(&out);
    let faults = said
        .lines()
        .filter(|l| l.starts_with("error: TASK-") && l.contains("which does not exist"))
        .count();
    assert_eq!(faults, 30, "a fault is never folded:\n{said}");
}

/// The value under `"<key>":` in a flat document, as a number.
fn num(json: &str, key: &str) -> usize {
    let at = json.find(&format!("\"{key}\":")).expect(key) + key.len() + 3;
    json[at..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .unwrap()
}

#[test]
fn json_carries_a_kind_on_every_finding_and_every_reader_counts_a_fold_once() {
    let (r, _) = agent_corpus("fold-json", 30);
    let out = r.ank(&["check", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json = stdout(&out);
    let findings = json.matches("{\"level\":").count();
    assert!(findings > 0, "{json}");
    assert_eq!(
        json.matches("\"kind\":\"").count(),
        findings,
        "kind on every finding:\n{json}"
    );
    assert!(
        json.contains("\"kind\":\"read-by-no-human\",\"subject\":\"corpus\""),
        "{json}"
    );
    // The folded signal is one finding, and the count says so.
    let signals = num(&json, "signals");
    assert_eq!(signals, json.matches("{\"level\":\"signal\"").count());
    assert!(signals < 30, "the thirty are one signal:\n{json}");

    let out = r.ank(&["review", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(num(&stdout(&out), "signals"), signals, "{}", stdout(&out));

    let out = r.ank(&["status", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(num(&stdout(&out), "signals"), signals, "{}", stdout(&out));
}

#[test]
fn a_kind_check_does_not_know_is_refused_naming_the_kinds() {
    let (r, _) = agent_corpus("fold-unknown-kind", 1);
    let out = r.ank(&["check", "--kind", "unread"]);
    assert_eq!(code(&out), 7, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stderr(&out).contains("read-by-no-human"),
        "{}",
        stderr(&out)
    );
}
