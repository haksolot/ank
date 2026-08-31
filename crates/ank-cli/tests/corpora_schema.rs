//! The reader's `corpora.yml` is refused on its schema, not on the first key
//! the binary does not recognise (TASK-409e4c7d8aac).
//!
//! The rule is `docs/format.md`'s, and TASK-742cd978a806 already applied it to
//! `.ank/config.yml`:
//!
//! > Newer is refused, and refused **on the version rather than on the first
//! > field it does not recognise**. Since unknown fields are rejected, a tool
//! > that checked only its own version would report a file one version newer as
//! > *unknown field `author`*, and its reader would go hunting for a typo.
//!
//! `CorporaFile` carried the identical defect at two call sites in one file:
//! `deny_unknown_fields` fires during the deserialize, and `schema` is compared
//! only after it has already succeeded. Measured before the fix — a
//! `corpora.yml` at `schema: 2` with a `mirrors:` key answered *unknown field
//! `mirrors`, expected `schema` or `corpora`* on every verb.
//!
//! **The second call site was not a wrong sentence but a silent write.**
//! `parse_corpora` is used differentially — the write is refused when it
//! *introduces* a parse failure, never when the file already had one — so a
//! file at a newer schema failed both sides of that comparison and the guard
//! never fired. Measured: `ank config --user corpora.<identity> <path>` wrote
//! its entry into a `schema: 2` file and exited 0, leaving a file the same
//! binary refuses to read. That is why the refusal is taken where the existing
//! text is read, ahead of the surgery, and why this suite asserts the file is
//! left byte-identical rather than only that the exit code changed.
//!
//! **Through the binary, and it has to be.** The criterion is about the
//! sentence a reader is handed at the surface they use, and this file is
//! addressed through the environment: `XDG_CONFIG_HOME` decides where it lives
//! (`APPDATA` on Windows), so nothing but a child process can say "a reader
//! whose declarations look like this". `CARGO_BIN_EXE_ank` is defined only for
//! an integration test, and `ank-cli` has no library target.

mod scratch;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The one schema `corpora.yml` is written at, and the one this binary reads.
///
/// Hard-coded rather than imported, for the reason `tests/schema.rs` gives:
/// `ank-cli` has no library target, so `config::SUPPORTED_SCHEMA` is not
/// reachable from here.
const SUPPORTED: u32 = 1;

/// A reader's home holding exactly one `corpora.yml`, and the environment that
/// points the binary at it.
///
/// **Both variables, on every platform.** `user_dir` reads `APPDATA` on Windows
/// and `XDG_CONFIG_HOME` elsewhere; setting one and not the other would make
/// this suite assert nothing on one of the three platforms CLAUDE.md requires
/// OS-dependent behaviour to run on, and it would do it silently — the file
/// would simply not be found and every refusal here would vanish.
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

    fn env(&self) -> HashMap<&'static str, String> {
        let home = self.home.to_string_lossy().into_owned();
        HashMap::from([
            ("XDG_CONFIG_HOME", home.clone()),
            ("APPDATA", home),
            ("HOME", "".to_string()),
        ])
    }

    /// What `corpora.yml` holds right now, which is how a refusal is told from
    /// a write that merely printed one.
    fn text(&self) -> String {
        fs::read_to_string(&self.file).expect("corpora.yml must still be readable")
    }

    fn run(&self, dir: &Path, args: &[&str]) -> (i32, String) {
        let mut cmd = Command::new(ANK);
        cmd.args(args).current_dir(dir);
        for (k, v) in self.env() {
            cmd.env(k, v);
        }
        let out = cmd.output().expect("the binary under test must run");
        let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
        said.push_str(&String::from_utf8_lossy(&out.stderr));
        (
            out.status.code().expect("the binary must not be signalled"),
            said,
        )
    }
}

/// A git repository with one commit, which is all an identity needs
/// (ADR-621a7fd96ce1: the identity is the root commit).
///
/// Committer identity and signing are passed per-invocation rather than
/// inherited: a contributor whose global `commit.gpgsign` is on would otherwise
/// have this hang or fail on a passphrase.
fn repo(what: &str) -> PathBuf {
    let dir = scratch::dir(what).join("repo");
    fs::create_dir_all(&dir).expect("the repository must be creatable");
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q", "."]);
    git(&[
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
    ]);
    dir
}

/// The identity `corpora.yml` is keyed on: forty lowercase hex characters, and
/// nothing this suite invents.
fn identity(dir: &Path) -> String {
    let out = Command::new("git")
        .args(["rev-list", "--max-parents=0", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("git must run");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Every assertion the newer-than-this-binary sentence has to satisfy, in one
/// place: the two suites below say it about two different verbs and it is one
/// sentence or the fix is not done.
fn asserts_newer(said: &str, forbidden: &[&str]) {
    assert!(
        said.contains(&format!("schema {}", SUPPORTED + 1)),
        "the refusal names the schema the file declares: {said}"
    );
    assert!(
        said.contains(&format!("this binary reads {SUPPORTED}")),
        "the refusal names what this binary supports: {said}"
    );
    assert!(
        said.contains("newer than this ank"),
        "the refusal says the file is newer than the tool: {said}"
    );
    assert!(
        !said.contains("unknown field"),
        "a newer file is never refused on a field: {said}"
    );
    for key in forbidden {
        assert!(
            !said.contains(key),
            "the key that happens to be new is not the diagnosis: {said}"
        );
    }
}

// ---------------------------------------------------------------------------
// declarations(): the resolution every verb passes through
// ---------------------------------------------------------------------------

/// The branch that was broken, in the shape a future revision of the file
/// actually takes: one version ahead, carrying a key this binary has never
/// heard of.
///
/// `status` is incidental and the resolution is not — the reader's declarations
/// are read before any verb runs (ADR-96174f1ac2b7), so any verb fails
/// identically.
#[test]
fn a_newer_corpora_schema_is_refused_by_version_and_never_by_a_field() {
    let reader = Reader::new(
        "corpora-newer-with-key",
        &format!(
            "schema: {}\ncorpora: {{}}\nmirrors:\n  a: b\n",
            SUPPORTED + 1
        ),
    );
    let repo = repo("corpora-newer-with-key");
    let (code, said) = reader.run(&repo, &["status"]);

    assert_eq!(code, 9, "the reader's environment is refused: {said}");
    asserts_newer(&said, &["mirrors"]);
}

/// The same file with nothing unknown in it. This half was never a wrong
/// diagnosis — it said *schema 2 is not 1*, which names the version but not
/// what to do about it, and left the two shapes of one problem answering two
/// different sentences. What this pins is that they now answer one.
#[test]
fn a_newer_corpora_schema_is_refused_by_version_with_no_unknown_key() {
    let reader = Reader::new(
        "corpora-newer-clean",
        &format!("schema: {}\ncorpora: {{}}\n", SUPPORTED + 1),
    );
    let repo = repo("corpora-newer-clean");
    let (code, said) = reader.run(&repo, &["status"]);

    assert_eq!(code, 9, "the reader's environment is refused: {said}");
    asserts_newer(&said, &[]);
}

/// The counterweight, and the reason the fix is a version probe placed ahead of
/// the parse rather than a relaxed `deny_unknown_fields`. At a schema this
/// binary reads, an unrecognised key is a mistake in the file and the reader is
/// told which key, by name, with the alternatives spelled out.
#[test]
fn an_unknown_corpora_key_at_a_readable_schema_is_still_refused_by_name() {
    let reader = Reader::new(
        "corpora-typo",
        &format!("schema: {SUPPORTED}\ncorpora: {{}}\nmirrors:\n  a: b\n"),
    );
    let repo = repo("corpora-typo");
    let (code, said) = reader.run(&repo, &["status"]);

    assert_eq!(code, 9, "a typo is still refused: {said}");
    assert!(
        said.contains("unknown field `mirrors`"),
        "the refusal names the key the file got wrong: {said}"
    );
    assert!(
        said.contains("`schema`") && said.contains("`corpora`"),
        "and names the keys it could have been: {said}"
    );
    assert!(
        !said.contains("newer than this ank"),
        "a typo is never reported as a version problem: {said}"
    );
}

/// Below the range there is no older tool to name: no binary ever read schema
/// 0, so the file is simply wrong and keeps the message it has always had. The
/// same split `parse()` makes for `.ank/config.yml`.
#[test]
fn a_corpora_schema_below_the_range_keeps_its_own_refusal() {
    let reader = Reader::new("corpora-below", "schema: 0\ncorpora: {}\n");
    let repo = repo("corpora-below");
    let (code, said) = reader.run(&repo, &["status"]);

    assert_eq!(code, 9, "schema 0 is refused: {said}");
    assert!(
        said.contains(&format!("schema 0 is not {SUPPORTED}")),
        "the message below the range is unchanged: {said}"
    );
    assert!(
        !said.contains("newer than this ank"),
        "nothing below the range is newer than anything: {said}"
    );
}

// ---------------------------------------------------------------------------
// parse_corpora(): the two gestures that write this file
// ---------------------------------------------------------------------------

/// **The write that used to succeed.** `ank config --user` runs before the
/// corpus is resolved (`cli.rs`), so `declarations()` never gets a say, and the
/// differential guard is blind to a file that already failed. Before the fix
/// this exited 0 and added the entry. The file is compared byte for byte
/// because an exit code alone would not tell a refusal from a write that
/// happened to print one.
#[test]
fn config_user_refuses_to_write_into_a_corpora_from_a_newer_ank() {
    let reader = Reader::new(
        "corpora-write-newer",
        &format!(
            "schema: {}\ncorpora: {{}}\nmirrors:\n  a: b\n",
            SUPPORTED + 1
        ),
    );
    let repo = repo("corpora-write-newer");
    let before = reader.text();
    let id = identity(&repo);
    let (code, said) = reader.run(
        &repo,
        &["config", "--user", &format!("corpora.{id}"), "/srv/c"],
    );

    assert_eq!(code, 1, "the write is refused: {said}");
    asserts_newer(&said, &["mirrors"]);
    assert_eq!(
        reader.text(),
        before,
        "a file from a newer ank is left exactly as it was"
    );
}

/// `--unset` is a write too, and it reaches the same surgery. A refusal that
/// covered only the setting half would leave the other one deleting lines out
/// of a file this binary cannot read.
#[test]
fn config_user_unset_refuses_on_a_corpora_from_a_newer_ank() {
    let reader = Reader::new(
        "corpora-unset-newer",
        &format!("schema: {}\ncorpora: {{}}\n", SUPPORTED + 1),
    );
    let repo = repo("corpora-unset-newer");
    let before = reader.text();
    let id = identity(&repo);
    let (code, said) = reader.run(
        &repo,
        &["config", "--user", "--unset", &format!("corpora.{id}")],
    );

    assert_eq!(code, 1, "the unset is refused: {said}");
    asserts_newer(&said, &[]);
    assert_eq!(reader.text(), before, "and removes nothing");
}

/// The other gesture that edits this file: `init --at` declares the corpus it
/// detaches through `declare_corpus`, which performs the same surgery.
#[test]
fn init_at_refuses_to_declare_into_a_corpora_from_a_newer_ank() {
    let reader = Reader::new(
        "corpora-init-newer",
        &format!("schema: {}\ncorpora: {{}}\n", SUPPORTED + 1),
    );
    let repo = repo("corpora-init-newer");
    let before = reader.text();
    let detached = scratch::dir("corpora-init-newer").join("detached");
    fs::create_dir_all(&detached).expect("the detached corpus must be creatable");
    let out = Command::new("git")
        .args(["init", "-q", "."])
        .current_dir(&detached)
        .output()
        .expect("git must run");
    assert!(out.status.success(), "the detached target must be a repo");

    let (code, said) = reader.run(&repo, &["init", "--at", &detached.to_string_lossy()]);

    assert_eq!(code, 1, "the declaration is refused: {said}");
    asserts_newer(&said, &[]);
    assert_eq!(reader.text(), before, "and declares nothing");
}

/// **The negative control for the guard, not for the message.** At a schema
/// this binary reads, the write still lands: a version check placed ahead of
/// the surgery that also refused the ordinary case would be a regression
/// wearing a fix's clothes.
#[test]
fn config_user_still_writes_at_a_readable_schema() {
    let reader = Reader::new(
        "corpora-write-readable",
        &format!("schema: {SUPPORTED}\ncorpora: {{}}\n"),
    );
    let repo = repo("corpora-write-readable");
    let id = identity(&repo);
    let (code, said) = reader.run(
        &repo,
        &["config", "--user", &format!("corpora.{id}"), "/srv/c"],
    );

    assert_eq!(code, 0, "an ordinary declaration is written: {said}");
    assert!(
        reader.text().contains(&format!("{id}: /srv/c")),
        "the entry is in the file: {}",
        reader.text()
    );
}
