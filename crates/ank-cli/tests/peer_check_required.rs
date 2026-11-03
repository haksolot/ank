//! `check --peers` makes a declared peer that cannot be read a fault
//! (TASK-35af3b803257, ADR-376bed7fa39a).
//!
//! A laptop without the sibling checkout is not wrong: there, a peer that
//! cannot be read is a signal and `check` exits 0. CI is the place that
//! expects the sibling, and `--peers` is how it says so: the same peer becomes
//! a fault at exit 8, naming the peer, its path and the override that settles
//! a layout that differs. Requiring the peer to be readable is not requiring
//! it to have merged (ADR-09b985a5627e): an edge naming a peer task its
//! default branch does not hold yet stays what plain `check` says it is.
//!
//! **Through the binary, in a scratch pair `api`/`web`** with both
//! declarations, and a reader whose home is the test's own so that no
//! override on the machine running it takes part.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-check-required";
const KIND: &str = "peer-unreadable";

struct Pair {
    base: PathBuf,
    home: PathBuf,
    api: PathBuf,
    web: PathBuf,
}

impl Pair {
    fn run(&self, dir: &Path, args: &[&str]) -> Output {
        let root = scratch::root();
        Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            .env("XDG_CONFIG_HOME", &self.home)
            .env("APPDATA", &self.home)
            .output()
            .expect("the binary runs")
    }

    fn ok(&self, dir: &Path, args: &[&str]) -> String {
        let out = self.run(dir, args);
        assert!(
            out.status.success(),
            "ank {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// `check` in `web` with `args` added: the exit code, the page, and the
    /// lines of the kind under test.
    fn check(&self, args: &[&str]) -> (Option<i32>, String, Vec<String>) {
        let mut all = vec!["check"];
        all.extend_from_slice(args);
        let out = self.run(&self.web, &all);
        let page = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let mut kinded = all.clone();
        kinded.extend_from_slice(&["--kind", KIND]);
        let lines = String::from_utf8_lossy(&self.run(&self.web, &kinded).stdout)
            .lines()
            .filter(|l| l.contains("peer 'api'"))
            .map(str::to_string)
            .collect();
        (out.status.code(), page, lines)
    }

    /// The levels `--json` gives the findings of the kind under test.
    fn levels(&self, args: &[&str]) -> Vec<String> {
        let mut all = vec!["check", "--json"];
        all.extend_from_slice(args);
        let json = String::from_utf8_lossy(&self.run(&self.web, &all).stdout).to_string();
        // A finding opens with its level and then its kind, the order of the
        // contract's own field list.
        ["signal", "fault"]
            .iter()
            .flat_map(|level| {
                let tag = format!("\"level\":\"{level}\",\"kind\":\"{KIND}\"");
                std::iter::repeat_n(level.to_string(), json.matches(&tag).count())
            })
            .collect()
    }
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn commit(dir: &Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            message,
        ],
    );
}

/// `api` and `web`, each declaring the other and each with one commit, since
/// a peer is read at its default branch.
fn pair(name: &str) -> Pair {
    let base = scratch::dir(name);
    let p = Pair {
        home: base.join("home"),
        api: base.join("api"),
        web: base.join("web"),
        base,
    };
    for (dir, peer, path) in [(&p.api, "web", "../web"), (&p.web, "api", "../api")] {
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("src").join("x.rs"), "").unwrap();
        git(dir, &["init", "-q", "-b", "main"]);
        p.ok(dir, &["init"]);
        p.ok(dir, &["config", "default_branch", "main"]);
        p.ok(dir, &["config", &format!("peers.{peer}"), path]);
        commit(dir, "init");
    }
    p
}

#[test]
fn an_unreadable_peer_is_a_signal_and_a_fault_under_peers() {
    let p = pair("peer-check-required");

    // Present: --peers changes nothing, the exit code included.
    let (plain, plain_page, plain_lines) = p.check(&[]);
    let (peers, peers_page, peers_lines) = p.check(&["--peers"]);
    assert_eq!(plain, Some(0), "check with api present:\n{plain_page}");
    assert_eq!(
        peers, plain,
        "check --peers exits as check does:\n{peers_page}"
    );
    assert!(plain_lines.is_empty() && peers_lines.is_empty());
    assert_eq!(
        plain_page.replace("--peers", ""),
        peers_page.replace("--peers", ""),
        "the same findings with api present"
    );

    // Moved away: a signal at exit 0, and a fault at exit 8 under --peers.
    let gone = p.base.join("api.gone");
    fs::rename(&p.api, &gone).unwrap();
    let (code, page, lines) = p.check(&[]);
    assert_eq!(code, Some(0), "a missing sibling is no fault:\n{page}");
    assert_eq!(lines.len(), 1, "one signal naming api: {page}");
    assert!(lines[0].starts_with("signal:"), "{}", lines[0]);
    assert_eq!(p.levels(&[]), ["signal"]);

    let (code, page, lines) = p.check(&["--peers"]);
    assert_eq!(code, Some(8), "check --peers faults:\n{page}");
    assert_eq!(lines.len(), 1, "one fault naming api: {page}");
    let line = &lines[0];
    assert!(line.starts_with("error:"), "a fault: {line}");
    assert!(line.contains("peer 'api'"), "names the peer: {line}");
    assert!(line.contains("../api"), "names its path: {line}");
    assert!(
        line.contains("ank config --user peers.") && line.contains(".api <path>"),
        "names the override: {line}"
    );
    assert_eq!(p.levels(&["--peers"]), ["fault"]);

    let _ = fs::remove_dir_all(&p.base);
}

#[test]
fn a_peer_task_not_yet_on_the_default_branch_stays_a_signal_under_peers() {
    let p = pair("peer-check-required-unmerged");

    // A task api holds on main, named by web, then left only on api's branch
    // `work`: the edge web wrote before api's default branch moved.
    let out = p.ok(
        &p.api,
        &[
            "new",
            "task",
            "--title",
            "Peer work",
            "--scope",
            "src/**",
            "--criteria",
            "It lands.",
            "--no-verify",
            "--json",
        ],
    );
    let at = out.find("\"TASK-").expect("new names the task") + 1;
    let peer_task = out[at..at + 17].to_string();
    commit(&p.api, "peer work");
    p.ok(
        &p.web,
        &[
            "new",
            "task",
            "--title",
            "Waits",
            "--scope",
            "src/**",
            "--criteria",
            "It waits.",
            "--no-verify",
            "--blocked-by",
            &format!("{peer_task}@api"),
        ],
    );
    commit(&p.web, "waits");
    git(&p.api, &["branch", "work"]);
    git(&p.api, &["reset", "-q", "--hard", "HEAD~1"]);

    let (plain, plain_page, _) = p.check(&[]);
    let (peers, peers_page, lines) = p.check(&["--peers"]);
    assert_eq!(plain, Some(0), "check:\n{plain_page}");
    assert_eq!(peers, Some(0), "check --peers:\n{peers_page}");
    assert!(lines.is_empty(), "api is readable: {lines:?}");
    let edge: Vec<&str> = peers_page
        .lines()
        .filter(|l| l.contains(&format!("{peer_task}@api")))
        .collect();
    assert_eq!(edge.len(), 1, "the edge is reported once:\n{peers_page}");
    assert!(edge[0].starts_with("signal:"), "as a signal: {}", edge[0]);

    let _ = fs::remove_dir_all(&p.base);
}
