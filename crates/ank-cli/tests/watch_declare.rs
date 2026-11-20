//! `ank init` and `ank watch` declare the checkout they run in, and a running
//! watcher re-reads `watch.yml` (ADR-f6b8e6539335), driven through the binary.
//!
//! **Every process here runs with `ANK_WATCH_DECLARE` removed**, because the
//! repository's `.cargo/config.toml` sets it to `off` for the whole suite: the
//! hundreds of `ank init` the other files run must never reach a reader's real
//! `watch.yml`. The first test measures that the setting reaches this process,
//! so a suite that stopped inheriting it fails here rather than in somebody's
//! home directory.

use crate::index_file;
use crate::scratch;
use ank_contract::env::ANK_WATCH_DECLARE;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

fn ank_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ank"))
}

/// A reader's configuration home, both platform rules pointed at it.
struct Home(PathBuf);

impl Home {
    fn new() -> Home {
        let dir = scratch::dir("declare-home");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("gitconfig"), "[commit]\n\tgpgsign = false\n").unwrap();
        Home(dir)
    }

    fn watch_file(&self) -> PathBuf {
        self.0.join("ank").join("watch.yml")
    }

    fn watch_text(&self) -> String {
        std::fs::read_to_string(self.watch_file()).unwrap_or_default()
    }

    /// The environment of a reader who has not opted out.
    fn apply(&self, cmd: &mut Command) {
        let config = self.0.join("gitconfig");
        cmd.env("GIT_CONFIG_GLOBAL", &config)
            .env("GIT_CONFIG_SYSTEM", &config)
            .env("XDG_CONFIG_HOME", &self.0)
            .env("APPDATA", &self.0)
            .env("HOME", &self.0)
            .env("ANK_AGENT", "test@ank.local")
            .env_remove(ANK_WATCH_DECLARE);
    }

    fn ank(&self, cwd: &Path, args: &[&str]) -> Output {
        let mut c = Command::new(ank_bin());
        c.args(args).current_dir(cwd);
        self.apply(&mut c);
        c.output().expect("ank must have been built")
    }

    fn git(&self, cwd: &Path, args: &[&str]) {
        let mut c = Command::new("git");
        c.args(args).current_dir(cwd);
        self.apply(&mut c);
        let out = c.output().expect("git is a hard dependency");
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    /// A repository with one commit and no corpus yet.
    ///
    /// Its identity is its root commit, so `what` goes into the message: two
    /// repositories with the same tree, author and message committed in the
    /// same second share one hash, and the declaration then sees one checkout
    /// identity with two paths.
    fn repository(&self, what: &str) -> PathBuf {
        let root = scratch::dir(what).join("tree");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        self.git(&root, &["init", "-q", "-b", "main"]);
        self.git(&root, &["config", "user.email", "test@ank.local"]);
        self.git(&root, &["config", "user.name", "Test"]);
        self.git(&root, &["config", "gc.auto", "0"]);
        self.git(&root, &["config", "maintenance.auto", "false"]);
        self.git(&root, &["add", "-A"]);
        self.git(&root, &["commit", "-qm", &format!("first {what}")]);
        root
    }

    /// A repository carrying a corpus, initialised with the declaration off so
    /// that whatever declares it later is what the test is about.
    fn undeclared_corpus(&self, what: &str) -> PathBuf {
        let root = self.repository(what);
        let mut c = Command::new(ank_bin());
        c.arg("init").current_dir(&root);
        self.apply(&mut c);
        c.env(ANK_WATCH_DECLARE, "off");
        let out = c.output().unwrap();
        assert!(out.status.success(), "{out:?}");
        assert!(!self.watch_file().exists(), "off wrote a declaration");
        root
    }

    fn identity(&self, root: &Path) -> String {
        let out = Command::new("git")
            .args(["rev-list", "--max-parents=0", "HEAD"])
            .current_dir(root)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }
}

fn index(root: &Path) -> PathBuf {
    index_file::path(&root.join(".ank"))
}

fn until(what: &str, mut done: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        if done() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for {what}");
}

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Measured rather than assumed: the opt-out the rest of the suite relies on
/// reaches the test process, and so every `ank` it spawns.
#[test]
fn the_suite_runs_with_the_declaration_off() {
    assert_eq!(
        std::env::var(ANK_WATCH_DECLARE).as_deref(),
        Ok("off"),
        ".cargo/config.toml no longer reaches the suite, and every ank init in it \
         would write into the real reader's watch.yml"
    );
}

#[test]
fn init_declares_a_repository_with_a_commit_and_creates_watch_yml() {
    let home = Home::new();
    let root = home.repository("init-declares");
    let out = home.ank(&root, &["init"]);
    assert!(out.status.success(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(said.contains("so ank watch keeps it warm"), "{said}");

    let listed = home.ank(&scratch::dir("elsewhere"), &["watch", "--list"]);
    assert!(listed.status.success(), "{listed:?}");
    let text = String::from_utf8_lossy(&listed.stdout);
    assert!(text.contains(&home.identity(&root)), "{text}");
    assert!(text.contains("watching 1 corpus, 1 checkout"), "{text}");

    // Again, and nothing is added twice.
    let before = home.watch_text();
    let out = home.ank(&root, &["init"]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(home.watch_text(), before);
}

#[test]
fn init_with_no_commit_declares_nothing_and_still_succeeds() {
    let home = Home::new();
    let root = scratch::dir("no-commit").join("tree");
    std::fs::create_dir_all(&root).unwrap();
    home.git(&root, &["init", "-q", "-b", "main"]);
    let out = home.ank(&root, &["init"]);
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(said.contains("no commit yet"), "{said}");
    assert!(!home.watch_file().exists());
}

#[test]
fn init_with_nowhere_to_declare_still_succeeds_and_says_so() {
    let home = Home::new();
    let root = home.repository("no-home");
    let mut c = Command::new(ank_bin());
    c.arg("init").current_dir(&root);
    home.apply(&mut c);
    c.env_remove("APPDATA")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("HOME");
    let out = c.output().unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(said.contains("not declared for ank watch"), "{said}");
}

#[test]
fn off_declares_nothing_from_init_or_from_watch() {
    let home = Home::new();
    let root = home.undeclared_corpus("off");
    let mut c = Command::new(ank_bin());
    c.args(["watch", "--once"]).current_dir(&root);
    home.apply(&mut c);
    c.env(ANK_WATCH_DECLARE, "off");
    let out = c.output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    assert!(!home.watch_file().exists());
}

#[test]
fn watch_started_in_an_undeclared_checkout_declares_it_and_warms_it() {
    let home = Home::new();
    let other = home.undeclared_corpus("other");
    let root = home.undeclared_corpus("started-here");
    // A declaration the reader wrote, with a comment of theirs.
    std::fs::create_dir_all(home.watch_file().parent().unwrap()).unwrap();
    let mine = format!(
        "# mine, keep this\nschema: 1\nwatch:\n  {}: {}\n",
        home.identity(&other),
        other.display()
    );
    std::fs::write(home.watch_file(), &mine).unwrap();
    index_file::remove_all(&root.join(".ank"));

    // From a subdirectory: the walk up every verb performs finds the checkout.
    let out = home.ank(&root.join("src"), &["watch", "--once"]);
    assert!(out.status.success(), "{out:?}");
    let log = String::from_utf8_lossy(&out.stderr);
    assert!(
        log.contains(&format!("declared {}", home.identity(&root))),
        "{log}"
    );
    assert!(
        index(&root).is_file(),
        "the checkout it declared was not warmed"
    );

    let text = home.watch_text();
    assert!(
        text.starts_with(&mine),
        "the reader's file was not kept:\n{text}"
    );
    let listed = home.ank(&scratch::dir("elsewhere"), &["watch", "--list"]);
    let listed = String::from_utf8_lossy(&listed.stdout);
    assert!(
        listed.contains("watching 2 corpora, 2 checkouts"),
        "{listed}"
    );
}

#[test]
fn watch_list_in_an_undeclared_checkout_writes_nothing() {
    let home = Home::new();
    let root = home.undeclared_corpus("list-only");
    let out = home.ank(&root, &["watch", "--list"]);
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    assert!(!home.watch_file().exists());
}

#[test]
fn a_corpus_declared_while_the_watcher_runs_is_warmed_without_a_restart() {
    let home = Home::new();
    let first = home.undeclared_corpus("running-first");
    let second = home.undeclared_corpus("running-second");
    std::fs::create_dir_all(home.watch_file().parent().unwrap()).unwrap();
    std::fs::write(
        home.watch_file(),
        format!(
            "schema: 1\nwatch:\n  {}: {}\n",
            home.identity(&first),
            first.display()
        ),
    )
    .unwrap();
    index_file::remove_all(&first.join(".ank"));
    index_file::remove_all(&second.join(".ank"));

    let mut c = Command::new(ank_bin());
    c.args(["watch", "--interval", "50"])
        .current_dir(scratch::dir("outside"))
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    home.apply(&mut c);
    let _running = Running(c.spawn().unwrap());
    until("the declared corpus to be warmed", || {
        index(&first).is_file()
    });
    assert!(!index(&second).exists(), "an undeclared corpus was read");

    // Declared by init, the way a reader adds one, while the watcher runs.
    let out = home.ank(&second, &["init"]);
    assert!(out.status.success(), "{out:?}");
    until("the corpus declared after start to be warmed", || {
        index(&second).is_file()
    });
}
