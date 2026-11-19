//! `ank reject <id> --reason <why>` turns a proposal down for good: the status
//! reads `rejected`, a tag at `refs/ank/reject/<id>` anchors the act the way
//! `refs/ank/ratify/<id>` anchors an acceptance, and a successor of a rejected
//! proposal is ratified without waiting on it (TASK-351d5cf24884,
//! ADR-0f4248812651, GitHub #777).
//!
//! Through the binary, on a feature branch, against a bare origin reached by
//! `file://` where the case is about the push.

use crate::{fixture, scratch};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";
/// Proposed, replacing nothing: the one turned down.
const A: &str = "ADR-00000000000a";
/// Accepted.
const B: &str = "ADR-00000000000b";
/// Proposed, superseding `A`.
const D: &str = "ADR-00000000000d";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("reject-gitconfig");
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

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_str().unwrap().replace('\\', "/"))
}

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

struct Corpus {
    repo: PathBuf,
    origin: Option<PathBuf>,
}

impl Corpus {
    /// Three ADRs committed on `main`, and the checkout moved to a feature
    /// branch before anything is decided.
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
            entities.join(format!("{D}.md")),
            // Written by an agent, so the human accepting it ratifies another's
            // proposal and check has nothing to say about who.
            adr(D, "proposed", &format!("supersedes: {A}\n"))
                .replace("author: human:marie", "author: claude-code/plan"),
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

    fn refs(&self) -> String {
        git(
            &self.repo,
            &[
                "for-each-ref",
                "--format=%(refname) %(objectname)",
                "refs/heads",
                "refs/tags",
                "refs/ank",
            ],
        )
    }

    /// Every finding `check --json --all` reports, as `(level, kind, subject,
    /// message)`, and the exit code.
    fn findings(&self) -> (Option<i32>, Vec<(String, String, String, String)>) {
        let out = self.ank(&["check", "--json", "--all"]);
        let doc = text(&out.stdout);
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
            .map(|c| {
                (
                    value(&c, "level"),
                    value(&c, "kind"),
                    value(&c, "subject"),
                    value(&c, "message"),
                )
            })
            .collect();
        (out.status.code(), found)
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

fn reject_ref(id: &str) -> String {
    format!("refs/ank/reject/{id}")
}

fn field<'a>(file: &'a str, key: &str) -> Option<&'a str> {
    file.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .map(str::trim)
}

const WHY: &str = "the pipeline it assumed was dropped";

#[test]
fn reject_rewrites_the_status_writes_and_pushes_the_ref_and_logs_the_reason() {
    let c = Corpus::new("reject-plain", true);
    let head = git(&c.repo, &["rev-parse", "HEAD"]);

    let out = c.ank(&["reject", A, "--reason", WHY]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("rejected"),
        "{}",
        text(&out.stdout)
    );

    // Nothing committed, nothing tagged under refs/tags.
    assert_eq!(git(&c.repo, &["rev-parse", "HEAD"]), head);
    assert_eq!(git(&c.repo, &["for-each-ref", "refs/tags"]), "");

    let file = c.file(A);
    assert_eq!(field(&file, "status"), Some("rejected"), "{file}");
    assert_eq!(field(&file, "version"), Some("2"), "{file}");
    assert_eq!(
        field(&file, "ratified"),
        None,
        "a rejection ratifies nothing"
    );

    // The ref: a tag object, on the blob of the file as reject wrote it, in
    // this clone and on origin.
    let tag = c.local_ref(&reject_ref(A)).expect("the reject ref exists");
    assert_eq!(c.origin_ref(&reject_ref(A)), Some(tag.clone()));
    assert_eq!(git(&c.repo, &["cat-file", "-t", &tag]), "tag");
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    let (headers, message) = object.split_once("\n\n").expect(&object);
    assert!(headers.contains("\ntype blob"), "{object}");
    assert!(headers.contains(&format!("\ntag reject/{A}")), "{object}");
    assert!(message.starts_with(&format!("reject {A}\n")), "{object}");
    assert!(message.contains(&format!("by: {AGENT}")), "{object}");
    let blob = headers
        .lines()
        .find_map(|l| l.strip_prefix("object "))
        .expect(headers);
    assert_eq!(
        format!("{}\n", git(&c.repo, &["cat-file", "blob", blob])),
        file
    );
    assert_eq!(c.local_ref(&format!("refs/ank/ratify/{A}")), None);

    // The reason is a log entry of the document.
    let log = c.ank(&["log", A]);
    assert_eq!(log.status.code(), Some(0), "{}", text(&log.stderr));
    assert!(
        text(&log.stdout).contains(&format!("rejected: {WHY}")),
        "{}",
        text(&log.stdout)
    );

    // It reads rejected everywhere.
    let show = c.ank(&["show", A]);
    assert!(
        text(&show.stdout).contains("status: rejected"),
        "{}",
        text(&show.stdout)
    );
    let find = c.ank(&["find", "decision", "--json"]);
    let find = text(&find.stdout);
    let row = find
        .split(&format!("\"id\":\"{A}\""))
        .nth(1)
        .unwrap_or_else(|| panic!("find lists {A}: {find}"));
    let row = row.split('}').next().unwrap();
    assert!(row.contains("\"status\":\"rejected\""), "{find}");
    let shown = c.ank(&["show", A, "--json"]);
    assert!(
        text(&shown.stdout).contains("\\nstatus: rejected\\n"),
        "{}",
        text(&shown.stdout)
    );

    // It leaves the queue and binds nobody.
    let review = c.ank(&["review", "--json"]);
    let review = text(&review.stdout);
    let proposed = review.split("\"signers\"").next().unwrap();
    assert!(!proposed.contains(A), "{review}");
    assert!(proposed.contains(D), "the other proposal stays: {review}");
    let context = c.ank(&["context", "src/lib.rs"]);
    let context = text(&context.stdout);
    assert!(!context.contains(A), "{context}");
    assert!(context.contains(B), "{context}");

    // And check finds it anchored.
    let (_, findings) = c.findings();
    let on_a: Vec<_> = findings
        .iter()
        .filter(|f| f.2 == A && f.0 == "fault")
        .collect();
    assert!(on_a.is_empty(), "{on_a:?}");
    assert!(
        !findings.iter().any(|f| f.2.contains("refs/ank/reject")),
        "the namespace is served: {findings:?}"
    );
}

#[test]
fn the_json_document_of_reject_is_pinned() {
    let c = Corpus::new("reject-json", false);
    let out = c.ank(&["reject", A, "--reason", WHY, "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    fixture::pin("reject", &text(&out.stdout));
}

#[test]
fn a_push_the_remote_refuses_exits_nine_and_keeps_nothing() {
    let c = Corpus::new("reject-refused", true);
    hook(
        &c.origin.as_ref().unwrap().join("hooks/pre-receive"),
        "#!/bin/sh\necho refused by policy >&2\nexit 1\n",
    );
    let before = c.file(A);
    let refs = c.refs();

    let out = c.ank(&["reject", A, "--reason", WHY]);
    assert_eq!(
        out.status.code(),
        Some(9),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert_eq!(c.local_ref(&reject_ref(A)), None, "the ref is not kept");
    assert_eq!(c.origin_ref(&reject_ref(A)), None);
    assert_eq!(c.refs(), refs);
    assert_eq!(c.file(A), before);
    assert_eq!(
        git(&c.repo, &["status", "--porcelain"]),
        "",
        "no log entry either"
    );
}

#[test]
fn a_document_that_is_not_proposed_is_refused_at_seven_and_nothing_is_written() {
    let c = Corpus::new("reject-accepted", false);
    let before = c.file(B);
    let refs = c.refs();

    let out = c.ank(&["reject", B, "--reason", WHY]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{stderr}");
    assert!(stderr.contains("accepted"), "{stderr}");
    assert_eq!(c.file(B), before);
    assert_eq!(c.refs(), refs);
    assert_eq!(git(&c.repo, &["status", "--porcelain"]), "");

    // And a second reject of a rejected one, the same way.
    assert_eq!(
        c.ank(&["reject", A, "--reason", WHY]).status.code(),
        Some(0)
    );
    let refs = c.refs();
    let out = c.ank(&["reject", A, "--reason", WHY]);
    assert_eq!(out.status.code(), Some(7), "{}", text(&out.stderr));
    assert_eq!(c.refs(), refs);
}

#[test]
fn a_missing_reason_is_refused_at_one() {
    let c = Corpus::new("reject-no-reason", false);
    let before = c.file(A);
    for args in [vec!["reject", A], vec!["reject", A, "--reason", "  "]] {
        let out = c.ank(&args);
        let stderr = text(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {stderr}");
        assert!(stderr.contains("--reason"), "{stderr}");
        assert_eq!(c.file(A), before);
        assert_eq!(c.local_ref(&reject_ref(A)), None);
    }
}

#[test]
fn a_required_signature_refuses_reject_with_no_key_to_sign() {
    let c = Corpus::new("reject-require-sig", false);
    // A declared key, and git configured to sign with none.
    let key = c.repo.join("..").join("declared-key");
    let made = spawn("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
        .arg(&key)
        .output()
        .expect("ssh-keygen ships with git on all three platforms");
    assert!(made.status.success(), "ssh-keygen: {}", text(&made.stderr));
    let public = fs::read_to_string(key.with_extension("pub")).unwrap();
    let mut fields = public.split_whitespace();
    let declared = format!("{} {}", fields.next().unwrap(), fields.next().unwrap());
    fs::write(
        c.repo.join(".ank/config.yml"),
        format!(
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n\
             signers:\n  marie@example.com:\n    keys:\n      - key: {declared}\n\
             ratify:\n  require_signature: true\n"
        ),
    )
    .unwrap();
    let before = c.file(A);

    let out = c.ank(&["reject", A, "--reason", WHY]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{stderr}");
    assert!(stderr.contains("require_signature"), "{stderr}");
    assert_eq!(c.file(A), before);
    assert_eq!(c.local_ref(&reject_ref(A)), None);
}

#[test]
fn a_successor_of_a_proposal_names_reject_and_of_a_rejected_one_is_ratified() {
    let c = Corpus::new("reject-successor", false);

    // The target is proposed: the way out is to turn it down, never to ratify
    // the very decision being replaced.
    let out = c.ank(&["accept", D]);
    let stderr = text(&out.stderr);
    assert_eq!(out.status.code(), Some(7), "{stderr}");
    assert!(stderr.contains(&format!("ank reject {A}")), "{stderr}");
    assert!(!stderr.contains(&format!("ank accept {A}")), "{stderr}");
    assert_eq!(field(&c.file(D), "status"), Some("proposed"));

    assert_eq!(
        c.ank(&["reject", A, "--reason", WHY]).status.code(),
        Some(0)
    );
    let out = c.ank(&["accept", D]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}{}",
        text(&out.stdout),
        text(&out.stderr)
    );
    assert_eq!(field(&c.file(D), "status"), Some("accepted"));
    assert_eq!(
        field(&c.file(A), "status"),
        Some("rejected"),
        "nothing superseded"
    );
    assert!(
        !text(&out.stdout).contains("superseded"),
        "{}",
        text(&out.stdout)
    );

    let (_, findings) = c.findings();
    let pair: Vec<_> = findings.iter().filter(|f| f.2 == A || f.2 == D).collect();
    assert!(pair.is_empty(), "check is silent on the pair: {pair:?}");
}

#[test]
fn check_faults_a_document_reading_rejected_with_no_reject_ref() {
    let c = Corpus::new("reject-forged", false);
    let path = c.repo.join(".ank/entities").join(format!("{A}.md"));
    fs::write(&path, adr(A, "rejected", "")).unwrap();
    git(&c.repo, &["commit", "-qam", "typed by hand"]);

    let (code, findings) = c.findings();
    assert_eq!(code, Some(8), "{findings:?}");
    assert!(
        findings
            .iter()
            .any(|f| f.0 == "fault" && f.2 == A && f.3.contains(&reject_ref(A))),
        "{findings:?}"
    );
}

#[test]
fn help_lists_reject() {
    let c = Corpus::new("reject-help", false);
    let out = c.ank(&["help"]);
    assert!(
        text(&out.stdout).contains("reject"),
        "{}",
        text(&out.stdout)
    );
    let out = c.ank(&["help", "reject"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).contains("--reason"),
        "{}",
        text(&out.stdout)
    );
}
