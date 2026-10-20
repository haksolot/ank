//! The readers find a ratification by its ref first, `refs/ank/ratify/<id>`,
//! and by the `ratify <id>` commit walk only for an anchor made before
//! ADR-5e47007a75be (TASK-6e3f8f0049e9).
//!
//! Through the binary throughout, and from a second clone where the case is
//! about what travels: `accept` pushes the ref and leaves the file it wrote
//! uncommitted, so a clone that fetched `refs/ank/*` holds a verified
//! ratification beside a file that still reads `proposed`. Signed for real,
//! under an SSH key made for the fixture and declared under `signers:`.
//!
//! Process counts come from `GIT_TRACE` at an absolute path, never from a
//! clock (CLAUDE.md).

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const HUMAN: &str = "human:marie";
/// Proposed, replacing nothing: what `accept` ratifies in most cases here.
const A: &str = "ADR-0000000006aa";
/// Accepted by hand, with no anchor: what `C` retires.
const B: &str = "ADR-0000000006bb";
/// Proposed, superseding `B`.
const C: &str = "ADR-0000000006cc";
/// What `GIT_TRACE` prefixes the argument list of every git with.
const MARK: &str = "trace: built-in: git ";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("ratify-ref-gitconfig");
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

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn git_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> String {
    let mut c = spawn("git");
    c.args(args).current_dir(dir);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.output().expect("git must be on PATH");
    assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    text(&out.stdout).trim().to_string()
}

fn git(dir: &Path, args: &[&str]) -> String {
    git_env(dir, args, &[])
}

/// `file://` and the path with its separators swapped, for the reason
/// `accept_tag.rs` gives: `file:///C:/...` depends on MSYS path conversion.
fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_str().unwrap().replace('\\', "/"))
}

fn adr(id: &str, status: &str, extra: &str) -> String {
    format!(
        "---\nid: {id}\ntype: adr\nslug: a-decision\ntitle: Decision {id}\n\
         created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: {status}\n\
         scope:\n  - src/**\nconstraint: |\n  The rule {id} binds.\n{extra}schema: 4\nversion: 1\n---\n\nWhy.\n"
    )
}

struct Corpus {
    base: PathBuf,
    repo: PathBuf,
    origin: PathBuf,
    /// The public half as `signers:` declares it.
    declared: String,
}

impl Corpus {
    /// `A` proposed, `B` accepted by hand, `C` proposed over `B`, pushed to a
    /// bare origin; the repository signs with a fixture key that `signers:`
    /// declares, committed with the rest.
    fn new(what: &str) -> Corpus {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        let origin = base.join("origin.git");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::create_dir_all(&origin).unwrap();
        git(&origin, &["init", "-q", "--bare", "-b", "main"]);
        git(&repo, &["init", "-q", "-b", "main"]);

        let key = base.join("signing-key");
        let out = spawn("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-C", "marie@laptop", "-q", "-f"])
            .arg(&key)
            .output()
            .expect("ssh-keygen ships with git on all three platforms");
        assert!(out.status.success(), "ssh-keygen: {}", text(&out.stderr));
        let public = fs::read_to_string(key.with_extension("pub")).unwrap();
        let declared = public
            .split_whitespace()
            .take(2)
            .collect::<Vec<_>>()
            .join(" ");
        let c = Corpus {
            base,
            repo,
            origin,
            declared,
        };
        c.sign_with(&c.repo, &key);

        fs::write(
            c.repo.join(".ank/config.yml"),
            format!(
                "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\nsigners:\n  marie@laptop:\n    keys:\n      - key: {}\n",
                c.declared
            ),
        )
        .unwrap();
        fs::write(c.repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(c.repo.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        c.put(A, &adr(A, "proposed", ""));
        c.put(B, &adr(B, "accepted", ""));
        c.put(C, &adr(C, "proposed", &format!("supersedes: {B}\n")));
        git(&c.repo, &["add", "-A"]);
        git(&c.repo, &["commit", "-qm", "seed"]);
        git(&c.repo, &["remote", "add", "origin", &file_url(&c.origin)]);
        git(&c.repo, &["push", "-q", "-u", "origin", "main"]);
        c
    }

    fn sign_with(&self, dir: &Path, key: &Path) {
        git(dir, &["config", "gpg.format", "ssh"]);
        git(
            dir,
            &[
                "config",
                "user.signingkey",
                key.with_extension("pub").to_str().unwrap(),
            ],
        );
    }

    fn put(&self, id: &str, body: &str) {
        fs::write(
            self.repo.join(".ank/entities").join(format!("{id}.md")),
            body,
        )
        .unwrap();
    }

    fn file(&self, id: &str) -> String {
        fs::read_to_string(self.repo.join(".ank/entities").join(format!("{id}.md"))).unwrap()
    }

    /// A clone of the origin as another machine makes it: the default
    /// branch, then `refs/ank/*` fetched read-only, as CI does.
    fn second_clone(&self) -> PathBuf {
        let clone = self.base.join("clone");
        git(
            &self.base,
            &[
                "clone",
                "-q",
                &file_url(&self.origin),
                clone.to_str().unwrap(),
            ],
        );
        git(&clone, &["fetch", "-q", "origin", "refs/ank/*:refs/ank/*"]);
        clone
    }

    fn accept(&self, id: &str) {
        let out = ank(&self.repo, &["accept", id]);
        assert_eq!(out.status.code(), Some(0), "accept: {}", text(&out.stderr));
    }
}

fn ank(dir: &Path, args: &[&str]) -> Output {
    ank_env(dir, args, &[])
}

fn ank_env(dir: &Path, args: &[&str], env: &[(&str, &Path)]) -> Output {
    let mut c = spawn(ANK);
    c.args(args).env("ANK_AGENT", HUMAN).current_dir(dir);
    for (k, v) in env {
        c.env(k, v);
    }
    c.output().expect("the binary must have been built")
}

fn both(out: &Output) -> String {
    text(&out.stdout) + &text(&out.stderr)
}

fn ratify_ref(id: &str) -> String {
    format!("refs/ank/ratify/{id}")
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

/// The lines `check` printed about `id`, faults and signals both.
fn about(report: &str, id: &str) -> Vec<String> {
    report
        .lines()
        .filter(|l| l.contains(id))
        .map(str::to_string)
        .collect()
}

fn faults(report: &str) -> Vec<String> {
    report
        .lines()
        .filter(|l| l.starts_with("error:"))
        .map(str::to_string)
        .collect()
}

/// A verified ref whose file has not caught up binds where the file is read:
/// `check` names the file, `context` serves the constraint as binding and
/// `scope` lists it accepted.
#[test]
fn a_second_clone_holding_the_ref_and_a_proposed_file_serves_the_constraint_as_binding() {
    let c = Corpus::new("ratify-ref-second-clone");
    c.accept(A);
    let clone = c.second_clone();
    let file = fs::read_to_string(clone.join(format!(".ank/entities/{A}.md"))).unwrap();
    assert!(file.contains("status: proposed"), "{file}");
    assert!(rev(&clone, &ratify_ref(A)).is_some(), "the ref travelled");

    let out = ank(&clone, &["check"]);
    let report = both(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a signal, not a fault: {report}"
    );
    let lines = about(&report, A);
    assert!(
        lines.iter().any(|l| l.starts_with("signal:")
            && l.contains(&format!(".ank/entities/{A}.md"))
            && l.contains("proposed")
            && l.contains(&ratify_ref(A))),
        "check names the file that has not caught up: {report}"
    );

    let out = ank(&clone, &["context", "src/lib.rs", "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    let ids = |key: &str| -> Vec<String> {
        doc[key]
            .as_sequence()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap().to_string())
            .collect()
    };
    assert!(ids("constraints").contains(&A.to_string()), "{doc:?}");
    assert!(!ids("proposed").contains(&A.to_string()), "{doc:?}");
    let served = doc["constraints"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|c| c["id"].as_str() == Some(A))
        .unwrap();
    assert!(
        served["constraint"]
            .as_str()
            .unwrap()
            .contains(&format!("The rule {A} binds.")),
        "{served:?}"
    );

    let out = ank(&clone, &["scope", "src/lib.rs", "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    let row = doc["adr"]
        .as_sequence()
        .unwrap()
        .iter()
        .find(|r| r["id"].as_str() == Some(A))
        .expect(&format!("{doc:?}"))
        .clone();
    assert_eq!(row["status"].as_str(), Some("accepted"), "{doc:?}");
    let out = ank(&clone, &["scope", "src/lib.rs"]);
    let line = text(&out.stdout)
        .lines()
        .find(|l| l.contains(&A[..15]))
        .unwrap_or_default()
        .to_string();
    assert!(line.contains("[accepted]"), "{}", text(&out.stdout));
}

/// The control: the same clone without the refs reads the file and sees a
/// proposal, which is less and never wrong.
#[test]
fn a_clone_without_the_refs_reads_the_file_and_serves_a_proposal() {
    let c = Corpus::new("ratify-ref-no-refs");
    c.accept(A);
    let clone = c.base.join("bare-clone");
    git(
        &c.base,
        &["clone", "-q", &file_url(&c.origin), clone.to_str().unwrap()],
    );
    let out = ank(&clone, &["context", "src/lib.rs", "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    let proposed: Vec<&str> = doc["proposed"]
        .as_sequence()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(proposed.contains(&A), "{doc:?}");
}

/// A ref nothing verifies does not bind: signed by a key `signers:` does not
/// declare, it is a fault, and the file's `proposed` is what is served.
#[test]
fn a_ref_signed_by_an_undeclared_key_is_a_fault_and_binds_nothing() {
    let c = Corpus::new("ratify-ref-undeclared");
    let other = c.base.join("other-key");
    let out = spawn("ssh-keygen")
        .args(["-t", "ed25519", "-N", "", "-C", "eve", "-q", "-f"])
        .arg(&other)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    c.sign_with(&c.repo, &other);
    c.accept(A);
    let clone = c.second_clone();

    let out = ank(&clone, &["check"]);
    let report = both(&out);
    assert_eq!(out.status.code(), Some(8), "{report}");
    assert!(
        faults(&report)
            .iter()
            .any(|l| l.contains(A) && l.contains("does not declare")),
        "{report}"
    );

    let out = ank(&clone, &["context", "src/lib.rs", "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    assert!(
        !doc["constraints"]
            .as_sequence()
            .unwrap()
            .iter()
            .any(|c| c["id"].as_str() == Some(A)),
        "an undeclared key ratifies nothing: {doc:?}"
    );
}

/// The four outcomes of §8 for a tag: declared is nothing, unsigned is a
/// fault, undeclared is a fault (above), and a signature this machine cannot
/// check is one signal for the corpus.
#[test]
fn a_tag_answers_the_four_outcomes_as_a_commit_does() {
    // Declared, and on the machine that ratified: nothing about the signature.
    let c = Corpus::new("ratify-ref-four-trusted");
    c.accept(A);
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert!(faults(&report).is_empty(), "{report}");
    assert!(!report.contains("not verified"), "{report}");

    // Unsigned, with a key declared: the anchor proves nothing.
    let c = Corpus::new("ratify-ref-four-unsigned");
    git(&c.repo, &["config", "--unset", "gpg.format"]);
    git(&c.repo, &["config", "--unset", "user.signingkey"]);
    c.accept(A);
    let tag = rev(&c.repo, &ratify_ref(A)).unwrap();
    assert!(!git(&c.repo, &["cat-file", "tag", &tag]).contains("-----BEGIN"));
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert_eq!(out.status.code(), Some(8), "{report}");
    assert!(
        faults(&report)
            .iter()
            .any(|l| l.contains(A) && l.contains("not signed")),
        "{report}"
    );

    // A signature present and no gpg to check it with: not verified, and not
    // refused, once for the corpus.
    let c = Corpus::new("ratify-ref-four-unchecked");
    c.accept(A);
    let tag = rev(&c.repo, &ratify_ref(A)).unwrap();
    let object = git(&c.repo, &["cat-file", "tag", &tag]);
    let unsigned = object.split("-----BEGIN").next().unwrap();
    let forged = format!(
        "{unsigned}-----BEGIN PGP SIGNATURE-----\n\niQEzBAABCAAdFiEEASNFZ4mrze8=\n=abcd\n-----END PGP SIGNATURE-----\n"
    );
    let path = c.base.join("forged-tag");
    fs::write(&path, forged).unwrap();
    let forged = git(
        &c.repo,
        &["hash-object", "-t", "tag", "-w", path.to_str().unwrap()],
    );
    git(&c.repo, &["update-ref", &ratify_ref(A), &forged]);
    // On the origin too: `check` fetches `refs/ank/*` before it reads them
    // (ADR-1b03), and would otherwise put the original tag back.
    git(&c.repo, &["push", "-q", "-f", "origin", &ratify_ref(A)]);
    git(&c.repo, &["config", "gpg.format", "openpgp"]);
    let nowhere = c.base.join("no-such-gpg");
    git(
        &c.repo,
        &["config", "gpg.program", nowhere.to_str().unwrap()],
    );
    let anchored = c.file(A).replace(&tag, &forged);
    c.put(A, &anchored);
    let config = fs::read_to_string(c.repo.join(".ank/config.yml")).unwrap();
    fs::write(
        c.repo.join(".ank/config.yml"),
        config.replace(&c.declared, "gpg 739A603FB05F9F2F7D3C8D50624FCFCC1482554A"),
    )
    .unwrap();
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert!(
        faults(&report).iter().all(|l| !l.contains(A)),
        "a signature nobody could check is never refused: {report}"
    );
    assert!(report.contains("not verified"), "{report}");
}

/// A key retired by `valid_before` ratifies nothing tagged after the date.
#[test]
fn a_tag_by_a_retired_key_after_its_valid_before_is_a_fault() {
    let c = Corpus::new("ratify-ref-retired");
    let config = fs::read_to_string(c.repo.join(".ank/config.yml")).unwrap();
    fs::write(
        c.repo.join(".ank/config.yml"),
        format!("{config}        valid_before: 2026-10-01\n"),
    )
    .unwrap();
    git(&c.repo, &["commit", "-qam", "retire"]);
    let out = spawn(ANK)
        .args(["accept", A])
        .env("ANK_AGENT", HUMAN)
        .env("GIT_COMMITTER_DATE", "2026-10-11T12:00:00+00:00")
        .current_dir(&c.repo)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", both(&out));

    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert_eq!(out.status.code(), Some(8), "{report}");
    assert!(
        faults(&report)
            .iter()
            .any(|l| l.contains(A) && l.contains("valid_before 2026-10-01")),
        "{report}"
    );
}

/// The freeze is read from the ref: an edited constraint on a file whose
/// ratification only a ref carries is a fault.
#[test]
fn an_altered_constraint_is_a_fault_against_the_ref() {
    let c = Corpus::new("ratify-ref-altered");
    c.accept(A);
    let edited = c.file(A).replace("The rule", "Another rule");
    c.put(A, &edited);
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert_eq!(out.status.code(), Some(8), "{report}");
    assert!(
        faults(&report)
            .iter()
            .any(|l| l.contains(A) && l.contains("altered since ratification")),
        "{report}"
    );
}

/// A file naming a tag the repository holds and no ref points at: a signal
/// naming the `update-ref` that restores it, and `ank update` restores it.
#[test]
fn a_held_anchor_no_ref_points_at_is_named_and_update_restores_it() {
    let c = Corpus::new("ratify-ref-restore");
    c.accept(A);
    let tag = rev(&c.repo, &ratify_ref(A)).unwrap();
    // Lost everywhere: `check` fetches `refs/ank/*` first (ADR-1b03), and a
    // ref the origin still holds comes back by that fetch alone.
    git(
        &c.repo,
        &["push", "-q", "origin", &format!(":{}", ratify_ref(A))],
    );
    git(&c.repo, &["update-ref", "-d", &ratify_ref(A)]);

    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert_eq!(out.status.code(), Some(0), "{report}");
    let restore = format!("git update-ref {} {tag}", ratify_ref(A));
    assert!(
        about(&report, A)
            .iter()
            .any(|l| l.starts_with("signal:") && l.contains(&restore)),
        "the signal names the command that restores the ref: {report}"
    );

    // `update` without `--check`: the release half refuses a binary under a
    // cargo target directory, and the restoration has already run.
    let out = ank(&c.repo, &["update"]);
    let said = both(&out);
    assert_eq!(rev(&c.repo, &ratify_ref(A)), Some(tag.clone()), "{said}");
    assert!(
        text(&out.stdout)
            .lines()
            .any(|l| l.contains("restored") && l.contains(&ratify_ref(A)) && l.contains(&tag)),
        "update says what it restored, on its own line: {said}"
    );
    let out = ank(&c.repo, &["check"]);
    assert!(!both(&out).contains(&restore), "{}", both(&out));
}

/// `update` restores only a tag whose subject is `ratify <id>`: a file naming
/// any other object the repository holds gets no ref.
#[test]
fn update_restores_nothing_but_a_ratification_of_that_entity() {
    let c = Corpus::new("ratify-ref-restore-only-ratify");
    c.accept(A);
    let tag = rev(&c.repo, &ratify_ref(A)).unwrap();
    git(&c.repo, &["update-ref", "-d", &ratify_ref(A)]);
    // The file of `C` claims `A`'s tag as its anchor.
    let claimed = c
        .file(C)
        .replace("schema: 4\n", &format!("anchor: {tag}\nschema: 4\n"));
    c.put(C, &claimed);
    git(&c.repo, &["update-ref", &ratify_ref(A), &tag]);

    let out = ank(&c.repo, &["update"]);
    assert_eq!(rev(&c.repo, &ratify_ref(C)), None, "{}", both(&out));
}

/// A ref whose entity has no file is an orphan, and said so.
#[test]
fn a_ref_with_no_file_is_an_orphan_signal() {
    let c = Corpus::new("ratify-ref-orphan");
    c.accept(A);
    fs::remove_file(c.repo.join(format!(".ank/entities/{A}.md"))).unwrap();
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    assert!(
        report.lines().any(|l| l.starts_with("signal:")
            && l.contains(&ratify_ref(A))
            && l.contains("no entity file")),
        "{report}"
    );
}

/// A file marked superseded that no ratification names as retired is a
/// signal; the supersession `accept` recorded in the tag is the control.
#[test]
fn a_superseded_file_no_ref_names_is_a_signal() {
    let c = Corpus::new("ratify-ref-superseded-unnamed");
    let marked = c.file(B).replace("status: accepted", "status: superseded");
    c.put(B, &marked);
    let out = ank(&c.repo, &["check"]);
    let report = both(&out);
    let named = |report: &str| {
        report.lines().any(|l| {
            l.starts_with("signal:") && l.contains(B) && l.contains("no ratification names")
        })
    };
    assert!(named(&report), "{report}");

    let c = Corpus::new("ratify-ref-superseded-named");
    c.accept(C);
    assert!(c.file(B).contains("status: superseded"));
    let out = ank(&c.repo, &["check"]);
    assert!(!named(&both(&out)), "{}", both(&out));
}

/// `show`, `find` and `review` say which of the two carried the anchor.
#[test]
fn show_find_and_review_say_whether_a_ref_or_a_commit_carried_it() {
    let c = Corpus::new("ratify-ref-surfaces");
    c.accept(A);
    // `C` ratified the way `accept` did before the decision: a commit, and
    // its ref gone.
    c.accept(C);
    let tag = rev(&c.repo, &ratify_ref(C)).unwrap();
    let message = git(&c.repo, &["cat-file", "tag", &tag]);
    let message = message
        .split_once("\n\n")
        .unwrap()
        .1
        .split("-----BEGIN")
        .next()
        .unwrap()
        .to_string();
    git(
        &c.repo,
        &["push", "-q", "origin", &format!(":{}", ratify_ref(C))],
    );
    git(&c.repo, &["update-ref", "-d", &ratify_ref(C)]);
    let file = c
        .file(C)
        .lines()
        .filter(|l| !l.starts_with("anchor: "))
        .map(|l| format!("{l}\n"))
        .collect::<String>();
    c.put(C, &file);
    git(&c.repo, &["add", "-A", ".ank"]);
    git(&c.repo, &["commit", "-q", "-S", "-m", &message]);

    let out = ank(&c.repo, &["show", A]);
    assert!(
        text(&out.stdout).contains(&format!("RATIFIED BY {}", ratify_ref(A))),
        "{}",
        both(&out)
    );
    let out = ank(&c.repo, &["show", C]);
    assert!(
        text(&out.stdout).contains("RATIFIED BY commit "),
        "{}",
        both(&out)
    );
    let out = ank(&c.repo, &["show", A, "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    assert_eq!(doc["ratification"]["via"].as_str(), Some("ref"), "{doc:?}");

    let out = ank(&c.repo, &["find", "--type", "adr"]);
    let listing = text(&out.stdout);
    let line = |id: &str| {
        listing
            .lines()
            .find(|l| l.contains(&id[..15]))
            .unwrap_or_default()
            .to_string()
    };
    assert!(line(A).contains("(anchor: ref)"), "{listing}");
    assert!(line(C).contains("(anchor: commit)"), "{listing}");
    let out = ank(&c.repo, &["find", "--type", "adr", "--json"]);
    let doc: serde_yaml::Value = serde_yaml::from_slice(&out.stdout).expect(&both(&out));
    let by = |id: &str| {
        doc["results"]
            .as_sequence()
            .unwrap()
            .iter()
            .find(|r| r["id"].as_str() == Some(id))
            .unwrap()["anchored_by"]
            .clone()
    };
    assert_eq!(by(A).as_str(), Some("ref"), "{doc:?}");
    assert_eq!(by(C).as_str(), Some("commit"), "{doc:?}");

    let out = ank(&c.repo, &["review"]);
    let listing = text(&out.stdout);
    assert!(
        listing
            .lines()
            .any(|l| l.contains(A) && l.contains("anchor: ref")),
        "{listing}"
    );
    assert!(
        listing
            .lines()
            .any(|l| l.contains(C) && l.contains("anchor: commit")),
        "{listing}"
    );
}

/// The ref plane costs the same processes whatever the number of
/// ratifications: the refs come from the one listing of `refs/ank/*` the
/// coordination plane already makes, the tags from one `cat-file --batch`, and
/// a verdict reached once is not asked of git again.
#[test]
fn check_reads_every_ref_in_one_listing_and_one_batch_and_caches_verdicts() {
    let c = Corpus::new("ratify-ref-processes");
    for n in 0..3 {
        let id = format!("ADR-0000000006d{n}");
        c.put(&id, &adr(&id, "proposed", ""));
    }
    git(&c.repo, &["add", "-A"]);
    git(&c.repo, &["commit", "-qm", "three more"]);
    c.accept(A);

    let run = |name: &str| -> Vec<String> {
        let trace = scratch::dir(name).join("trace");
        let out = ank_env(&c.repo, &["check"], &[("GIT_TRACE", trace.as_path())]);
        assert!(faults(&both(&out)).is_empty(), "{}", both(&out));
        fs::read_to_string(&trace)
            .unwrap_or_default()
            .lines()
            .filter_map(|l| l.split_once(MARK).map(|(_, a)| a.to_string()))
            .collect()
    };
    let others = |trace: &[String]| -> Vec<String> {
        trace
            .iter()
            .filter(|g| !g.contains("verify-tag"))
            .cloned()
            .collect()
    };
    let verified = |trace: &[String]| trace.iter().filter(|g| g.contains("verify-tag")).count();

    // Warm: the first `check` adds the refspec and fetches, and judges `A`.
    let first = run("ratify-ref-trace-0");
    assert_eq!(verified(&first), 1, "{first:#?}");
    let one = run("ratify-ref-trace-1");
    assert_eq!(
        verified(&one),
        0,
        "a verdict is cached by object name: {one:#?}"
    );

    for n in 0..3 {
        c.accept(&format!("ADR-0000000006d{n}"));
    }
    let four = run("ratify-ref-trace-4");
    assert_eq!(
        verified(&four),
        3,
        "the three new tags and not A: {four:#?}"
    );
    assert_eq!(
        others(&four),
        others(&one),
        "four ratifications start the processes one did"
    );
    assert!(
        !four
            .iter()
            .any(|g| g.starts_with("cat-file tag") || g.contains("refs/ank/ratify/")),
        "no tag is read on its own: {four:#?}"
    );
    let again = run("ratify-ref-trace-4-again");
    assert_eq!(verified(&again), 0, "{again:#?}");
}

/// What a hub of corpora consumes: the files at one commit and `refs/ank/*`,
/// no history. A ratification carried by a ref travels whole and is judged
/// there; one carried by a commit the clone does not hold cannot be, and says
/// so as a signal rather than calling the freeze broken (TASK-8afa3d0a6e25).
#[test]
fn a_shallow_clone_verifies_a_ref_ratification_and_cannot_verify_a_commit_one() {
    let c = Corpus::new("ratify-ref-shallow");
    c.accept(A);
    // `C` the way `accept` ratified before ADR-5e47007a75be: a signed commit
    // and no ref. One commit after it, so depth 1 leaves it out.
    c.accept(C);
    crate::legacy_anchor::commit(&|a| git(&c.repo, a), C, true);
    fs::write(c.repo.join("src/lib.rs"), "pub fn f() {}\npub fn g() {}\n").unwrap();
    git(&c.repo, &["commit", "-qam", "after the ratification"]);
    git(&c.repo, &["push", "-q", "origin", "main"]);
    git(&c.repo, &["push", "-q", "origin", "refs/ank/*:refs/ank/*"]);

    let clone = c.base.join("shallow");
    git(
        &c.base,
        &[
            "clone",
            "-q",
            "--depth",
            "1",
            &file_url(&c.origin),
            clone.to_str().unwrap(),
        ],
    );
    git(&clone, &["fetch", "-q", "origin", "+refs/ank/*:refs/ank/*"]);
    assert_eq!(
        git(&clone, &["rev-parse", "--is-shallow-repository"]),
        "true",
        "fetching the refs must not have deepened the clone"
    );
    assert_eq!(git(&clone, &["rev-list", "--count", "HEAD"]), "1");
    assert!(rev(&clone, &ratify_ref(A)).is_some(), "the ref travelled");
    assert!(rev(&clone, &ratify_ref(C)).is_none(), "C has no ref");

    let trace = scratch::dir("ratify-ref-shallow-trace").join("trace");
    let out = ank_env(&clone, &["check"], &[("GIT_TRACE", trace.as_path())]);
    let report = both(&out);
    assert_eq!(
        out.status.code(),
        Some(0),
        "signals, never a fault: {report}"
    );
    assert!(faults(&report).is_empty(), "{report}");

    // The ref: its tag's signature judged against `signers:`, and nothing
    // said about its anchor or its signature, which is what a trusted
    // ratification looks like. The one line left is about who ratified,
    // which the fixture makes the author on purpose.
    let judged = fs::read_to_string(&trace)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once(MARK).map(|(_, a)| a.to_string()))
        .filter(|g| g.contains("verify-tag"))
        .count();
    assert_eq!(judged, 1, "the tag of {A} is judged, once: {report}");
    assert_eq!(
        about(&report, A),
        [format!("signal: {A}: ratified by its own author ({HUMAN})")],
        "{report}"
    );
    assert!(!report.contains("not verified"), "{report}");

    // The commit: out of reach, so unverifiable, and never altered.
    let lines = about(&report, C);
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("signal:") && l.contains("no ratification commit is reachable")),
        "{report}"
    );
    assert!(
        !lines.iter().any(|l| l.contains("altered")),
        "a shallow clone is not a broken freeze: {report}"
    );
}
