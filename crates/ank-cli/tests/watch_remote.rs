//! The watcher mirrors claims from the remote that carries them
//! (ADR-b6fe84504ff2, TASK-451aaa32043f).
//!
//! Through the binary, over bare remotes reached by `file://`, and counted with
//! `GIT_TRACE2_EVENT` at an absolute path (ADR-cc65f1388a71): one `start` per
//! process, written by git itself. Only the processes a non-git parent started
//! are counted -- a trace2 session id with no `/` -- so the `upload-pack` a
//! fetch over `file://` starts, and whatever git runs on its own behalf, are
//! left out: they are git's work and not the watcher's.
//!
//! The origin-only literal is what the binary before this change ran
//! (2b7c8957), measured in the same layout: the identity at startup, then a
//! cycle of `config`, `fetch`, `for-each-ref`, then the warming's own four.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const T: &str = "TASK-000000000001";
const FIRST: &str = "first@ank.local";
const SECOND: &str = "second@ank.local";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("watch-remote-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [protocol \"file\"]\n\tallow = always\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

/// A process with the isolated configuration and `home` as the reader's
/// configuration directory, on both platform rules at once.
fn spawn(program: &str, home: &Path) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    let root = scratch::root();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config)
        .env("XDG_CONFIG_HOME", home)
        .env("APPDATA", home)
        .env("HOME", home)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root);
    c
}

fn run(program: &str, home: &Path, dir: &Path, args: &[&str]) -> String {
    let out = spawn(program, home)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("{program}: {e}"));
    assert!(
        out.status.success(),
        "{program} {args:?} in {}: {}{}",
        dir.display(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

/// `file:///C:/...` on Windows, `file:///tmp/...` elsewhere.
fn file_url(path: &Path) -> String {
    let path = path.to_str().unwrap().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// The arguments after `git` of every process a non-git parent started.
fn calls(trace: &Path) -> Vec<String> {
    let text = fs::read_to_string(trace).unwrap_or_default();
    text.lines()
        .filter(|l| l.contains("\"event\":\"start\""))
        .filter(|l| {
            let sid = l.split("\"sid\":\"").nth(1).unwrap_or("");
            !sid.split('"').next().unwrap_or("").contains('/')
        })
        .map(|l| {
            let argv = l.split("\"argv\":[").nth(1).unwrap_or("");
            let argv = argv.split("\"]").next().unwrap_or("");
            argv.trim_start_matches('"')
                .split("\",\"")
                .skip(1)
                .collect::<Vec<_>>()
                .join(" ")
                .replace("\\\\", "\\")
        })
        .collect()
}

/// Two clones of one corpus, the second declared to a watcher: `first` claims,
/// `second` is where the claim has to arrive.
struct Layout {
    base: PathBuf,
    home: PathBuf,
    first: PathBuf,
    second: PathBuf,
}

impl Layout {
    /// Bare `origin`, and a bare `ank` beside it when `with_ank`; `first`
    /// claims `T`, which travels wherever the routing of ADR-b6fe84504ff2
    /// sends it, after `second` has run one `status` so that its refspecs are
    /// already what the repair makes them and no later verb fetches.
    fn new(what: &str, with_ank: bool) -> Layout {
        let base = scratch::dir(what);
        let home = base.join("home");
        fs::create_dir_all(home.join("ank")).unwrap();
        let first = base.join("first");
        let second = base.join("second");
        let git = |dir: &Path, args: &[&str]| run("git", &home, dir, args);

        let remotes: &[&str] = if with_ank {
            &["origin", "ank"]
        } else {
            &["origin"]
        };
        for name in remotes {
            let bare = base.join(format!("{name}.git"));
            fs::create_dir_all(&bare).unwrap();
            git(&bare, &["init", "-q", "--bare", "-b", "main"]);
        }

        fs::create_dir_all(first.join(".ank/entities")).unwrap();
        fs::create_dir_all(first.join("src")).unwrap();
        git(&first, &["init", "-q", "-b", "main"]);
        fs::write(
            first.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(first.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(first.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            first.join(".ank/entities").join(format!("{T}.md")),
            format!(
                "---\nid: {T}\ntype: task\nslug: example\ntitle: The watched task\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
        git(&first, &["add", "-A"]);
        git(&first, &["commit", "-qm", "seed"]);
        for name in remotes {
            let url = file_url(&base.join(format!("{name}.git")));
            git(&first, &["remote", "add", name, &url]);
            git(&first, &["push", "-q", name, "main"]);
        }

        let origin = file_url(&base.join("origin.git"));
        run(
            "git",
            &home,
            &base,
            &["clone", "-q", &origin, second.to_str().unwrap()],
        );
        if with_ank {
            git(
                &second,
                &["remote", "add", "ank", &file_url(&base.join("ank.git"))],
            );
        }
        let layout = Layout {
            base,
            home: home.clone(),
            first,
            second,
        };
        layout.ank(&layout.second, SECOND, &["status"]);
        layout.ank(&layout.first, FIRST, &["claim", T]);
        assert_eq!(
            layout.refs("refs/ank/claims/"),
            "",
            "the second clone holds no claim of its own before the watcher runs"
        );

        let identity = run(
            "git",
            &layout.home,
            &layout.second,
            &["rev-list", "--max-parents=0", "--reverse", "HEAD"],
        );
        fs::write(
            layout.home.join("ank/watch.yml"),
            format!(
                "schema: 1\nfetch: 1\nwatch:\n  {}: {}\n",
                identity.trim(),
                layout.second.display()
            ),
        )
        .unwrap();
        layout
    }

    fn ank(&self, dir: &Path, agent: &str, args: &[&str]) -> String {
        let out = spawn(ANK, &self.home)
            .args(args)
            .arg("--repo")
            .arg(dir)
            .env("ANK_AGENT", agent)
            .current_dir(scratch::root())
            .output()
            .expect("the binary must have been built");
        assert!(
            out.status.success(),
            "ank {args:?}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// The refs of the second clone under `prefix`, one name per line.
    fn refs(&self, prefix: &str) -> String {
        run(
            "git",
            &self.home,
            &self.second,
            &["for-each-ref", "--format=%(refname)", prefix],
        )
    }

    /// One `ank watch --once`, and the git processes it started.
    fn cycle(&self) -> Vec<String> {
        let trace = self.base.join("watch.trace");
        let _ = fs::remove_file(&trace);
        let out = spawn(ANK, &self.home)
            .args(["watch", "--once"])
            .env("ANK_AGENT", SECOND)
            .env("GIT_TRACE2_EVENT", &trace)
            .current_dir(scratch::root())
            .output()
            .expect("the binary must have been built");
        assert!(
            out.status.success(),
            "ank watch --once: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        calls(&trace)
    }
}

/// The warming's own processes, the same in every layout: `ank find` asking
/// git what it always asks.
const WARM: [&str; 4] = [
    "--version",
    "rev-parse --path-format=absolute --git-dir --git-common-dir --show-toplevel",
    "for-each-ref --format=%(refname)%09%(objectname) refs/ank/",
    "rev-list --max-parents=0 --reverse HEAD",
];

/// The startup: the identity the declaration is checked against, and the one
/// question this change adds -- which remote carries the claims, asked of the
/// resolver `crates/ank-cli/src/git.rs` routes every verb with, once per
/// checkout and never per cycle.
const STARTUP: [&str; 2] = [
    "rev-list --max-parents=0 --reverse HEAD",
    r"config --get-regexp ^remote\.(origin|ank)\.url$",
];

#[test]
fn with_ank_a_cycle_mirrors_claims_from_ank_and_fetches_nothing_from_origin() {
    let l = Layout::new("watch-remote-ank", true);
    let calls = l.cycle();

    let fetches: Vec<&String> = calls.iter().filter(|c| c.starts_with("fetch ")).collect();
    assert_eq!(
        fetches,
        [
            "fetch --quiet --prune --no-tags --no-write-fetch-head --no-recurse-submodules \
             --refmap= ank +refs/ank/claims/*:refs/ank/watch/ank/claims/*"
        ],
        "one fetch, from ank, into its own mirror: {calls:#?}"
    );
    assert!(
        !calls.iter().any(|c| c.split(' ').any(|a| a == "origin")),
        "the cycle named origin: {calls:#?}"
    );
    let mut expected: Vec<String> = STARTUP.iter().map(|s| s.to_string()).collect();
    expected.extend([
        "config --get remote.ank.url".to_string(),
        fetches[0].clone(),
        "for-each-ref --format=%(refname)%09%(objectname) refs/ank/watch/ank/".to_string(),
    ]);
    expected.extend(WARM.iter().map(|s| s.to_string()));
    assert_eq!(calls, expected);

    assert_eq!(
        l.refs("refs/ank/watch/"),
        format!("refs/ank/watch/ank/claims/{T}\n"),
        "the claim is mirrored under the remote that carries it, and nothing else"
    );
    assert_eq!(l.refs("refs/ank/claims/"), "", "the plane was not touched");

    let status = l.ank(&l.second, SECOND, &["status"]);
    let line = status
        .lines()
        .find(|line| line.contains(T))
        .unwrap_or_else(|| panic!("status lists no claim of {T}:\n{status}"));
    assert!(line.contains(FIRST), "{line}");
}

#[test]
fn without_ank_a_cycle_runs_what_it_ran_before() {
    let l = Layout::new("watch-remote-origin", false);
    let calls = l.cycle();

    // Measured on 2b7c8957, the binary before this change, in this layout.
    let before = [
        "config --get remote.origin.url",
        "fetch --quiet --prune --no-tags --no-write-fetch-head --no-recurse-submodules \
         --refmap= origin +refs/ank/claims/*:refs/ank/watch/origin/claims/*",
        "for-each-ref --format=%(refname)%09%(objectname) refs/ank/watch/origin/",
    ];
    assert_eq!(calls[..STARTUP.len()], STARTUP, "{calls:#?}");
    let cycle: Vec<&str> = calls[STARTUP.len()..].iter().map(String::as_str).collect();
    let mut expected = before.to_vec();
    expected.extend(WARM);
    assert_eq!(cycle, expected, "the cycle moved: {calls:#?}");

    assert_eq!(
        l.refs("refs/ank/watch/"),
        format!("refs/ank/watch/origin/claims/{T}\n")
    );
    let status = l.ank(&l.second, SECOND, &["status"]);
    assert!(
        status
            .lines()
            .any(|line| line.contains(T) && line.contains(FIRST)),
        "{status}"
    );
}
