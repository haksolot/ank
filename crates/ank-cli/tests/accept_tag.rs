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

// ---------------------------------------------------------------------------
// ratify.require_signature (ADR-2e777475c4de, TASK-4b86ac17ca91)
// ---------------------------------------------------------------------------

/// A fresh ed25519 key beside the repository, and its public half as
/// `signers:` declares it, `<keytype> <base64>`.
fn new_key(c: &Corpus, name: &str) -> (PathBuf, String) {
    let key = c.repo.join("..").join(name);
    let out = spawn("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
        .arg(&key)
        .output()
        .expect("ssh-keygen ships with git on all three platforms");
    assert!(out.status.success(), "ssh-keygen: {}", text(&out.stderr));
    let public = fs::read_to_string(key.with_extension("pub")).unwrap();
    let mut fields = public.split_whitespace();
    let declared = format!("{} {}", fields.next().unwrap(), fields.next().unwrap());
    (key, declared)
}

/// git signs every tag with `key` from here on.
fn sign_with(c: &Corpus, key: &Path) {
    git(&c.repo, &["config", "gpg.format", "ssh"]);
    git(
        &c.repo,
        &[
            "config",
            "user.signingkey",
            key.with_extension("pub").to_str().unwrap(),
        ],
    );
}

/// The configuration `Corpus::new` writes, with `extra` after it.
fn configure(c: &Corpus, extra: &str) {
    fs::write(
        c.repo.join(".ank/config.yml"),
        format!("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n{extra}"),
    )
    .unwrap();
}

fn signers_block(key: &str) -> String {
    format!("signers:\n  marie@example.com:\n    keys:\n      - key: {key}\n")
}

const REQUIRED: &str = "ratify:\n  require_signature: true\n";

/// The `signature` findings `check --json` reports, as `(level, subject,
/// message)`.
fn signature_findings(c: &Corpus) -> (Option<i32>, Vec<(String, String, String)>) {
    let out = c.ank(&["check", "--json", "--all"]);
    let doc = text(&out.stdout);
    // One finding per `{"level":` and every value read here is a plain string
    // with no escaped quote in the cases these tests build.
    let value = |chunk: &str, key: &str| -> String {
        let at = format!("\"{key}\":\"");
        chunk
            .split_once(&at)
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(v, _)| v.to_string())
            .unwrap_or_default()
    };
    let found = doc
        .split("{\"level\":")
        .skip(1)
        .map(|chunk| format!("\"level\":{chunk}"))
        .filter(|chunk| value(chunk, "kind") == "signature")
        .map(|chunk| {
            (
                value(&chunk, "level"),
                value(&chunk, "subject"),
                value(&chunk, "message"),
            )
        })
        .collect();
    (out.status.code(), found)
}

/// Everything a refused accept must leave as it found it.
fn assert_untouched(c: &Corpus, refs: &str, before: &str) {
    assert_eq!(c.refs(), refs, "no tag, no ref, no branch moved");
    assert_eq!(c.local_ref(&ratify_ref(A)), None);
    assert_eq!(c.file(A), before, "the file is as it was");
    assert_eq!(
        git(&c.repo, &["status", "--porcelain"]),
        " M .ank/config.yml".trim(),
        "nothing but the configuration the test wrote"
    );
}

#[test]
fn a_required_signature_refuses_accept_with_no_key_to_sign_and_writes_nothing() {
    let c = Corpus::new("require-sig-no-key", false);
    let (_, declared) = new_key(&c, "declared-key");
    configure(&c, &format!("{}{REQUIRED}", signers_block(&declared)));
    let before = c.file(A);
    let refs = c.refs();

    let out = c.ank(&["accept", A]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{}{stderr}", text(&out.stdout));
    assert!(stderr.contains("require_signature"), "{stderr}");
    assert!(
        stderr.contains("user.signingkey"),
        "names the key it lacks: {stderr}"
    );
    assert_untouched(&c, &refs, &before);
}

#[test]
fn a_required_signature_refuses_a_key_signers_does_not_declare_and_names_it() {
    let c = Corpus::new("require-sig-undeclared", false);
    let (_, declared) = new_key(&c, "declared-key");
    let (other, undeclared) = new_key(&c, "other-key");
    sign_with(&c, &other);
    configure(&c, &format!("{}{REQUIRED}", signers_block(&declared)));
    let before = c.file(A);
    let refs = c.refs();

    let out = c.ank(&["accept", A]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{}{stderr}", text(&out.stdout));
    assert!(
        stderr.contains(&undeclared),
        "names the key it would sign with: {stderr}"
    );
    assert!(stderr.contains("signers"), "{stderr}");
    assert_untouched(&c, &refs, &before);
}

#[test]
fn a_required_signature_refuses_a_retired_key() {
    let c = Corpus::new("require-sig-retired", false);
    let (key, declared) = new_key(&c, "declared-key");
    sign_with(&c, &key);
    configure(
        &c,
        &format!(
            "{}        valid_before: 2000-01-01\n{REQUIRED}",
            signers_block(&declared)
        ),
    );
    let before = c.file(A);
    let refs = c.refs();

    let out = c.ank(&["accept", A]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{}{stderr}", text(&out.stdout));
    assert!(stderr.contains(&declared), "{stderr}");
    assert!(stderr.contains("2000-01-01"), "{stderr}");
    assert_untouched(&c, &refs, &before);
}

#[test]
fn a_required_signature_ratifies_with_a_declared_key_and_check_passes_it() {
    let c = Corpus::new("require-sig-declared", false);
    let (key, declared) = new_key(&c, "declared-key");
    sign_with(&c, &key);
    configure(&c, &format!("{}{REQUIRED}", signers_block(&declared)));

    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let tag = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    assert!(object.contains("-----BEGIN SSH SIGNATURE-----"), "{object}");

    let (_, found) = signature_findings(&c);
    assert_eq!(found, [], "a declared key passes");
}

#[test]
fn absent_or_false_accept_writes_an_unsigned_tag_as_before() {
    for (what, extra) in [
        ("require-sig-false", "ratify:\n  require_signature: false\n"),
        ("require-sig-absent", ""),
    ] {
        let c = Corpus::new(what, false);
        let (_, declared) = new_key(&c, "declared-key");
        configure(&c, &format!("{}{extra}", signers_block(&declared)));

        let out = c.ank(&["accept", A]);
        assert_eq!(out.status.code(), Some(0), "{what}: {}", text(&out.stderr));
        let tag = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
        let object = git(&c.repo, &["cat-file", "tag", &tag]);
        assert!(!object.contains("-----BEGIN"), "{what}: {object}");
    }
}

/// A ratification made before the key was turned on, and the regime turned
/// on afterwards by hand: every unsigned one is a fault, even where `signers:`
/// declares nothing, which is the advisory regime without the key.
#[test]
fn check_faults_an_unsigned_ratification_once_a_signature_is_required() {
    let c = Corpus::new("require-sig-check-unsigned", false);
    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    // No key declared: advisory, and the unsigned tag is not judged.
    let (code, found) = signature_findings(&c);
    assert_eq!((code, found), (Some(0), vec![]));

    // Required with no key declared, written by hand past `ank config`.
    configure(&c, REQUIRED);
    let (code, found) = signature_findings(&c);
    assert_eq!(code, Some(8), "{found:?}");
    assert!(
        found
            .iter()
            .any(|(level, subject, _)| level == "fault" && subject == A),
        "the unsigned ratification is a fault: {found:?}"
    );
    assert!(
        found
            .iter()
            .any(|(level, subject, _)| level == "fault" && subject == ".ank/config.yml"),
        "the configuration names a regime nobody can satisfy: {found:?}"
    );

    // Required with a key declared.
    let (_, declared) = new_key(&c, "declared-key");
    configure(&c, &format!("{}{REQUIRED}", signers_block(&declared)));
    let (code, found) = signature_findings(&c);
    assert_eq!(code, Some(8), "{found:?}");
    assert!(
        found.iter().any(|(level, subject, m)| level == "fault"
            && subject == A
            && m.contains("not signed")),
        "{found:?}"
    );
}

#[test]
fn check_faults_a_ratification_signed_by_an_undeclared_key_once_required() {
    let c = Corpus::new("require-sig-check-undeclared", false);
    let (other, _) = new_key(&c, "other-key");
    sign_with(&c, &other);
    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));

    let (_, declared) = new_key(&c, "declared-key");
    configure(&c, &format!("{}{REQUIRED}", signers_block(&declared)));
    let (code, found) = signature_findings(&c);
    assert_eq!(code, Some(8), "{found:?}");
    assert!(
        found.iter().any(|(level, subject, m)| level == "fault"
            && subject == A
            && m.contains("does not declare")),
        "{found:?}"
    );
}

#[test]
fn config_refuses_to_require_a_signature_while_signers_declares_no_key() {
    let c = Corpus::new("require-sig-config", false);
    let config = c.repo.join(".ank/config.yml");

    let out = c.ank(&["config", "ratify.require_signature"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stdout).trim(),
        "false (default)",
        "absent reads false"
    );

    let before = fs::read_to_string(&config).unwrap();
    let out = c.ank(&["config", "ratify.require_signature", "true"]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{stderr}");
    assert!(stderr.contains("signers"), "{stderr}");
    assert!(
        stderr.contains("ank signers add"),
        "names the repair: {stderr}"
    );
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        before,
        "nothing written"
    );

    // false is always a regime somebody can satisfy.
    let out = c.ank(&["config", "ratify.require_signature", "false"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let out = c.ank(&["config", "--unset", "ratify.require_signature"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    // The emptied mapping stays as `{}`, as every `<map>.<key>` unset leaves it.
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        format!("{before}ratify: {{}}\n")
    );

    let (_, declared) = new_key(&c, "declared-key");
    configure(&c, &signers_block(&declared));
    let out = c.ank(&["config", "ratify.require_signature", "true"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let written = fs::read_to_string(&config).unwrap();
    assert!(written.ends_with(REQUIRED), "{written}");
    let out = c.ank(&["config", "ratify.require_signature"]);
    assert_eq!(text(&out.stdout).trim(), "true");

    // A value that is not a boolean is refused like any other.
    let out = c.ank(&["config", "ratify.require_signature", "yes please"]);
    assert_ne!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let out = c.ank(&["config", "ratify.other", "true"]);
    assert_ne!(out.status.code(), Some(0), "{}", text(&out.stdout));
}

// ---------------------------------------------------------------------------
// A ratify ref already in the clone (#810, TASK-4937d03297e9)
// ---------------------------------------------------------------------------

/// Every tag object the clone holds, whatever names it: what a call that
/// "writes no new tag" must leave counted as it found it.
fn tag_objects(c: &Corpus) -> usize {
    git(
        &c.repo,
        &["cat-file", "--batch-all-objects", "--batch-check"],
    )
    .lines()
    .filter(|l| l.split(' ').nth(1) == Some("tag"))
    .count()
}

/// Ratifies `A` for real, then takes back the file and the push: the ref is
/// left in this clone alone, as a first `accept` that never finished leaves
/// it. Returns the tag the ref names.
fn leave_a_ratification_behind(c: &Corpus, before: &str) -> String {
    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let tag = c.local_ref(&ratify_ref(A)).expect("the ratify ref exists");
    git(
        &c.repo,
        &["push", "-q", "origin", &format!(":{}", ratify_ref(A))],
    );
    fs::write(c.repo.join(".ank/entities").join(format!("{A}.md")), before).unwrap();
    assert_eq!(c.origin_ref(&ratify_ref(A)), None);
    tag
}

/// What a refused accept over a leftover ref must print and keep.
fn assert_refused_over(c: &Corpus, out: &Output, left: &str, before: &str, why: &str) {
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(6), "{}{stderr}", text(&out.stdout));
    assert!(stderr.contains(&ratify_ref(A)), "names the ref: {stderr}");
    assert!(stderr.contains(why), "says why it does not match: {stderr}");
    assert!(
        stderr.contains(&format!("git update-ref -d {}", ratify_ref(A))),
        "prints the command that resolves it: {stderr}"
    );
    for git_text in ["fatal:", "cannot lock ref", "update_ref failed", "error: "] {
        assert!(!stderr.contains(git_text), "no git error text: {stderr}");
    }
    assert_eq!(
        c.local_ref(&ratify_ref(A)).as_deref(),
        Some(left),
        "ref kept"
    );
    assert_eq!(c.origin_ref(&ratify_ref(A)), None, "the remote untouched");
    assert_eq!(c.file(A), before, "the file byte-identical");
}

#[test]
fn a_leftover_ref_that_is_no_ratification_is_refused_naming_the_fix() {
    let c = Corpus::new("accept-leftover-foreign", true);
    let before = c.file(A);
    let blob = git(
        &c.repo,
        &["rev-parse", &format!("HEAD:.ank/entities/{A}.md")],
    );
    git(&c.repo, &["tag", "-a", "-m", "leftover", "tmp", &blob]);
    let left = git(&c.repo, &["rev-parse", "tmp"]);
    git(&c.repo, &["update-ref", &ratify_ref(A), &left]);
    git(&c.repo, &["tag", "-d", "tmp"]);
    let tags = tag_objects(&c);

    let out = c.ank(&["accept", A]);
    assert_refused_over(&c, &out, &left, &before, "not a ratification");
    assert_eq!(tag_objects(&c), tags, "no tag written");
}

#[test]
fn a_leftover_ratification_of_another_constraint_is_refused_naming_the_fix() {
    let c = Corpus::new("accept-leftover-stale", true);
    let original = c.file(A);
    let left = leave_a_ratification_behind(&c, &original);
    // The constraint moves after the ratification was written.
    let edited = original.replace("A binding rule.", "Another binding rule.");
    fs::write(
        c.repo.join(".ank/entities").join(format!("{A}.md")),
        &edited,
    )
    .unwrap();
    let tags = tag_objects(&c);

    let out = c.ank(&["accept", A]);
    assert_refused_over(&c, &out, &left, &edited, "constraint+scope");
    assert_eq!(tag_objects(&c), tags, "no tag written");
}

#[test]
fn a_leftover_ratification_that_matches_is_finished_with_the_same_tag() {
    let c = Corpus::new("accept-leftover-matches", true);
    let before = c.file(A);
    let left = leave_a_ratification_behind(&c, &before);
    let tags = tag_objects(&c);

    let out = c.ank(&["accept", A]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );

    assert_eq!(
        c.local_ref(&ratify_ref(A)).as_deref(),
        Some(left.as_str()),
        "the ref's oid is unchanged"
    );
    assert_eq!(
        c.origin_ref(&ratify_ref(A)).as_deref(),
        Some(left.as_str()),
        "that same tag object pushed"
    );
    assert_eq!(tag_objects(&c), tags, "no new tag object");
    let file = c.file(A);
    assert_eq!(field(&file, "status"), Some("accepted"), "{file}");
    assert_eq!(field(&file, "anchor"), Some(left.as_str()), "{file}");

    // The file is the blob the tag ratified, plus the anchor naming it.
    let object = git(&c.repo, &["cat-file", "tag", &left]);
    let blob = object
        .lines()
        .find_map(|l| l.strip_prefix("object "))
        .expect(&object);
    let without_anchor: String = file
        .lines()
        .filter(|l| !l.starts_with("anchor: "))
        .map(|l| format!("{l}\n"))
        .collect();
    assert_eq!(
        format!("{}\n", git(&c.repo, &["cat-file", "blob", blob])),
        without_anchor
    );
}

/// The state #810 was reported in: the leftover tag pushed by hand, so origin
/// holds it too, and the file still reads proposed.
#[test]
fn a_matching_ratification_origin_already_holds_is_finished_too() {
    let c = Corpus::new("accept-leftover-pushed", true);
    let before = c.file(A);
    let left = leave_a_ratification_behind(&c, &before);
    git(&c.repo, &["push", "-q", "origin", &ratify_ref(A)]);

    let out = c.ank(&["accept", A]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert_eq!(c.local_ref(&ratify_ref(A)).as_deref(), Some(left.as_str()));
    assert_eq!(c.origin_ref(&ratify_ref(A)).as_deref(), Some(left.as_str()));
    assert_eq!(field(&c.file(A), "status"), Some("accepted"));
}

#[test]
fn help_accept_lists_the_leftover_ref_refusal_with_its_code() {
    let c = Corpus::new("accept-leftover-help", false);
    let out = c.ank(&["help", "accept"]);
    let page = text(&out.stdout);
    let line = page
        .lines()
        .find(|l| l.contains("refs/ank/ratify/<id> already exists"))
        .unwrap_or_else(|| panic!("the refusal is listed: {page}"));
    assert!(line.contains('6'), "with its code: {line}");
}
