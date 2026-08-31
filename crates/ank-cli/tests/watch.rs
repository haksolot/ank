//! `ank watch`, driven as a process.
//!
//! Driven rather than called, on the rule this repository learned twice: a
//! criterion that talks about the binary is tested through the binary, because
//! green unit tests have twice covered code that was right on a path the binary
//! never reached. Every claim ADR-24e21cb83793 makes is a claim about a running
//! process -- what it reads, what it leaves untouched, and what stopping it
//! changes -- and none of them can be asserted from inside a function.
//!
//! **It sits here rather than in `crates/ank-daemon/tests/`** for the
//! mechanical reason `tests/tui.rs` and `tests/mcp.rs` give: `CARGO_BIN_EXE_ank`
//! is defined only for the package that declares the binary, and the watcher
//! stopped declaring one when it became a verb (TASK-9dd22f2b0430). The suite
//! did not otherwise change shape -- it spawned a sibling executable and now
//! spawns `ank watch`, which is the whole of what the fold did.
//!
//! **Being started by a verb is not answering one**, and this suite is where
//! that is mechanical rather than argued. There is no assertion here that asks
//! a running watcher anything, because there is no way to: the four flags are
//! all read before the loop begins, and
//! `stopping_the_daemon_changes_no_verbs_output_and_no_verbs_exit_code` holds
//! the other half -- every verb answers identically with the process gone.
//!
//! Two of the assertions here are the two ways this is usually got wrong.
//! `nothing_walks_a_filesystem_looking_for_a_corpus` plants corpora above,
//! beside and below the declared one, and shows they stay unread: the friendly
//! implementation that scans is the one the corpus refused in writing, and this
//! is what makes it fail rather than ship. And
//! `a_warm_listing_equals_a_cold_listing_byte_for_byte` is how the cache stops
//! being trusted -- the moment a warm listing and a cold one differ, the daemon
//! has become a source of truth nobody voted for.

mod scratch;
use ank_contract::events;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use std::sync::OnceLock;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// The one door
// ---------------------------------------------------------------------------

/// git's global and system configuration, for every process this suite spawns.
///
/// The CLI's own suite learned this the expensive way: on a machine that signs
/// by default, fixtures depended on a gpg agent staying unlocked and failed the
/// moment pinentry timed out, while CI -- where nothing signs -- passed forever.
/// A fixture may not inherit what it did not declare.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("gitconfig");
        std::fs::write(&p, "[commit]\n\tgpgsign = false\n").unwrap();
        p
    })
    .as_path()
}

/// The binary, which is now the only one there is.
///
/// `CARGO_BIN_EXE_ank` and not a path built up beside something: the watcher
/// and the CLI are one executable (ADR-1ea31c2f3c5a), so a suite that looked
/// for two would be asserting the shape this task removed.
fn ank_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ank"))
}

/// A temporary directory nothing else in this suite uses.
fn scratch(what: &str) -> PathBuf {
    scratch::dir(what)
}

/// A reader's configuration home, holding the watch file and nothing else.
///
/// Both platform rules are pointed at one directory, so the file this suite
/// writes is the file the daemon reads on Windows and elsewhere alike:
/// `%APPDATA%\ank` and `$XDG_CONFIG_HOME/ank` resolve to the same place here.
struct Home(PathBuf);

impl Home {
    fn new() -> Home {
        let dir = scratch("home");
        std::fs::create_dir_all(dir.join("ank")).unwrap();
        Home(dir)
    }

    fn declare(&self, body: &str) {
        std::fs::write(self.0.join("ank").join("watch.yml"), body).unwrap();
    }

    fn watch_file(&self) -> PathBuf {
        self.0.join("ank").join("watch.yml")
    }

    /// The change stream, where this reader's environment puts it.
    ///
    /// Named through the contract rather than spelled here: the daemon writes
    /// it and `ank-tui` follows it, and a suite carrying its own copy of the
    /// name would agree with a stream that moved.
    fn stream(&self) -> PathBuf {
        self.0.join("ank").join(events::STREAM_FILE)
    }

    fn stream_text(&self) -> String {
        std::fs::read_to_string(self.stream()).unwrap_or_default()
    }

    fn apply(&self, cmd: &mut Command) {
        let config = isolated_git_config();
        cmd.env("GIT_CONFIG_GLOBAL", config)
            .env("GIT_CONFIG_SYSTEM", config)
            .env("XDG_CONFIG_HOME", &self.0)
            .env("APPDATA", &self.0)
            .env("HOME", &self.0)
            .env("ANK_AGENT", "test@ank.local");
    }

    /// The watcher, started the one way there is to start it.
    ///
    /// A verb of `ank` and not a file beside it: what the flags mean, and what
    /// each refusal says, is asserted against the surface a reader actually
    /// has.
    fn daemon(&self, args: &[&str]) -> Command {
        let mut c = Command::new(ank_bin());
        c.arg("watch").args(args);
        self.apply(&mut c);
        c
    }
}

/// A git repository carrying a corpus, built through the CLI.
struct Corpus {
    root: PathBuf,
}

impl Corpus {
    fn new(root: PathBuf, home: &Home) -> Corpus {
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        let corpus = Corpus { root };
        corpus.git(home, &["init", "-q", "-b", "main"]);
        corpus.git(home, &["config", "user.email", "test@ank.local"]);
        corpus.git(home, &["config", "user.name", "Test"]);
        corpus.git(home, &["config", "commit.gpgsign", "false"]);
        // Maintenance off, because git is otherwise free to repack a fixture
        // between two reads of it -- and this suite reads one twice to assert
        // that a watch touched nothing (TASK-fc6bef21e268).
        corpus.git(home, &["config", "gc.auto", "0"]);
        corpus.git(home, &["config", "maintenance.auto", "false"]);
        corpus.ank(home, &["init"]);
        corpus.ank(home, &["config", "default_branch", "main"]);
        corpus.ank(
            home,
            &[
                "new",
                "task",
                "--title",
                "A watched task",
                "--scope",
                "src/**",
            ],
        );
        corpus.git(home, &["add", "-A"]);
        corpus.git(home, &["commit", "-qm", "the corpus"]);
        corpus
    }

    fn git(&self, home: &Home, args: &[&str]) -> Output {
        let mut c = Command::new("git");
        c.args(args).current_dir(&self.root);
        home.apply(&mut c);
        let out = c
            .output()
            .expect("git is a hard dependency of this repository");
        assert!(
            out.status.success(),
            "git {args:?} in {}: {}",
            self.root.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    fn ank_raw(&self, home: &Home, args: &[&OsStr], cwd: &Path) -> Output {
        let mut c = Command::new(ank_bin());
        c.args(args).current_dir(cwd);
        home.apply(&mut c);
        c.output()
            .expect("cargo builds every binary of the workspace before running a test")
    }

    fn ank(&self, home: &Home, args: &[&str]) -> Output {
        let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
        let out = self.ank_raw(home, &owned, &self.root);
        assert!(
            out.status.success(),
            "ank {args:?} in {}: {}",
            self.root.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// The CLI under a named identity.
    ///
    /// Two clones with one `ANK_AGENT` are one agent as far as `refs/ank/*` can
    /// tell, and `status` filters its own identity out of what other agents
    /// hold -- so a fixture that let both sides share the default would assert
    /// the mirror by never reading it.
    fn ank_as(&self, home: &Home, agent: &str, args: &[&str]) -> Output {
        let mut c = Command::new(ank_bin());
        c.args(args).current_dir(&self.root);
        home.apply(&mut c);
        c.env("ANK_AGENT", agent);
        let out = c
            .output()
            .expect("cargo builds every binary of the workspace");
        assert!(
            out.status.success(),
            "ank {args:?} as {agent} in {}: {}",
            self.root.display(),
            String::from_utf8_lossy(&out.stderr)
        );
        out
    }

    /// Every `refs/ank/*` this repository carries, as `<name> <object>` lines.
    ///
    /// The plane and the mirror both, because the assertions that matter are
    /// about which of the two moved.
    fn ank_refs(&self, home: &Home, pattern: &str) -> String {
        let out = self.git(
            home,
            &["for-each-ref", "--format=%(refname) %(objectname)", pattern],
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    fn ank_dir(&self) -> PathBuf {
        self.root.join(".ank")
    }

    fn index(&self) -> PathBuf {
        self.ank_dir().join("index.db")
    }

    fn identity(&self, home: &Home) -> String {
        let out = self.git(home, &["rev-list", "--max-parents=0", "--reverse", "HEAD"]);
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .next()
            .unwrap()
            .trim()
            .to_string()
    }

    fn task_id(&self, home: &Home) -> String {
        let out = self.ank(home, &["find", "--status", "open", "--json"]);
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let at = text.find("\"id\":\"").expect("a corpus with one task");
        text[at + 6..at + 6 + 17].to_string()
    }

    fn drop_index(&self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(self.ank_dir().join(format!("index.db{suffix}")));
        }
    }
}

/// A bare repository two clones arbitrate through.
///
/// Bare and not a third checkout: `origin` here is what a forge is, a place
/// refs are pushed to and fetched from, and giving it a working tree would let
/// a test pass by writing into a file nobody would have in production.
fn bare(home: &Home, path: &Path) {
    let mut c = Command::new("git");
    c.args(["init", "-q", "--bare", "-b", "main"])
        .arg(path)
        .current_dir(std::env::temp_dir());
    home.apply(&mut c);
    let out = c.output().expect("git is a hard dependency");
    assert!(out.status.success(), "{out:?}");
    // The bare remote is a repository this suite builds, and git maintains one
    // it is not told not to (TASK-fc6bef21e268).
    for args in [
        ["config", "gc.auto", "0"],
        ["config", "maintenance.auto", "false"],
    ] {
        let mut c = Command::new("git");
        c.arg("--git-dir").arg(path).args(args);
        home.apply(&mut c);
        let out = c.output().expect("git is a hard dependency");
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }
}

/// A clone of `origin`, wired the way `git clone` wires one and no further.
///
/// **No `ank init` and therefore no `+refs/ank/*:refs/ank/*` refspec.** That is
/// the whole point: a clone made by hand fetches branches and tags, so
/// `refs/ank/claims/*` reaches it only when somebody runs the fetch by hand --
/// which is the staleness the watcher exists to remove, and it cannot be
/// demonstrated in a clone that was already synchronising itself.
fn clone(home: &Home, origin: &Path, into: &Path) -> Corpus {
    let mut c = Command::new("git");
    // Unmaintained from the moment the clone exists, rather than shortly after
    // it (TASK-fc6bef21e268).
    c.args([
        "clone",
        "-q",
        "-c",
        "gc.auto=0",
        "-c",
        "maintenance.auto=false",
    ])
    .arg(origin)
    .arg(into)
    .current_dir(std::env::temp_dir());
    home.apply(&mut c);
    let out = c.output().expect("git is a hard dependency");
    assert!(
        out.status.success(),
        "git clone: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let corpus = Corpus {
        root: into.to_path_buf(),
    };
    corpus.git(home, &["config", "user.email", "test@ank.local"]);
    corpus.git(home, &["config", "user.name", "Test"]);
    corpus.git(home, &["config", "commit.gpgsign", "false"]);
    corpus
}

/// Every file under a directory, with its bytes: the shape of "never touched".
fn snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(root, &path, out);
                continue;
            }
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((rel, std::fs::read(&path).unwrap_or_default()));
        }
    }
    walk(dir, dir, &mut out);
    out.sort();
    out
}

/// A running daemon, stopped when the test drops it.
struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

impl Running {
    fn stop(mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start(home: &Home, args: &[&str]) -> Running {
    let mut cmd = home.daemon(args);
    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    Running(cmd.spawn().expect("ank must have been built"))
}

/// The same, with stderr kept: what the watcher reports is an assertion in its
/// own right, and a failure it swallowed reads exactly like one it never had.
fn start_logging(home: &Home, args: &[&str], log: &Path) -> Running {
    let mut cmd = home.daemon(args);
    let file = std::fs::File::create(log).unwrap();
    cmd.stdout(Stdio::null()).stderr(Stdio::from(file));
    Running(cmd.spawn().expect("ank must have been built"))
}

/// Waits for a condition the daemon is meant to produce.
///
/// Bounded, and generous: this is asserting that something happens at all, not
/// how fast, so the wall is high enough that a loaded runner never reports the
/// runner instead of the code.
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

/// The bytes of a file the daemon has stopped writing.
///
/// Two identical, non-empty reads a poll apart. Without this, "the index
/// changed" would be satisfied by the tail of the write that produced it, and
/// the assertion would pass whether or not anything was ever reindexed.
fn settled(path: &Path) -> Vec<u8> {
    let mut last: Option<Vec<u8>> = None;
    let deadline = Instant::now() + Duration::from_secs(30);
    while Instant::now() < deadline {
        let now = std::fs::read(path).unwrap_or_default();
        if !now.is_empty() && last.as_deref() == Some(now.as_slice()) {
            return now;
        }
        last = Some(now);
        std::thread::sleep(Duration::from_millis(300));
    }
    panic!("timed out waiting for {} to settle", path.display());
}

// ---------------------------------------------------------------------------
// Starting, and refusing to
// ---------------------------------------------------------------------------

#[test]
fn a_declaration_naming_a_directory_with_no_ank_refuses_to_start() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("declared").join("tree"), &home);
    let bare = scratch("bare");
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        bare.display()
    ));
    let out = home.daemon(&["--once"]).output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains(&bare.display().to_string()), "{said}");
    assert!(
        said.contains("nothing looks for one elsewhere"),
        "the refusal has to say it does not search: {said}"
    );
}

#[test]
fn a_checkout_filed_under_another_repositorys_identity_refuses_to_start() {
    let home = Home::new();
    let mine = Corpus::new(scratch("mine").join("tree"), &home);
    let theirs = Corpus::new(scratch("theirs").join("tree"), &home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        mine.identity(&home),
        theirs.root.display()
    ));
    let out = home.daemon(&["--once"]).output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains(&theirs.identity(&home)), "{said}");
    assert!(said.contains(&mine.identity(&home)), "{said}");
}

#[test]
fn a_declaration_that_does_not_exist_refuses_to_start() {
    let home = Home::new();
    let out = home.daemon(&["--once"]).output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("watch.yml"), "{said}");
}

#[test]
fn the_watch_file_sits_beside_the_corpora_file() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("beside").join("tree"), &home);
    // The CLI writes its own declarations wherever this reader's home is
    // (ADR-96174f1ac2b7), and the daemon has to agree about where that is: a
    // reader with two homes has none. Both binaries are asked, and the answers
    // are compared rather than assumed.
    corpus.ank(
        &home,
        &[
            "config",
            "--user",
            &format!("corpora.{}", corpus.identity(&home)),
            &corpus.root.display().to_string(),
        ],
    );
    let out = home.daemon(&["--where"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(Path::new(&said), home.watch_file());
    assert!(
        home.watch_file()
            .parent()
            .unwrap()
            .join("corpora.yml")
            .is_file(),
        "the CLI wrote its declarations somewhere else"
    );
}

/// The four flags §4 gives this verb, and no fifth.
///
/// **The flag surface is where "it answers no verb" is kept mechanical.** Not
/// one of these asks the watcher a question about a corpus: `--list` and
/// `--where` report the declaration back and never a corpus's contents,
/// `--once` and `--interval` say when to look. A flag that queried a running
/// watcher is the shape ADR-24e21cb83793 refuses, and this is where its absence
/// is read off the binary rather than off a promise.
///
/// It is `ank help watch` and not a `--help` of the watcher's own, which is the
/// other half of the fold: one help surface, generated from the table both the
/// dispatch and the listing come out of (ADR-6fd69efb629c).
#[test]
fn the_verb_offers_the_four_flags_of_section_4_and_no_query() {
    let out = Command::new(ank_bin())
        .args(["help", "watch"])
        .output()
        .expect("ank must have been built");
    assert!(out.status.success(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stdout);
    for flag in ["--list", "--once", "--interval", "--where"] {
        assert!(said.contains(flag), "ank help watch omits {flag}:\n{said}");
    }
}

/// The two globals that address a corpus are refused, and refused rather than
/// ignored.
///
/// **This is the discovery ban reached from the caller's side.** Nothing walks a
/// filesystem looking for a corpus (ADR-24e21cb83793), and the mirror image of
/// that is a caller who *names* one: `ank watch --repo <tree>` reads as "watch
/// this", and a verb that took the flag and warmed something else would have
/// answered a question nobody could tell it had gone unanswered. `ank help
/// watch` therefore offers neither, on §9's rule that a name the verb rejects
/// by design may not appear in the offer, and the refusal points at the file
/// that does decide.
#[test]
fn addressing_one_corpus_is_refused_and_the_declaration_is_named() {
    let home = Home::new();
    for global in ["--repo", "--worktree"] {
        let out = home.daemon(&[global, "."]).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{global}: {out:?}");
        let said = String::from_utf8_lossy(&out.stderr);
        assert!(said.contains(global), "{said}");
        assert!(
            said.contains("--where"),
            "the refusal has to point at what does decide: {said}"
        );
    }

    // And the page does not offer what the verb rejects.
    let out = Command::new(ank_bin())
        .args(["help", "watch"])
        .output()
        .expect("ank must have been built");
    let said = String::from_utf8_lossy(&out.stdout);
    let offer = said
        .lines()
        .find(|l| l.trim_start().starts_with("global:"))
        .expect("ank help watch prints its globals");
    for refused in ["--repo", "--worktree"] {
        assert!(
            !offer.contains(refused),
            "ank help watch offers {refused} and the verb refuses it: {offer}"
        );
    }
}

/// A flag nobody declared is refused, and refused by the surface the reader
/// actually typed at.
///
/// The watcher used to parse its own argv and had its own word for this. It
/// does not any more: the parser refuses against the flags the table declares,
/// so a mistyped `ank watch` fails the way a mistyped `ank find` does, with §4's
/// environment code and the command that lists what there is. One parser, and
/// the refusal a reader learns once.
#[test]
fn an_unknown_flag_is_refused_by_the_verbs_own_surface() {
    let home = Home::new();
    let out = home.daemon(&["--scan-everything"]).output().unwrap();
    assert!(!out.status.success(), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("--scan-everything"), "{said}");
}

/// `--interval` is carried through to the watcher's own reading of it, and the
/// two refusals it can raise are still its own words.
///
/// The dispatch hands the value across as the caller typed it precisely so
/// these stay here: a number is what the watcher needs, so what a number has to
/// be is answered where it is needed and not re-derived beside it.
#[test]
fn an_interval_that_would_spin_is_refused_before_anything_is_read() {
    let home = Home::new();
    // No declaration at all, so a watcher that read the file before the number
    // would refuse about the file instead: the number is checked first.
    let out = home.daemon(&["--interval", "0"]).output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("would spin"), "{said}");

    let out = home.daemon(&["--interval", "soon"]).output().unwrap();
    assert_eq!(out.status.code(), Some(9), "{out:?}");
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("soon"), "{said}");
}

// ---------------------------------------------------------------------------
// Declared, never discovered
// ---------------------------------------------------------------------------

#[test]
fn nothing_walks_a_filesystem_looking_for_a_corpus() {
    let home = Home::new();
    let root = scratch("neighbourhood");
    // Three corpora nobody declared, planted in the three places a scan looks:
    // above the declared one, beside it, and inside it.
    let above = Corpus::new(root.join("above"), &home);
    let declared = Corpus::new(root.join("above").join("declared"), &home);
    let beside = Corpus::new(root.join("above").join("beside"), &home);
    let below = Corpus::new(root.join("above").join("declared").join("below"), &home);
    for undeclared in [&above, &beside, &below] {
        undeclared.drop_index();
    }
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        declared.identity(&home),
        declared.root.display()
    ));

    let listed = home.daemon(&["--list"]).output().unwrap();
    assert!(listed.status.success(), "{listed:?}");
    let text = String::from_utf8_lossy(&listed.stdout);
    assert!(text.contains("watching 1 corpus, 1 checkout"), "{text}");
    assert!(
        text.contains(&declared.root.display().to_string()),
        "{text}"
    );

    let before: Vec<_> = [&above, &beside, &below]
        .iter()
        .map(|c| snapshot(&c.ank_dir()))
        .collect();
    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(
        declared.index().is_file(),
        "the declared corpus is the one warmed"
    );

    for (undeclared, was) in [&above, &beside, &below].iter().zip(before) {
        // **Opening an index writes it.** So the absence of `index.db` is not a
        // proxy for "never read": it is the read itself, which cannot happen
        // without leaving this file behind.
        assert!(
            !undeclared.index().exists(),
            "{} was opened, and nothing declared it",
            undeclared.root.display()
        );
        assert_eq!(
            snapshot(&undeclared.ank_dir()),
            was,
            "{} changed under a daemon that was never told about it",
            undeclared.root.display()
        );
    }
}

// ---------------------------------------------------------------------------
// One repository, one corpus
// ---------------------------------------------------------------------------

#[test]
fn two_worktrees_of_one_repository_resolve_to_one_watched_corpus() {
    let home = Home::new();
    let root = scratch("worktrees");
    let first = Corpus::new(root.join("first"), &home);
    let second_path = root.join("second");
    first.git(
        &home,
        &["worktree", "add", "-q", &second_path.display().to_string()],
    );
    let second = Corpus { root: second_path };
    assert!(
        second.ank_dir().is_dir(),
        "a worktree checks out the corpus files like any other tree"
    );
    // Two paths, one key. The key is the root commit, which both worktrees
    // share and neither path carries (ADR-621a7fd96ce1).
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}:\n    - {}\n    - {}\n",
        first.identity(&home),
        first.root.display(),
        second.root.display()
    ));
    assert_eq!(first.identity(&home), second.identity(&home));

    let listed = home.daemon(&["--list"]).output().unwrap();
    assert!(listed.status.success(), "{listed:?}");
    let text = String::from_utf8_lossy(&listed.stdout);
    assert!(
        text.contains("watching 1 corpus, 2 checkouts"),
        "two worktrees are one corpus: {text}"
    );
    assert_eq!(
        text.lines().filter(|l| l.starts_with("corpus ")).count(),
        1,
        "{text}"
    );

    first.drop_index();
    second.drop_index();
    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(
        first.index().is_file(),
        "every checkout of the corpus is kept warm"
    );
    assert!(
        second.index().is_file(),
        "every checkout of the corpus is kept warm"
    );
}

#[test]
fn one_checkout_declared_twice_is_one_checkout() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("twice").join("tree"), &home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}:\n    - {}\n    - {}\n",
        corpus.identity(&home),
        corpus.root.display(),
        corpus.root.join(".").display()
    ));
    let listed = home.daemon(&["--list"]).output().unwrap();
    let text = String::from_utf8_lossy(&listed.stdout);
    assert!(text.contains("watching 1 corpus, 1 checkout"), "{text}");
}

// ---------------------------------------------------------------------------
// The cache is a cache
// ---------------------------------------------------------------------------

/// The verbs compared, and why these: a listing served straight out of the
/// index, a search served out of its full-text table, an entity read whole, the
/// graph, the scope resolution, and the one verb that writes.
const VERBS: &[&[&str]] = &[
    &["find", "--status", "open", "--json"],
    &["find", "watched", "--json"],
    &["graph"],
    &["scope", "src/**"],
    &["check"],
];

fn listings(corpus: &Corpus, home: &Home, task: &str) -> Vec<(String, Vec<u8>, Option<i32>)> {
    let mut out = Vec::new();
    for verb in VERBS {
        let owned: Vec<&OsStr> = verb.iter().map(|a| OsStr::new(*a)).collect();
        let got = corpus.ank_raw(home, &owned, &corpus.root);
        out.push((format!("{verb:?}"), got.stdout, got.status.code()));
    }
    let show: Vec<&OsStr> = vec![OsStr::new("show"), OsStr::new(task)];
    let got = corpus.ank_raw(home, &show, &corpus.root);
    out.push(("show".to_string(), got.stdout, got.status.code()));
    out
}

#[test]
fn a_warm_listing_equals_a_cold_listing_byte_for_byte() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("warm").join("tree"), &home);
    let task = corpus.task_id(&home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        corpus.root.display()
    ));

    corpus.drop_index();
    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(corpus.index().is_file(), "the daemon warmed nothing");
    let warm = listings(&corpus, &home, &task);

    // Cold: no daemon has run, and no index exists for one to have left behind.
    corpus.drop_index();
    let cold = listings(&corpus, &home, &task);

    for ((verb, warm_bytes, warm_code), (_, cold_bytes, cold_code)) in warm.iter().zip(&cold) {
        assert_eq!(
            String::from_utf8_lossy(warm_bytes),
            String::from_utf8_lossy(cold_bytes),
            "{verb} answered differently off a warm index"
        );
        assert_eq!(warm_code, cold_code, "{verb}");
    }
}

#[test]
fn the_index_follows_the_files_under_ank() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("follows").join("tree"), &home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        corpus.root.display()
    ));
    // A second entity, committed on a branch, so that a `git checkout` moves
    // files under `.ank/` without the CLI ever being asked to. That is the case
    // this exists for: a corpus changes under an agent that did not change it.
    corpus.git(&home, &["checkout", "-q", "-b", "more"]);
    corpus.ank(
        &home,
        &[
            "new",
            "task",
            "--title",
            "A later task",
            "--scope",
            "src/**",
        ],
    );
    corpus.git(&home, &["add", "-A"]);
    corpus.git(&home, &["commit", "-qm", "a later task"]);
    corpus.git(&home, &["checkout", "-q", "main"]);
    corpus.drop_index();

    let daemon = start(&home, &["--interval", "50"]);
    // **Settled, not merely present.** A file that exists may be one the daemon
    // is still writing, and recording those bytes would make the wait below
    // succeed against a half-written page rather than against a reindex. The
    // daemon rewrites only what moved, so an unchanging corpus goes quiet.
    let warmed = settled(&corpus.index());

    corpus.git(&home, &["checkout", "-q", "more"]);
    until("the index to follow the checkout", || {
        std::fs::read(corpus.index()).unwrap_or_default() != warmed
    });
    daemon.stop();

    // And what it reindexed is what the files say, which is the only property
    // that matters: the listing off this index is the listing off no index.
    let task = corpus.task_id(&home);
    let warm = listings(&corpus, &home, &task);
    corpus.drop_index();
    let cold = listings(&corpus, &home, &task);
    for ((verb, warm_bytes, warm_code), (_, cold_bytes, cold_code)) in warm.iter().zip(&cold) {
        assert_eq!(
            String::from_utf8_lossy(warm_bytes),
            String::from_utf8_lossy(cold_bytes),
            "{verb} answered differently after the daemon followed a checkout"
        );
        assert_eq!(warm_code, cold_code, "{verb}");
    }
}

// ---------------------------------------------------------------------------
// Nothing depends on it
// ---------------------------------------------------------------------------

#[test]
fn stopping_the_daemon_changes_no_verbs_output_and_no_verbs_exit_code() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("stopping").join("tree"), &home);
    let task = corpus.task_id(&home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        corpus.root.display()
    ));

    corpus.drop_index();
    let daemon = start(&home, &["--interval", "50"]);
    until("the daemon to warm the corpus", || corpus.index().is_file());
    let running = listings(&corpus, &home, &task);
    daemon.stop();

    let stopped = listings(&corpus, &home, &task);
    for ((verb, up_bytes, up_code), (_, down_bytes, down_code)) in running.iter().zip(&stopped) {
        assert_eq!(
            String::from_utf8_lossy(up_bytes),
            String::from_utf8_lossy(down_bytes),
            "{verb} is not the same verb with the daemon stopped"
        );
        assert_eq!(up_code, down_code, "{verb} changed its exit code");
    }
}

/// A checkout with no remote gains its own index and nothing whatsoever else.
///
/// The other half of the negative, and the case every solo repository is in:
/// with nothing to mirror there is no fetch, so `for-each-ref` is unchanged
/// entirely rather than unchanged outside one namespace. A watcher that
/// complained about the absent remote, or invented one, would show up here.
#[test]
fn the_only_thing_it_writes_into_a_repository_is_the_index() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("writes").join("tree"), &home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        corpus.root.display()
    ));
    corpus.drop_index();

    let refs_before = corpus.git(&home, &["for-each-ref"]).stdout;
    let head_before = corpus.git(&home, &["rev-parse", "HEAD"]).stdout;
    let tree_before = snapshot(&corpus.root.join("src"));
    let corpus_before: Vec<_> = snapshot(&corpus.ank_dir())
        .into_iter()
        .filter(|(name, _)| !name.starts_with("index.db"))
        .collect();

    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(corpus.index().is_file());

    assert_eq!(corpus.git(&home, &["for-each-ref"]).stdout, refs_before);
    assert_eq!(
        corpus.git(&home, &["rev-parse", "HEAD"]).stdout,
        head_before
    );
    assert_eq!(snapshot(&corpus.root.join("src")), tree_before);
    assert_eq!(
        snapshot(&corpus.ank_dir())
            .into_iter()
            .filter(|(name, _)| !name.starts_with("index.db"))
            .collect::<Vec<_>>(),
        corpus_before,
        "the corpus itself is read and never written"
    );
    assert!(
        String::from_utf8_lossy(&corpus.git(&home, &["status", "--porcelain"]).stdout)
            .trim()
            .is_empty(),
        "the working tree is not the daemon's to touch"
    );
}

// ---------------------------------------------------------------------------
// The mirror: refs/ank/*, and demonstrably nothing else
// ---------------------------------------------------------------------------

/// The fact this whole namespace exists for: `status` in one clone reports what
/// another clone holds, and nobody ran a fetch.
///
/// **Two clones is the only honest fixture.** The thing being fixed is that
/// `status` reports who holds what out of a stale local copy of the refs. One
/// clone cannot show that, because there is no second plane for the first to be
/// stale about, and a test that tried would be measuring nothing.
///
/// The second clone is made by `git clone` and never by `ank init`, so it
/// carries no `+refs/ank/*:refs/ank/*` refspec: without the watcher there is no
/// route by which a claim taken elsewhere reaches it at all.
#[test]
fn a_claim_taken_in_one_clone_is_reported_by_status_in_the_other() {
    let home = Home::new();
    let root = scratch("clones");
    let origin = root.join("origin.git");
    bare(&home, &origin);

    let first = Corpus::new(root.join("first"), &home);
    // `ank init` has already written `remote.origin.fetch`, so the section
    // exists and only the url is missing: `git remote add` would refuse.
    first.git(
        &home,
        &["config", "remote.origin.url", &origin.display().to_string()],
    );
    first.git(&home, &["push", "-q", "-u", "origin", "main"]);
    let second = clone(&home, &origin, &root.join("second"));
    assert_eq!(first.identity(&home), second.identity(&home));
    let task = first.task_id(&home);

    // Before anything: the second clone has no coordination plane at all, so
    // the claim below is unreachable from it by every route but the mirror.
    assert!(
        second.ank_refs(&home, "refs/ank").is_empty(),
        "a plain clone carries no refs/ank/*"
    );

    // One key, two roots: two clones of one repository are one watched corpus,
    // and each holds its own mirror because each is its own git repository.
    home.declare(&format!(
        "schema: 1\nfetch: 1\nwatch:\n  {}:\n    - {}\n    - {}\n",
        first.identity(&home),
        first.root.display(),
        second.root.display()
    ));

    first.ank_as(
        &home,
        "first@ank.local",
        &["claim", &task, "--criteria", "the mirror carries it"],
    );
    assert!(
        first
            .ank_refs(&home, "refs/ank/claims")
            .contains(&format!("refs/ank/claims/{task}")),
        "the claim is a ref in the clone that took it"
    );

    let elsewhere = |c: &Corpus| -> String {
        let out = c.ank_raw(
            &home,
            &[OsStr::new("status"), OsStr::new("--json")],
            &c.root,
        );
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    assert!(
        !elsewhere(&second).contains("first@ank.local"),
        "nothing has fetched yet, so the second clone cannot know"
    );

    let daemon = start(&home, &["--interval", "50"]);
    until("status in the second clone to report the claim", || {
        elsewhere(&second).contains("first@ank.local")
    });
    let said = elsewhere(&second);
    daemon.stop();

    assert!(said.contains(&task), "the task the claim is about: {said}");
    assert!(
        said.contains("A watched task"),
        "the title the corpus already carries: {said}"
    );
    // And it is reported as a claim held by somebody else, with the record read
    // rather than guessed at: the mirror carries the object, not merely the
    // name of a ref.
    assert!(
        said.contains("\"holder\":\"first@ank.local\""),
        "the holder comes out of the record: {said}"
    );
    // The local plane is where a claim of this clone's own would live, and the
    // watcher never wrote there.
    assert!(
        second.ank_refs(&home, "refs/ank/claims").is_empty(),
        "the mirror is not the plane"
    );
}

/// The negative, which is the point of this namespace.
///
/// "We only fetch `refs/ank/*`" is a sentence that stays true right up until a
/// refspec is written slightly wrong, so what must be identical afterwards is
/// listed and asserted against a repository that has something to lose: a
/// commit nobody pushed, a branch nobody pushed, a tag, a dirty working tree
/// and a claim of its own.
#[test]
fn a_watching_cycle_moves_the_tracking_namespace_and_nothing_else() {
    let home = Home::new();
    let root = scratch("negative");
    let origin = root.join("origin.git");
    bare(&home, &origin);

    let seed = Corpus::new(root.join("seed"), &home);
    seed.git(
        &home,
        &["config", "remote.origin.url", &origin.display().to_string()],
    );
    seed.git(&home, &["push", "-q", "-u", "origin", "main"]);

    let mine = clone(&home, &origin, &root.join("mine"));
    let task = mine.task_id(&home);

    // A tag pushed to the remote *after* the clone, so it is a ref the working
    // clone has never had. If the fetch followed tags, this is what would
    // appear in somebody's repository without anybody asking for it.
    seed.git(&home, &["tag", "on-the-remote"]);
    seed.git(&home, &["push", "-q", "origin", "on-the-remote"]);

    // A repository with something to lose.
    mine.git(&home, &["checkout", "-q", "-b", "unpushed"]);
    std::fs::write(mine.root.join("src/local.rs"), "// mine\n").unwrap();
    mine.git(&home, &["add", "-A"]);
    mine.git(&home, &["commit", "-qm", "a commit nobody has seen"]);
    mine.git(&home, &["tag", "mine-only"]);
    std::fs::write(mine.root.join("src/main.rs"), "fn main() { /* dirty */ }\n").unwrap();
    std::fs::write(mine.root.join("src/untracked.rs"), "// not added\n").unwrap();
    // A claim of this clone's own, which is the one ref a watcher must never
    // move: it is a live lease, and rewriting it from the remote would hand
    // somebody's task away underneath them.
    mine.ank_as(
        &home,
        "mine@ank.local",
        &["claim", &task, "--criteria", "the mirror never touches it"],
    );

    home.declare(&format!(
        "schema: 1\nfetch: 1\nwatch:\n  {}: {}\n",
        mine.identity(&home),
        mine.root.display()
    ));

    let head = mine.git(&home, &["rev-parse", "HEAD"]).stdout;
    let branches = mine.ank_refs(&home, "refs/heads");
    let tags = mine.ank_refs(&home, "refs/tags");
    let index = mine.git(&home, &["ls-files", "--stage"]).stdout;
    let porcelain = mine.git(&home, &["status", "--porcelain"]).stdout;
    let tree = snapshot(&mine.root.join("src"));
    let claims = mine.ank_refs(&home, "refs/ank/claims");
    let remotes = mine.ank_refs(&home, "refs/remotes");
    // Every ref this repository carries, whatever its namespace. The named
    // assertions below say what must not move and are worth reading; this one
    // says nothing else moved either, which no list of names can.
    let every_ref = mine.ank_refs(&home, "refs/");
    assert!(!claims.is_empty(), "the fixture holds a claim of its own");
    assert!(
        mine.ank_refs(&home, "refs/ank/watch").is_empty(),
        "nothing has mirrored yet"
    );

    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(out.status.success(), "{out:?}");

    // Whatever moved, moved inside the tracking namespace: the whole ref space
    // is diffed, and the mirror is the only difference allowed to appear.
    let after = mine.ank_refs(&home, "refs/");
    let moved: Vec<&str> = after
        .lines()
        .filter(|l| !every_ref.lines().any(|was| was == *l))
        .collect();
    assert!(!moved.is_empty(), "nothing was mirrored at all");
    assert!(
        moved
            .iter()
            .all(|l| l.starts_with("refs/ank/watch/origin/")),
        "a ref outside the tracking namespace moved: {moved:?}"
    );
    let gone: Vec<&str> = every_ref
        .lines()
        .filter(|l| !after.lines().any(|now| now == *l))
        .collect();
    assert!(gone.is_empty(), "a ref was removed: {gone:?}");

    // Only the mirror moved.
    assert!(
        mine.ank_refs(&home, "refs/ank/watch")
            .contains(&format!("refs/ank/watch/origin/claims/{task}")),
        "the mirror is what the cycle was for: {}",
        mine.ank_refs(&home, "refs/ank")
    );
    assert_eq!(
        mine.git(&home, &["rev-parse", "HEAD"]).stdout,
        head,
        "HEAD moved"
    );
    assert_eq!(
        mine.ank_refs(&home, "refs/heads"),
        branches,
        "a branch moved"
    );
    assert_eq!(
        mine.ank_refs(&home, "refs/tags"),
        tags,
        "a tag arrived, or one moved"
    );
    assert!(
        !mine.ank_refs(&home, "refs/tags").contains("on-the-remote"),
        "the remote's tag followed the fetch in"
    );
    assert_eq!(
        mine.git(&home, &["ls-files", "--stage"]).stdout,
        index,
        "git's index moved"
    );
    assert_eq!(
        mine.git(&home, &["status", "--porcelain"]).stdout,
        porcelain,
        "the working tree moved"
    );
    assert_eq!(snapshot(&mine.root.join("src")), tree, "a file changed");
    assert_eq!(
        mine.ank_refs(&home, "refs/ank/claims"),
        claims,
        "a claim of this clone's own was rewritten from the remote"
    );
    // And git's own mirror is untouched: a remote-tracking branch that moved
    // is how a background fetch usually announces itself, and the refspec here
    // is narrow precisely so none of them can.
    assert_eq!(
        mine.ank_refs(&home, "refs/remotes"),
        remotes,
        "a remote-tracking branch moved, so the fetch reached beyond refs/ank/*"
    );
}

/// A dead network is a normal Tuesday.
///
/// The watcher is optional by construction, so a failed mirror downgrades what
/// it offers, says so, and never stops it -- and never reaches the exit code of
/// anything the person is running.
#[test]
fn an_unreachable_remote_is_reported_and_the_watcher_keeps_watching() {
    let home = Home::new();
    let root = scratch("unreachable");
    let corpus = Corpus::new(root.join("tree"), &home);
    // A remote that is configured and cannot be reached, which is the case that
    // has to degrade: no remote at all is the other one, and it is silent.
    corpus.git(
        &home,
        &[
            "config",
            "remote.origin.url",
            &root.join("nowhere.git").display().to_string(),
        ],
    );
    home.declare(&format!(
        "schema: 1\nfetch: 1\nwatch:\n  {}: {}\n",
        corpus.identity(&home),
        corpus.root.display()
    ));

    // One cycle: it reports, and it still exits zero. Nothing depends on this
    // process, so a network it could not reach is not the caller's failure.
    let out = home.daemon(&["--once"]).output().unwrap();
    assert!(
        out.status.success(),
        "a dead remote is not an exit code: {out:?}"
    );
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("fetch:"), "the failure is reported: {said}");
    assert!(
        corpus.index().is_file(),
        "a corpus is warmed whether or not its remote answered"
    );

    // And it keeps watching: the failure repeats every cycle and the loop
    // survives all of them, which is the property a single pass cannot show.
    corpus.drop_index();
    let log = root.join("watcher.log");
    let daemon = start_logging(&home, &["--interval", "50"], &log);
    let warmed = settled(&corpus.index());
    corpus.ank(
        &home,
        &[
            "new",
            "task",
            "--title",
            "A later task",
            "--scope",
            "src/**",
        ],
    );
    until(
        "the watcher to follow the corpus past a dead remote",
        || std::fs::read(corpus.index()).unwrap_or_default() != warmed,
    );
    daemon.stop();
    let logged = std::fs::read_to_string(&log).unwrap_or_default();
    assert!(
        logged.matches("fetch:").count() >= 1,
        "the watcher went quiet about a remote it never reached: {logged}"
    );
}

// ---------------------------------------------------------------------------
// The stream: a change becomes an event (TASK-2f7777a1fdff)
// ---------------------------------------------------------------------------

/// A running daemon that has certainly finished its first look at a corpus.
///
/// **The first look is a sighting and not a change**, which is the property this
/// crate is careful about and the one that makes starting a watcher racy to test
/// against: a change made before the opening pass is part of what that pass
/// sees, so it produces no event and never will. Dropping the index and waiting
/// for it to come back is what says the pass happened -- the loop mirrors, takes
/// the fingerprint and then warms, so an index that exists again is a
/// fingerprint that has already been taken.
fn started_and_looked(home: &Home, corpus: &Corpus, args: &[&str]) -> Running {
    corpus.drop_index();
    let daemon = start(home, args);
    until("the watcher's first look", || corpus.index().is_file());
    daemon
}

/// Every line the stream could carry about one corpus, built by the encoder
/// both ends share.
///
/// The set is finite and it is two, which is what makes the assertion below
/// exhaustive rather than a sample: a line that is not one of these is a line
/// carrying something an event is not allowed to carry.
fn every_possible_line(identity: &str) -> Vec<String> {
    events::CHANGES
        .iter()
        .map(|c| {
            events::Event::new(identity, *c)
                .line()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn lines_of(text: &str) -> Vec<String> {
    text.lines().map(str::to_string).collect()
}

/// The change the reader was waiting for: what moved, and about which corpus.
///
/// **A first sighting is not a change**, and that is asserted first. The opening
/// pass warms every checkout it was handed, and a watcher that called its own
/// first look a change would wake every reader on the machine at startup for
/// news that is not news.
#[test]
fn a_change_becomes_an_event_naming_the_corpus_and_the_kind() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("event").join("tree"), &home);
    let identity = corpus.identity(&home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        identity,
        corpus.root.display()
    ));

    let warmed = home.daemon(&["--once"]).output().unwrap();
    assert!(warmed.status.success(), "{warmed:?}");
    assert_eq!(
        home.stream_text(),
        "",
        "the first look at a corpus is a sighting and not a change"
    );

    let daemon = started_and_looked(&home, &corpus, &["--interval", "50"]);
    corpus.ank(
        &home,
        &[
            "new",
            "task",
            "--title",
            "A second task",
            "--scope",
            "src/**",
        ],
    );
    until("the stream to carry the change", || {
        !home.stream_text().is_empty()
    });
    daemon.stop();

    let seen = lines_of(&home.stream_text());
    let expected = events::Event::new(&identity, events::Change::Entities)
        .line()
        .trim_end()
        .to_string();
    assert!(
        seen.contains(&expected),
        "the event names this corpus and says the entities moved:\n{seen:?}"
    );
}

/// **Two corpora, and each change lands keyed on the one it happened in.**
///
/// This is the property the fold had to keep and the reason it is asserted
/// rather than argued (TASK-9dd22f2b0430). A watcher is worth having because
/// one process covers every checkout a reader keeps -- the declaration is a
/// map, not a path -- and the two ways a fold could quietly break that are both
/// checked here. It could watch only the first corpus, which the second half of
/// the test catches. And it could report a change under the wrong identity, or
/// under none, which the keying catches: the line for each change names that
/// corpus and never the other, so a reader following the stream for one
/// repository is not woken by the other and does not miss its own.
///
/// **The changes are made one at a time, and each is waited for.** Two writes
/// inside one poll would be one change per corpus in the best case and an
/// ordering race in the worst, and what is being asserted is which corpus a
/// line names -- not how many lines a poll may coalesce.
///
/// The first pass is a sighting and not a change, for both corpora and for the
/// reason `a_change_becomes_an_event_naming_the_corpus_and_the_kind` gives, so
/// the stream is empty until something actually moves.
#[test]
fn a_declaration_naming_two_corpora_keys_each_change_on_its_own() {
    let home = Home::new();
    let root = scratch("two");
    let first = Corpus::new(root.join("first"), &home);
    let second = Corpus::new(root.join("second"), &home);
    let one = first.identity(&home);
    let two = second.identity(&home);
    // Two repositories and therefore two identities. Were these equal the whole
    // test would pass while asserting nothing, since every line would name the
    // one corpus there was.
    assert_ne!(
        one, two,
        "two git repositories with two root commits are two corpora"
    );
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n  {}: {}\n",
        one,
        first.root.display(),
        two,
        second.root.display()
    ));

    // Both are declared, so both are watched: a listing that named one would be
    // the failure this test exists for, seen before a single event.
    let listed = home.daemon(&["--list"]).output().unwrap();
    assert!(listed.status.success(), "{listed:?}");
    let listing = String::from_utf8_lossy(&listed.stdout);
    assert!(
        listing.contains(&one) && listing.contains(&two),
        "{listing}"
    );
    assert!(
        listing.contains("watching 2 corpora, 2 checkouts"),
        "{listing}"
    );

    first.drop_index();
    second.drop_index();
    let daemon = start(&home, &["--interval", "50"]);
    until("the watcher's first look at both corpora", || {
        first.index().is_file() && second.index().is_file()
    });
    assert_eq!(
        home.stream_text(),
        "",
        "the first look at a corpus is a sighting and not a change, in each of them"
    );

    let entities_moved = |identity: &str| {
        events::Event::new(identity, events::Change::Entities)
            .line()
            .trim_end()
            .to_string()
    };

    for (corpus, identity, title) in [
        (&first, &one, "A task in the first corpus"),
        (&second, &two, "A task in the second corpus"),
    ] {
        let expected = entities_moved(identity);
        corpus.ank(
            &home,
            &["new", "task", "--title", title, "--scope", "src/**"],
        );
        until("the stream to carry this corpus's change", || {
            lines_of(&home.stream_text()).contains(&expected)
        });
    }
    daemon.stop();

    // Both lines are there, which is the multi-corpus half.
    let seen = lines_of(&home.stream_text());
    for identity in [&one, &two] {
        assert!(
            seen.contains(&entities_moved(identity)),
            "no line names {identity}:\n{seen:?}"
        );
    }
    // And every line names a corpus that was declared, which is the keying
    // half: a change reported under the wrong identity, or under none, would
    // leave one of these two unmatched. The set of lines a corpus can produce
    // is finite and shared with the encoder, so this is exhaustive rather than
    // a sample.
    let possible: Vec<String> = every_possible_line(&one)
        .into_iter()
        .chain(every_possible_line(&two))
        .collect();
    for line in &seen {
        assert!(
            possible.contains(line),
            "a line names neither declared corpus:\n{line}\nof {possible:?}"
        );
    }
}

/// The other kind of change, and the one no local look could see: a claim taken
/// in a clone this reader cannot read, arriving through the mirror.
///
/// Two clones for the reason `a_claim_taken_in_one_clone_is_reported_by_status_in_the_other`
/// gives: a claim moving in the same checkout would be a file under `.ank/`
/// moving, which is the other event entirely.
#[test]
fn a_claim_taken_in_another_clone_becomes_a_refs_event() {
    let home = Home::new();
    let root = scratch("refsevent");
    let origin = root.join("origin.git");
    bare(&home, &origin);

    let first = Corpus::new(root.join("first"), &home);
    first.git(
        &home,
        &["config", "remote.origin.url", &origin.display().to_string()],
    );
    first.git(&home, &["push", "-q", "-u", "origin", "main"]);
    let second = clone(&home, &origin, &root.join("second"));
    let identity = first.identity(&home);
    let task = first.task_id(&home);

    // Only the second clone is watched, so nothing the first one does to its own
    // files can reach this stream: the only route left is the mirror.
    home.declare(&format!(
        "schema: 1\nfetch: 1\nwatch:\n  {}: {}\n",
        identity,
        second.root.display()
    ));
    // The first mirror has to have happened before the claim is taken, or the
    // claim is part of what the opening pass saw and is therefore not a change.
    let daemon = started_and_looked(&home, &second, &["--interval", "50"]);

    first.ank_as(
        &home,
        "first@ank.local",
        &["claim", &task, "--criteria", "the stream carries it"],
    );
    first.git(&home, &["push", "-q", "origin", "refs/ank/claims/*"]);

    let expected = events::Event::new(&identity, events::Change::Refs)
        .line()
        .trim_end()
        .to_string();
    until("the stream to say the refs moved", || {
        lines_of(&home.stream_text()).contains(&expected)
    });
    daemon.stop();
}

/// The property the criterion asks to be asserted rather than argued: an event
/// says what changed and never carries what the CLI answers.
///
/// **Stated exhaustively.** Every line of the stream is byte-identical to one of
/// the two the shared encoder can produce for this corpus, so there is no room
/// for a title, a status, a body or an identifier to have arrived -- the set of
/// possible lines is enumerated, not sampled.
///
/// The second half is the same statement made the way a person would check it,
/// against a corpus whose content is distinctive enough that a leak could not
/// hide: the identifier, the title and a log entry's message are all things a
/// reader gets by running `ank show`, and none of them is here.
#[test]
fn no_event_carries_entity_content_a_reader_would_get_from_the_cli() {
    let home = Home::new();
    let corpus = Corpus::new(scratch("content").join("tree"), &home);
    let identity = corpus.identity(&home);
    home.declare(&format!(
        "schema: 1\nwatch:\n  {}: {}\n",
        identity,
        corpus.root.display()
    ));

    let daemon = started_and_looked(&home, &corpus, &["--interval", "50"]);
    let title = "Piezoelectric ratchets in the transept";
    corpus.ank(
        &home,
        &["new", "task", "--title", title, "--scope", "src/**"],
    );
    until("the first change to reach the stream", || {
        !lines_of(&home.stream_text()).is_empty()
    });
    // The second is made after the first has landed, so it is a second event
    // and not the same one: two writes inside one poll are one change, which is
    // the watcher being a watcher rather than a journal.
    let second = "The transept was measured and the ratchet was not";
    corpus.ank(
        &home,
        &["new", "task", "--title", second, "--scope", "src/**"],
    );
    until("the second change to reach the stream", || {
        lines_of(&home.stream_text()).len() >= 2
    });
    let task = corpus.task_id(&home);
    daemon.stop();

    let text = home.stream_text();
    let possible = every_possible_line(&identity);
    for line in lines_of(&text) {
        assert!(
            possible.contains(&line),
            "an event carried something an event may not:\n{line}\nof {possible:?}"
        );
    }

    for leaked in [title, second, task.as_str(), "test@ank.local", "open"] {
        assert!(
            !text.contains(leaked),
            "the stream carries '{leaked}', which is what ank show answers:\n{text}"
        );
    }
}

/// Every git repository inside a fixture, found rather than listed.
///
/// A directory holding a `HEAD` file and an `objects` directory is one,
/// whether it is the `.git` beside a working tree or a bare corpus. Found,
/// because a list would have to be maintained, and what is being guarded
/// against is exactly a repository nobody remembered to enrol.
fn repositories_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
            found.push(dir);
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in entries.flatten() {
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            }
        }
    }
    found
}

/// What git answers for one key of one repository's own configuration, `None`
/// when the key is unset -- which is the state this asserts against, since an
/// unset `maintenance.auto` means maintenance is on.
///
/// `--local`, so what comes back is the fixture's answer and never the
/// machine's: a contributor carrying `gc.auto` in a global configuration would
/// otherwise read a pass out of a repository that sets nothing.
fn config_of(git_dir: &std::path::Path, key: &str) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("--git-dir")
        .arg(git_dir)
        .args(["config", "--local", "--get", key])
        .output()
        .expect("git must be installed: it is a hard dependency");
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Asserts that every repository under `root` is one git will not maintain.
///
/// Read back out of a freshly built fixture rather than grepped out of this
/// file: what is under test is the configuration `git init` was actually
/// followed by, and a grep passes on a comment and fails on a refactor.
fn assert_unmaintained(root: &std::path::Path) {
    let repos = repositories_under(root);
    assert!(
        !repos.is_empty(),
        "no repository found under {}: this asserts nothing",
        root.display()
    );
    for git_dir in repos {
        let at = git_dir.display();
        assert_eq!(
            config_of(&git_dir, "gc.auto").as_deref(),
            Some("0"),
            "gc.auto at {at}"
        );
        assert_eq!(
            config_of(&git_dir, "maintenance.auto").as_deref(),
            Some("false"),
            "maintenance.auto at {at}"
        );
    }
}

/// A fixture repository is not maintained under the test.
///
/// Measured on 2026-08-30 in run 33284185681: git repacked a fixture between
/// two fingerprints of it -- `objects/maintenance.lock`, a `tmp_pack` and six
/// loose objects in the first, a multi-pack-index, two packs and `info/refs`
/// in the second -- and a test asserting that a read writes nothing failed on
/// one platform of three. Ank had written nothing (TASK-fc6bef21e268).
///
/// The repositories are found by walking the fixture and not named here, so a
/// second one grown under it later is held to this without anyone remembering
/// to enrol it.
#[test]
fn a_fixture_repository_is_not_maintained_under_the_test() {
    let home = Home::new();
    let root = scratch("unmaintained");
    // All three shapes this suite builds, under one root: a corpus with a
    // working tree, the bare remote two clones arbitrate through, and a clone
    // made by hand.
    let _corpus = Corpus::new(root.join("tree"), &home);
    let origin = root.join("origin.git");
    bare(&home, &origin);
    let _cloned = clone(&home, &origin, &root.join("clone"));
    assert_unmaintained(&root);
}
