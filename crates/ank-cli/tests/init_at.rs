//! `ank init --at <dir>` writes nothing into `<dir>` when the declaration that
//! authorises it is refused (TASK-0dd151b02854).
//!
//! `init.rs` states the rule twice — `--repo` is refused "before anything is
//! written", and `detachable` is documented as taking everything a detached
//! corpus is refused for "before a byte is written" — and the declaration was
//! the one refusal that escaped that ordering. `init_at` ran first, so a
//! reader's `corpora.yml` at a schema this binary cannot read produced a
//! correct refusal *after* the corpus had been created. Measured before the
//! fix: the target was left holding `.ank/config.yml`, `.ank/entities`,
//! `.ank/log`, `AGENTS.md`, `.gitattributes`, `.gitignore` and a
//! `+refs/ank/*:refs/ank/*` refspec written into its own `.git/config`, with
//! nothing declared pointing at any of it.
//!
//! **Through the binary, and it has to be.** The criterion is about what a
//! directory holds after a process exits, and the reader's home is addressed
//! through the environment — `XDG_CONFIG_HOME`, or `APPDATA` on Windows — so
//! nothing but a child process can say "a reader whose declarations look like
//! this". `CARGO_BIN_EXE_ank` is defined only for an integration test, and
//! `ank-cli` has no library target.
//!
//! **The refusal is asserted as a whole, not as a list of filenames.** A test
//! naming the six artefacts would go green against a seventh. What is compared
//! is every path under the target with its contents, plus the target
//! repository's own local git config, taken before the run and after it.

mod scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The one schema `corpora.yml` is written at, hard-coded for the reason
/// `tests/schema.rs` gives: `ank-cli` has no library target, so
/// `config::SUPPORTED_SCHEMA` is not reachable from here.
const SUPPORTED: u32 = 1;

/// A reader's home holding one `corpora.yml`, and the environment pointing the
/// binary at it.
///
/// **Both variables, on every platform.** `user_dir` reads `APPDATA` on Windows
/// and `XDG_CONFIG_HOME` elsewhere; setting one and not the other would leave
/// this suite asserting nothing on one of the three platforms CLAUDE.md
/// requires OS-dependent behaviour to run on, and silently — the file would
/// simply not be found.
struct Reader {
    home: PathBuf,
    file: PathBuf,
}

impl Reader {
    fn new(what: &str, corpora: &str) -> Self {
        let home = scratch::dir(what).join("config");
        let dir = home.join("ank");
        fs::create_dir_all(&dir).expect("the reader's home must be creatable");
        let file = dir.join("corpora.yml");
        fs::write(&file, corpora).expect("corpora.yml must be writable");
        Reader { home, file }
    }

    fn text(&self) -> String {
        fs::read_to_string(&self.file).expect("corpora.yml must still be readable")
    }

    fn run(&self, dir: &Path, args: &[&str]) -> (i32, String) {
        let home = self.home.to_string_lossy().into_owned();
        let out = Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("XDG_CONFIG_HOME", &home)
            .env("APPDATA", &home)
            .env("HOME", "")
            .output()
            .expect("the binary under test must run");
        let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
        said.push_str(&String::from_utf8_lossy(&out.stderr));
        (
            out.status.code().expect("the binary must not be signalled"),
            said,
        )
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A git repository with one commit, which is all an identity needs
/// (ADR-621a7fd96ce1: the identity is the root commit).
fn repo(dir: &Path) {
    fs::create_dir_all(dir).expect("the repository must be creatable");
    git(dir, &["init", "-q", "."]);
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
            "-q",
            "--allow-empty",
            "-m",
            "root",
        ],
    );
}

/// Every path under `dir` with what it holds, `.git` excluded.
///
/// `.git` is left out and answered for separately by [`local_config`]: git
/// rewrites its own bookkeeping for reasons that have nothing to do with this
/// verb, and the one thing `init` writes in there is a config line, which is
/// compared exactly.
fn tree(dir: &Path) -> BTreeMap<String, String> {
    fn walk(root: &Path, at: &Path, into: &mut BTreeMap<String, String>) {
        let entries = fs::read_dir(at).unwrap_or_else(|e| panic!("{}: {e}", at.display()));
        for entry in entries.flatten() {
            let p = entry.path();
            let rel = p
                .strip_prefix(root)
                .expect("every walked path is under the root")
                .to_string_lossy()
                .replace('\\', "/");
            if rel == ".git" {
                continue;
            }
            if p.is_dir() {
                into.insert(format!("{rel}/"), String::new());
                walk(root, &p, into);
            } else {
                let what = fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                into.insert(rel, String::from_utf8_lossy(&what).into_owned());
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(dir, dir, &mut out);
    out
}

/// The target repository's own settings, which is where the `+refs/ank/*`
/// refspec lands.
fn local_config(dir: &Path) -> String {
    let mut lines: Vec<String> = git(dir, &["config", "--local", "--list"])
        .lines()
        .map(str::to_string)
        .collect();
    lines.sort();
    lines.join("\n")
}

/// A target that already holds files of its own, so "left as it was found"
/// covers a pointer appended to an existing `AGENTS.md` as well as one created
/// from nothing.
fn target(dir: &Path) {
    repo(dir);
    fs::write(dir.join("AGENTS.md"), "The reader's own notes.\n").expect("AGENTS.md is writable");
    fs::write(dir.join(".gitignore"), "*.tmp\n").expect(".gitignore is writable");
}

/// **The criterion.** A declaration this binary refuses leaves the directory it
/// was pointed at exactly as it found it.
#[test]
fn a_refused_declaration_leaves_the_target_exactly_as_it_was_found() {
    let reader = Reader::new(
        "init-at-refused",
        &format!("schema: {}\ncorpora: {{}}\n", SUPPORTED + 1),
    );
    let base = scratch::dir("init-at-refused");
    let (source, detached) = (base.join("repo"), base.join("detached"));
    repo(&source);
    target(&detached);

    let declarations = reader.text();
    let before = tree(&detached);
    let before_config = local_config(&detached);

    let (code, said) = reader.run(&source, &["init", "--at", &detached.to_string_lossy()]);

    assert_eq!(code, 1, "the declaration is refused: {said}");
    assert!(
        said.contains("newer than this ank"),
        "and refused on the schema: {said}"
    );
    assert_eq!(
        tree(&detached),
        before,
        "the target holds neither more nor less than it did"
    );
    assert_eq!(
        local_config(&detached),
        before_config,
        "and no refspec was written into its git config"
    );
    assert_eq!(
        reader.text(),
        declarations,
        "and the declarations file is untouched, as TASK-409e4c7d8aac left it"
    );
}

/// **The negative control, for the ordering rather than for the message.** A
/// refusal taken before the corpus is written would be worth nothing if it also
/// stopped the run that is allowed: the declaration still lands, and it lands
/// after the corpus it names.
#[test]
fn an_accepted_declaration_still_creates_the_corpus_and_declares_it() {
    let reader = Reader::new(
        "init-at-accepted",
        &format!("schema: {SUPPORTED}\ncorpora: {{}}\n"),
    );
    let base = scratch::dir("init-at-accepted");
    let (source, detached) = (base.join("repo"), base.join("detached"));
    repo(&source);
    target(&detached);
    let identity = git(&source, &["rev-list", "--max-parents=0", "HEAD"]);

    let (code, said) = reader.run(&source, &["init", "--at", &detached.to_string_lossy()]);

    assert_eq!(code, 0, "an ordinary detached init succeeds: {said}");
    assert!(
        detached.join(".ank").join("config.yml").is_file(),
        "the corpus is created at the target"
    );
    assert!(
        local_config(&detached).contains("+refs/ank/*:refs/ank/*"),
        "and carries the refspec: {}",
        local_config(&detached)
    );
    let declared = reader.text();
    assert!(
        declared.contains(&identity),
        "the declaration is keyed on the root commit: {declared}"
    );
    assert!(
        declared.contains(&detached.to_string_lossy().replace('\\', "\\\\"))
            || declared.contains(&*detached.to_string_lossy()),
        "and names the corpus: {declared}"
    );
}
