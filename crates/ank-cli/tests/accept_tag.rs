//! `ank accept` writes the ratification as an annotated tag object at
//! `refs/ank/ratify/<id>`, on the blob of the entity file, and commits nothing
//! (TASK-1f3e455dc134, ADR-5e47007a75be).
//!
//! Through the binary, on a feature branch, which is the half of the decision
//! that removed a precondition: the ref is a fact about the repository and not
//! about a branch. A bare origin reached by `file://` where the case is about
//! the push.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";
/// Proposed, replacing nothing.
const A: &str = "ADR-00000000000a";
/// Accepted, and retired by `C`.
const B: &str = "ADR-00000000000b";
/// Proposed, superseding `B`.
const C: &str = "ADR-00000000000c";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("accept-tag-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [init]\n\tdefaultBranch = main\n[protocol \"file\"]\n\tallow = always\n",
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

/// `file://` and the path with its separators swapped, and nothing else: three
/// slashes on Unix, two on Windows. `file:///C:/...` depends on git's MSYS path
/// conversion, which `MSYS_NO_PATHCONV` turns off for a `git` spawned with no
/// shell at all, as `clone_of` in `cli.rs` found.
fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_str().unwrap().replace('\\', "/"))
}

/// A script git runs, executable where that is a permission.
fn hook(path: &Path, body: &str) {
    fs::write(path, body).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

fn adr(id: &str, status: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: adr\nslug: a-decision\ntitle: A decision\n\
         created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: {status}\n\
         scope:\n  - src/**\nconstraint: |\n  A binding rule.\n{extra}schema: 4\nversion: 1\n---\n\nWhy.\n"
    )
}

/// Where the corpus is, and the origin when it has one.
struct Corpus {
    repo: PathBuf,
    origin: Option<PathBuf>,
}

impl Corpus {
    /// Three ADRs committed on `main`, and the checkout moved to a feature
    /// branch before anything is ratified.
    fn new(what: &str, with_origin: bool) -> Corpus {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        git(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        let entities = repo.join(".ank/entities");
        fs::write(entities.join(format!("{A}.md")), adr(A, "proposed", "")).unwrap();
        fs::write(entities.join(format!("{B}.md")), adr(B, "accepted", "")).unwrap();
        fs::write(
            entities.join(format!("{C}.md")),
            adr(C, "proposed", &format!("supersedes: {B}\n")),
        )
        .unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "seed"]);

        let origin = with_origin.then(|| {
            let origin = base.join("origin.git");
            fs::create_dir_all(&origin).unwrap();
            git(&origin, &["init", "-q", "--bare", "-b", "main"]);
            git(&repo, &["remote", "add", "origin", &file_url(&origin)]);
            git(&repo, &["push", "-q", "-u", "origin", "main"]);
            origin
        });
        git(&repo, &["switch", "-q", "-c", "feat/elsewhere"]);
        Corpus { repo, origin }
    }

    /// `accept` signs for real here, under an SSH key made for the fixture:
    /// `ssh-keygen` ships beside git on all three platforms and needs no agent,
    /// no keyring and no passphrase prompt.
    fn enable_signing(&self) {
        let key = self.repo.join("..").join("signing-key");
        let out = spawn("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
            .arg(&key)
            .output()
            .expect("ssh-keygen ships with git on all three platforms");
        assert!(out.status.success(), "ssh-keygen: {}", text(&out.stderr));
        git(&self.repo, &["config", "gpg.format", "ssh"]);
        git(
            &self.repo,
            &[
                "config",
                "user.signingkey",
                key.with_extension("pub").to_str().unwrap(),
            ],
        );
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .env("ANK_AGENT", AGENT)
            .current_dir(&self.repo)
            .output()
            .expect("the binary must have been built")
    }

    fn file(&self, id: &str) -> String {
        fs::read_to_string(self.repo.join(".ank/entities").join(format!("{id}.md"))).unwrap()
    }

    fn local_ref(&self, name: &str) -> Option<String> {
        rev(&self.repo, name)
    }

    fn origin_ref(&self, name: &str) -> Option<String> {
        rev(self.origin.as_ref().expect("a corpus with an origin"), name)
    }

    /// Everything a call must leave as it found it, the files aside: the
    /// history of every branch, the tags, and what `refs/ank/ratify` holds.
    fn refs(&self) -> String {
        git(
            &self.repo,
            &[
                "for-each-ref",
                "--format=%(refname) %(objectname)",
                "refs/heads",
                "refs/tags",
                "refs/ank/ratify",
            ],
        )
    }
}

fn rev(dir: &Path, name: &str) -> Option<String> {
    let out = spawn("git")
        .args(["rev-parse", "--verify", "-q", name])
        .current_dir(dir)
        .output()
        .unwrap();
    out.status
        .success()
        .then(|| text(&out.stdout).trim().to_string())
}

fn ratify_ref(id: &str) -> String {
    format!("refs/ank/ratify/{id}")
}

/// The value of a top-level scalar field, as the file carries it.
fn field<'a>(file: &'a str, key: &str) -> Option<&'a str> {
    file.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .map(str::trim)
}

#[test]
fn accept_on_a_feature_branch_writes_a_tag_on_the_blob_and_commits_nothing() {
    let c = Corpus::new("accept-tag-plain", false);
    let head = git(&c.repo, &["rev-parse", "HEAD"]);
    let log = git(&c.repo, &["log", "--all", "--format=%H %s"]);

    let out = c.ank(&["accept", A]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "accept runs on a feature branch: {}{}",
        text(&out.stdout),
        text(&out.stderr)
    );

    // Nothing in the history moved, and the only change is the file.
    assert_eq!(git(&c.repo, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&c.repo, &["log", "--all", "--format=%H %s"]), log);
    assert_eq!(
        git(&c.repo, &["status", "--porcelain"]),
        format!("M .ank/entities/{A}.md"),
        "the file and nothing else"
    );
    assert_eq!(
        git(&c.repo, &["for-each-ref", "refs/tags"]),
        "",
        "the temporary tag name is gone"
    );

    // The ref holds a tag object, on a blob, named for the ratification.
    let tag = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
    assert_eq!(git(&c.repo, &["cat-file", "-t", &tag]), "tag");
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    let (headers, message) = object.split_once("\n\n").expect(&object);
    assert!(headers.contains("\ntype blob"), "{object}");
    assert!(headers.contains(&format!("\ntag ratify/{A}")), "{object}");
    assert!(
        !object.contains("-----BEGIN"),
        "a repository that does not sign writes no signature: {object}"
    );

    // The file: accepted, its hash, the tag named, the reading recorded.
    let file = c.file(A);
    assert_eq!(field(&file, "status"), Some("accepted"), "{file}");
    let ratified = field(&file, "ratified").expect("ratified is written");
    assert_eq!(field(&file, "anchor"), Some(tag.as_str()), "{file}");
    assert!(file.contains(&format!("  - by: {AGENT}\n")), "{file}");

    // The message is the one the commit carried, line for line.
    assert_eq!(
        message,
        format!("ratify {A}\n\nconstraint+scope: {ratified}\nby: {AGENT}"),
        "{object}"
    );

    // The blob is the file as accept wrote it, but for the one anchor line.
    let blob = headers
        .lines()
        .find_map(|l| l.strip_prefix("object "))
        .expect(headers);
    let bytes = git(&c.repo, &["cat-file", "blob", blob]);
    let without_anchor: String = file
        .lines()
        .filter(|l| !l.starts_with("anchor: "))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(format!("{bytes}\n"), without_anchor);
    assert!(!bytes.contains("anchor:"), "{bytes}");
}

#[test]
fn a_repository_that_signs_writes_a_signed_tag() {
    let c = Corpus::new("accept-tag-signed", false);
    c.enable_signing();

    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    let tag = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    assert!(
        object.contains("-----BEGIN SSH SIGNATURE-----"),
        "signed where the repository signs: {object}"
    );
    assert!(
        object.starts_with("object ") && object.contains(&format!("ratify {A}\n")),
        "{object}"
    );
}

#[test]
fn the_ratify_ref_is_pushed_to_the_remote() {
    let c = Corpus::new("accept-tag-pushed", true);

    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    let local = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
    assert_eq!(c.origin_ref(&ratify_ref(A)), Some(local));
}

#[test]
fn a_push_the_remote_refuses_exits_nine_and_keeps_nothing() {
    let c = Corpus::new("accept-tag-refused", true);
    hook(
        &c.origin.as_ref().unwrap().join("hooks/pre-receive"),
        "#!/bin/sh\necho refused by policy >&2\nexit 1\n",
    );
    let before = c.file(A);
    let refs = c.refs();

    let out = c.ank(&["accept", A]);
    assert_eq!(
        out.status.code(),
        Some(9),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );

    assert_eq!(c.local_ref(&ratify_ref(A)), None, "the ref is not kept");
    assert_eq!(c.origin_ref(&ratify_ref(A)), None);
    assert_eq!(c.refs(), refs, "no tag, no ref, no branch moved");
    assert_eq!(c.file(A), before, "the file is as it was");
    assert_eq!(git(&c.repo, &["status", "--porcelain"]), "");
}

#[test]
fn a_supersession_marks_the_retired_document_and_the_tag_names_it() {
    let c = Corpus::new("accept-tag-supersedes", false);

    let out = c.ank(&["accept", C]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    assert_eq!(field(&c.file(B), "status"), Some("superseded"));
    assert_eq!(field(&c.file(C), "status"), Some("accepted"));
    let mut changed: Vec<String> = git(&c.repo, &["status", "--porcelain"])
        .lines()
        .map(|l| l.trim().to_string())
        .collect();
    changed.sort();
    assert_eq!(
        changed,
        [
            format!("M .ank/entities/{B}.md"),
            format!("M .ank/entities/{C}.md")
        ]
    );

    let tag = c.local_ref(&ratify_ref(C)).expect("the ratify ref exists");
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    assert!(object.contains(&format!("\nsupersedes: {B}\n")), "{object}");
    assert_eq!(c.local_ref(&ratify_ref(B)), None);
}

/// `--pr` carried the commits to a pull request, and there are no commits
/// left to carry: the flag is one this verb does not take.
#[test]
fn pr_is_gone_and_refused_as_a_flag_accept_does_not_take() {
    let c = Corpus::new("accept-tag-no-pr", false);
    let before = c.file(A);

    let out = c.ank(&["accept", A, "--pr"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stdout));
    assert!(text(&out.stderr).contains("--pr"), "{}", text(&out.stderr));
    assert_eq!(c.file(A), before);
    assert_eq!(c.local_ref(&ratify_ref(A)), None);
}
