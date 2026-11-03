//! A peer constraint reaches the executor, shown and never frozen
//! (ADR-50952ba5e226, TASK-d84e0098dfd4).
//!
//! Measured before the change at e04c0dad: in `web`, `ank context src/` lists
//! the `api` ADR scoped `web:src/**` with no claim, and once a task scoped
//! `src/**` is claimed the same command serves the local ADR only. The agent
//! writing the code a cross-repository rule governs was the one never told.
//!
//! **Through the binary, over a pair** `api` / `web`, each declaring the other.
//! Every verb run in `web` is bracketed by a byte-for-byte snapshot of `api`,
//! `.git` included: serving a peer's rule writes nothing over there.

use crate::scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-execution";

/// The ADR of `api` binding `web:src/**`.
const RULE: &str = "ADR-a1a1a1a1a1a1";
/// The ADR of `api` that supersedes [`RULE`] while the claim is held.
const NEXT: &str = "ADR-b2b2b2b2b2b2";
/// A local ADR of `web`, the rule the freeze covers.
const LOCAL: &str = "ADR-e4e4e4e4e4e4";

struct Pair {
    base: PathBuf,
    api: PathBuf,
    web: PathBuf,
}

struct Answer {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

impl Pair {
    fn run(&self, dir: &Path, args: &[&str]) -> Answer {
        let root = scratch::root();
        let home = self.base.join("home");
        let out = Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", &home)
            .env("APPDATA", &home)
            .output()
            .expect("the binary runs");
        Answer {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        }
    }

    /// A verb run in `web`, asserting nothing under `api` moved.
    fn web(&self, args: &[&str]) -> Answer {
        let before = snapshot(&self.api);
        let answer = self.run(&self.web, args);
        let after = snapshot(&self.api);
        assert!(
            before == after,
            "ank {args:?} in web modified api: {:?}",
            changed(&before, &after)
        );
        answer
    }

    fn web_ok(&self, args: &[&str]) -> Answer {
        let a = self.web(args);
        assert_eq!(a.code, Some(0), "ank {args:?}: {}{}", a.stdout, a.stderr);
        a
    }

    /// The claim record `web` holds on `id`, as git stores it.
    fn record(&self, id: &str) -> String {
        git_out(
            &self.web,
            &["cat-file", "-p", &format!("refs/ank/claims/{id}")],
        )
    }
}

fn git_out(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.email=t@example.invalid", "-c", "user.name=t"])
        .args(["-c", "commit.gpgsign=false"])
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

fn commit(dir: &Path, message: &str) {
    git_out(dir, &["add", "-A"]);
    git_out(dir, &["commit", "-qm", message]);
}

fn adr(dir: &Path, id: &str, status: &str, glob: &str, rule: &str, supersedes: Option<&str>) {
    let supersedes = supersedes
        .map(|s| format!("supersedes: {s}\n"))
        .unwrap_or_default();
    fs::write(
        dir.join(format!(".ank/entities/{id}.md")),
        format!(
            "---\nid: {id}\ntype: adr\nslug: s-{id}\ntitle: Rule {id}\n\
             created: 2026-08-01T00:00:00Z\nstatus: {status}\nscope:\n  - '{glob}'\n\
             constraint: |\n  {rule}\n{supersedes}schema: 1\nversion: 1\n---\n\nWhy.\n"
        ),
    )
    .unwrap();
}

fn pair(name: &str) -> Pair {
    let base = scratch::dir(name);
    let p = Pair {
        api: base.join("api"),
        web: base.join("web"),
        base,
    };
    for (dir, peer, path) in [(&p.api, "web", "../web"), (&p.web, "api", "../api")] {
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src/lib.rs"), "\n").unwrap();
        git_out(dir, &["init", "-q", "-b", "main"]);
        // No background maintenance: its lock file under `.git` would read as
        // a write of ank's in the snapshot of `api`.
        git_out(dir, &["config", "maintenance.auto", "false"]);
        git_out(dir, &["config", "gc.auto", "0"]);
        for args in [
            vec!["init"],
            vec!["config", "default_branch", "main"],
            vec!["config", &format!("peers.{peer}"), path],
        ] {
            let a = p.run(dir, &args);
            assert_eq!(a.code, Some(0), "ank {args:?}: {}", a.stderr);
        }
    }
    adr(
        &p.web,
        LOCAL,
        "accepted",
        "src/**",
        "A local rule about src.",
        None,
    );
    adr(
        &p.api,
        RULE,
        "accepted",
        "web:src/**",
        "The web client speaks version two of the api.",
        None,
    );
    commit(&p.web, "init");
    commit(&p.api, "init");
    p
}

fn snapshot(dir: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(
                    p.strip_prefix(dir).unwrap().to_path_buf(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    out
}

fn changed(a: &BTreeMap<PathBuf, Vec<u8>>, b: &BTreeMap<PathBuf, Vec<u8>>) -> Vec<PathBuf> {
    a.keys()
        .chain(b.keys())
        .filter(|k| a.get(*k) != b.get(*k))
        .cloned()
        .collect()
}

/// The value of one top-level field of a claim record.
fn field<'a>(record: &'a str, key: &str) -> &'a str {
    record
        .lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .unwrap_or_else(|| panic!("no {key} in the record:\n{record}"))
}

/// The page from the peer section's header to the next blank line.
fn peer_section<'a>(page: &'a str, peer: &str) -> &'a str {
    let header = format!("PEER CONSTRAINTS ({peer} at ");
    let at = page
        .find(&header)
        .unwrap_or_else(|| panic!("no section for {peer}:\n{page}"));
    let rest = &page[at..];
    let end = rest.find("\n\n").unwrap_or(rest.len());
    &rest[..end]
}

#[test]
fn under_a_claim_the_peer_rule_is_shown_recorded_never_frozen_and_done_names_its_succession() {
    let p = pair("peer-execution");
    // A verifier that always passes: the close is what the test reads, and its
    // exit code is the one a close with no peer at all would have.
    p.web_ok(&["config", "verifiers.ok.run", "true"]);
    let created = p.web_ok(&[
        "new",
        "task",
        "--title",
        "Local work under the api rule",
        "--scope",
        "src/**",
        "--criteria",
        "It holds.",
        "--verify",
        "ok",
        "--json",
    ]);
    let at = created
        .stdout
        .find("\"TASK-")
        .expect("the new task is named")
        + 1;
    let id = created.stdout[at..at + 17].to_string();
    commit(&p.web, "task");
    let api_head = git_out(&p.api, &["rev-parse", "main"]);
    let short = &api_head[..12];
    let peer_rule = format!("{RULE}@api");

    // The freeze with the peer undeclared: the reference the hash is held to.
    p.web_ok(&["config", "--unset", "peers.api"]);
    p.web_ok(&["claim", &id]);
    let alone = p.record(&id);
    assert!(
        !alone.contains(RULE),
        "undeclared, api is not read:\n{alone}"
    );
    p.web_ok(&["release", "--reason", "measure the freeze with the peer"]);
    git_out(&p.web, &["checkout", "--", ".ank/config.yml"]);

    // Claimed with the peer declared: shown, in a section of its own.
    let claimed = p.web_ok(&["claim", &id]);
    let section = peer_section(&claimed.stdout, "api");
    assert!(
        section.contains(short),
        "the commit read is named:\n{section}"
    );
    assert!(section.contains(&peer_rule), "{section}");
    assert!(
        section.contains("The web client speaks version two of the api."),
        "served in full, as a local rule is under a claim:\n{section}"
    );

    // Recorded, and never frozen.
    let record = p.record(&id);
    assert_eq!(
        field(&record, "constraints"),
        field(&alone, "constraints"),
        "the frozen hash is the local rules' alone:\n{alone}\n---\n{record}"
    );
    assert!(record.contains(&api_head), "the peer commit:\n{record}");
    assert!(record.contains(RULE), "the peer rule's hash:\n{record}");

    // context serves the same section, after the local rules.
    let context = p.web_ok(&["context"]).stdout;
    let section = peer_section(&context, "api");
    assert!(
        section.contains(short) && section.contains(&peer_rule),
        "{context}"
    );
    assert!(
        context.find(LOCAL) < context.find("PEER CONSTRAINTS"),
        "{context}"
    );

    // And `--json` under a key of its own.
    let json: serde_yaml::Value =
        serde_yaml::from_str(&p.web_ok(&["context", "--json"]).stdout).unwrap();
    let peers = json["peer_constraints"]
        .as_sequence()
        .unwrap_or_else(|| panic!("no peer_constraints: {json:?}"));
    assert_eq!(peers.len(), 1, "{json:?}");
    assert_eq!(peers[0]["peer"].as_str(), Some("api"));
    assert_eq!(peers[0]["commit"].as_str(), Some(api_head.as_str()));
    assert_eq!(
        peers[0]["constraints"][0]["short"].as_str(),
        Some(peer_rule.as_str())
    );
    // The document is the one the contract table declares for this case, key
    // for key and in order, and its rows carry the declared keys.
    let help: serde_yaml::Value =
        serde_yaml::from_str(&p.web_ok(&["help", "context", "--json"]).stdout).unwrap();
    let declared: Vec<&str> = help["verbs"][0]["returns"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|r| r["when"].as_str() == Some("under a claim a declared peer binds"))
        .expect("the contract declares the document")["fields"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .collect();
    let keys = |v: &serde_yaml::Value| -> Vec<String> {
        v.as_mapping()
            .unwrap()
            .keys()
            .filter_map(|k| k.as_str().map(str::to_string))
            .collect()
    };
    // Nested fields are declared as dotted names: the top level, and the whole
    // of the new key down to its rows.
    let top: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|n| !n.contains('.'))
        .collect();
    assert_eq!(keys(&json), top, "{json:?}");
    let mut nested = vec!["peer_constraints".to_string()];
    for k in keys(&peers[0]) {
        nested.push(format!("peer_constraints.{k}"));
        if k == "constraints" {
            for c in keys(&peers[0]["constraints"][0]) {
                nested.push(format!("peer_constraints.constraints.{c}"));
            }
        }
    }
    let declared_nested: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|n| n.starts_with("peer_constraints"))
        .collect();
    assert_eq!(nested, declared_nested);
    let ids: Vec<&str> = json["constraints"]
        .as_sequence()
        .unwrap()
        .iter()
        .filter_map(|c| c["id"].as_str())
        .collect();
    assert_eq!(ids, vec![LOCAL], "the local section is unchanged: {json:?}");

    // The rule is superseded on api's default branch while the claim is held.
    adr(
        &p.api,
        RULE,
        "superseded",
        "web:src/**",
        "The web client speaks version two of the api.",
        None,
    );
    adr(
        &p.api,
        NEXT,
        "accepted",
        "web:src/**",
        "The web client speaks version three of the api.",
        Some(RULE),
    );
    commit(&p.api, "supersede");

    let done = p.web(&["done"]);
    assert_eq!(
        done.code,
        Some(0),
        "never refused on a peer: {}{}",
        done.stdout,
        done.stderr
    );
    let line = done
        .stderr
        .lines()
        .find(|l| l.contains(&peer_rule))
        .unwrap_or_else(|| panic!("no line names the retired rule:\n{}", done.stderr));
    assert!(
        line.contains(&format!("{NEXT}@api")),
        "one line names both the retired and the new rule: {line}"
    );

    let _ = fs::remove_dir_all(&p.base);
}

#[test]
fn a_claim_no_peer_rule_reaches_carries_no_peer_section_and_no_peer_record() {
    let p = pair("peer-execution-elsewhere");
    let created = p.web_ok(&[
        "new",
        "task",
        "--title",
        "Work outside the api rule",
        "--scope",
        "docs/**",
        "--criteria",
        "It holds.",
        "--no-verify",
        "--json",
    ]);
    let at = created
        .stdout
        .find("\"TASK-")
        .expect("the new task is named")
        + 1;
    let id = created.stdout[at..at + 17].to_string();
    commit(&p.web, "task");

    let claimed = p.web_ok(&["claim", &id]);
    assert!(!claimed.stdout.contains("PEER"), "{}", claimed.stdout);
    let record = p.record(&id);
    assert!(!record.contains("peers:"), "{record}");
    let context = p.web_ok(&["context"]).stdout;
    assert!(!context.contains("PEER"), "{context}");
    let json = p.web_ok(&["context", "--json"]).stdout;
    assert!(!json.contains("peer_constraints"), "{json}");

    let _ = fs::remove_dir_all(&p.base);
}
