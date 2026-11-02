//! A peer binds every worktree of the reader, compared by identity and never by
//! path (TASK-d1996aa9851d, ADR-621a7fd96ce1).
//!
//! A binding across corpora is resolved at both ends: `web` reads `api` through
//! its own declaration `peers.api`, and `api`'s scope entry `web:src/**` is
//! resolved through `api`'s declaration `peers.web` back to the reader
//! (`Peer::binds`). That second resolution used to end in a comparison of
//! paths: the directory `../web` names against the reader's root. A linked
//! worktree of `web` has a root of its own, so the rule was dropped there with
//! no warning at all, and an override pointing `web` at the worktree only moved
//! the hole to the main checkout.
//!
//! **Through the binary, over three repositories**: `api` declares `web` and
//! `other`, `web` declares `api`, and `third` exists so that `other` resolves to
//! a readable corpus that is not `web` — the comparison by identity must still
//! tell two corpora apart.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-worktree";

/// In `api`, scoped `web:src/**`: binds every worktree of `web`.
const BOUND: &str = "ADR-aaaabbbbcccc";
/// In `api`, scoped `other:src/**`, where `other` names `third`: binds nobody
/// in `web`.
const ELSEWHERE: &str = "ADR-ddddeeeeffff";

/// A reader with a home of their own, so that the absence of an override is a
/// fact about this test and not about the machine running it.
struct Reader {
    home: PathBuf,
}

impl Reader {
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

    /// Standard output, and every line either stream carried.
    fn ok(&self, dir: &Path, args: &[&str]) -> (String, String) {
        let out = self.run(dir, args);
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(out.status.success(), "ank {args:?}: {stderr}");
        (stdout.clone(), format!("{stdout}{stderr}"))
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
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
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn adr(repo: &Path, id: &str, glob: &str) {
    fs::write(
        repo.join(format!(".ank/entities/{id}.md")),
        format!(
            "---\nid: {id}\ntype: adr\nslug: s-{id}\ntitle: Scoped {glob}\n\
             created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - '{glob}'\n\
             constraint: |\n  A rule.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
        ),
    )
    .unwrap();
}

/// A corpus with one commit: a peer is read from its default branch
/// (ADR-09b9), and a worktree needs a commit to check out.
fn corpus(reader: &Reader, dir: &Path, peers: &[(&str, &str)], adrs: &[(&str, &str)]) {
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src").join("x.rs"), "").unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    reader.ok(dir, &["init"]);
    // Named, so that any warning left in the answer is about a peer.
    reader.ok(dir, &["config", "default_branch", "main"]);
    for (name, path) in peers {
        reader.ok(dir, &["config", &format!("peers.{name}"), path]);
    }
    for (id, glob) in adrs {
        adr(dir, id, glob);
    }
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
            "init",
        ],
    );
}

fn warnings(all: &str) -> Vec<&str> {
    all.lines().filter(|l| l.starts_with("warning:")).collect()
}

#[test]
fn a_peer_binds_the_main_checkout_and_a_linked_worktree_alike() {
    let base = scratch::dir("peer-worktree");
    let reader = Reader {
        home: base.join("home"),
    };
    let api = base.join("api");
    let web = base.join("web");
    let third = base.join("third");
    let linked = base.join("web-linked");
    corpus(&reader, &third, &[], &[]);
    corpus(
        &reader,
        &api,
        &[("web", "../web"), ("other", "../third")],
        &[(BOUND, "web:src/**"), (ELSEWHERE, "other:src/**")],
    );
    corpus(&reader, &web, &[("api", "../api")], &[]);
    // A sibling of `web`, so `web`'s own `../api` resolves from it too: what is
    // under test is the back-resolution, and only that.
    git(
        &web,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            "linked",
            &linked.to_string_lossy(),
        ],
    );

    let bound = format!("{BOUND}@api");
    for (what, dir) in [("main checkout", &web), ("linked worktree", &linked)] {
        let (out, all) = reader.ok(dir, &["context", "src/"]);
        assert!(out.contains(&bound), "{what} lists {bound}:\n{all}");
        assert!(
            !out.contains(&ELSEWHERE[..8]),
            "{what}: a peer name resolving to a third repository binds nobody here:\n{all}"
        );
        assert!(warnings(&all).is_empty(), "{what}:\n{all}");
    }

    // No override was written, and none was needed.
    assert!(
        !reader.home.join("ank").join("corpora.yml").exists(),
        "an override was written"
    );

    // The third repository is still bound by its own rule, so the entry that
    // binds nobody in `web` is a live binding and not a dead one.
    reader.ok(&third, &["config", "peers.api", "../api"]);
    let (out, all) = reader.ok(&third, &["context", "src/"]);
    assert!(out.contains(&format!("{ELSEWHERE}@api")), "{all}");
    assert!(!out.contains(&BOUND[..8]), "{all}");

    let _ = fs::remove_dir_all(&base);
}
