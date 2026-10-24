//! `ank accept <id> --rescope` re-signs the scope of an accepted ADR or spec
//! whose constraint, or body, did not change, and `check` verifies the chain
//! the new tag carries (TASK-14b7c08503cb, ADR-bb9ccdba9be7).
//!
//! Every case through the binary, on scratch repositories with a bare origin
//! reached by `file://`: the push with compare-and-swap is half of the act.
//! The scope moves because the code did, so each corpus scopes two files and
//! `git mv`s one of them; the ADR and the spec go through the same steps.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "human:marie";

/// Long enough that git's similarity index recognises it after a move.
const BODY: &str = "pub fn old() {}\n// one\n// two\n// three\n// four\n// five\n// six\n";

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("accept-rescope-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n\
             [user]\n\tname = t\n\temail = t@example.com\n\
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
        .env("GIT_CONFIG_SYSTEM", config)
        .env_remove("EDITOR")
        .env_remove("VISUAL")
        .env("ANK_AGENT", AGENT);
    let root = scratch::root();
    c.env("TMPDIR", root).env("TMP", root).env("TEMP", root);
    c
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_str().unwrap().replace('\\', "/"))
}

fn git_in(dir: &Path, args: &[&str]) -> String {
    let out = spawn("git").args(args).current_dir(dir).output().unwrap();
    assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
    text(&out.stdout).trim().to_string()
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
fn field(file: &str, key: &str) -> Option<String> {
    file.lines()
        .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        .map(|v| v.trim().to_string())
}

/// One finding `check` printed: its level, its sentence and its note.
#[derive(Debug, PartialEq)]
struct Finding {
    level: String,
    message: String,
    note: Vec<String>,
}

/// The findings `check` printed about `id`.
fn findings(printed: &str, id: &str) -> Vec<Finding> {
    let mut found: Vec<Finding> = Vec::new();
    let mut ours = false;
    for line in printed.lines() {
        if let Some(note) = line
            .strip_prefix("└── ")
            .or_else(|| line.strip_prefix("    "))
        {
            if ours {
                found.last_mut().unwrap().note.push(note.to_string());
            }
            continue;
        }
        let mut parts = line.splitn(3, ": ");
        let (Some(level), Some(subject), Some(message)) =
            (parts.next(), parts.next(), parts.next())
        else {
            ours = false;
            continue;
        };
        // The fixture ratifies what it wrote, under one identity: that signal
        // is about who signed, and no case here is.
        ours = subject == id
            && matches!(level, "signal" | "error")
            && !message.starts_with("ratified by its own author");
        if ours {
            // `check` prints a fault as `error`, and the cases name it fault.
            found.push(Finding {
                level: if level == "error" { "fault" } else { level }.to_string(),
                message: message.to_string(),
                note: Vec::new(),
            });
        }
    }
    found
}

struct Corpus {
    repo: PathBuf,
    origin: PathBuf,
    signed: bool,
}

impl Corpus {
    /// A repository with an origin, `src/old.rs` and `src/keep.rs`, and under
    /// `signed` a throwaway ssh key git signs with and `signers:` declares.
    fn new(what: &str, signed: bool) -> Corpus {
        let base = scratch::dir(what);
        let repo = base.join("repo");
        fs::create_dir_all(repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(repo.join("src")).unwrap();
        git_in(&repo, &["init", "-q", "-b", "main"]);
        fs::write(
            repo.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        fs::write(repo.join(".gitignore"), ".ank/index.db*\n").unwrap();
        fs::write(repo.join("src/old.rs"), BODY).unwrap();
        fs::write(repo.join("src/keep.rs"), "pub fn keep() {}\n").unwrap();
        let origin = base.join("origin.git");
        fs::create_dir_all(&origin).unwrap();
        git_in(&origin, &["init", "-q", "--bare", "-b", "main"]);
        git_in(&repo, &["remote", "add", "origin", &file_url(&origin)]);
        let c = Corpus {
            repo,
            origin,
            signed,
        };
        if signed {
            let key = base.join("signing-key");
            let out = spawn("ssh-keygen")
                .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
                .arg(&key)
                .output()
                .expect("ssh-keygen ships with git on all three platforms");
            assert!(out.status.success(), "ssh-keygen: {}", text(&out.stderr));
            c.git(&["config", "gpg.format", "ssh"]);
            c.git(&[
                "config",
                "user.signingkey",
                key.with_extension("pub").to_str().unwrap(),
            ]);
            c.ok(&["signers", "add"]);
        }
        c.commit("seed");
        c.git(&["push", "-q", "-u", "origin", "main"]);
        c
    }

    fn git(&self, args: &[&str]) -> String {
        git_in(&self.repo, args)
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", message, "--allow-empty"]);
    }

    fn ank(&self, args: &[&str]) -> Output {
        spawn(ANK)
            .args(args)
            .current_dir(&self.repo)
            .output()
            .unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.ank(args);
        assert_eq!(
            code(&out),
            0,
            "ank {args:?}: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        text(&out.stdout)
    }

    fn created(&self, args: &[&str]) -> String {
        self.ok(args)
            .split_whitespace()
            .nth(1)
            .expect("created <id> <slug>")
            .to_string()
    }

    /// An accepted ADR and an accepted spec, each scoping `src/old.rs` and
    /// `src/keep.rs`, committed and ratified.
    fn ratified(&self) -> [String; 2] {
        let scopes = ["--scope", "src/old.rs", "--scope", "src/keep.rs"];
        let mut adr = vec![
            "new",
            "adr",
            "--title",
            "A rule",
            "--constraint",
            "A binding rule.",
        ];
        adr.extend(scopes);
        let mut spec = vec!["new", "spec", "--title", "A document"];
        spec.extend(scopes);
        let ids = [self.created(&adr), self.created(&spec)];
        self.commit("propose");
        for id in &ids {
            self.ok(&["accept", id]);
        }
        self.commit("ratify");
        ids
    }

    fn file(&self, id: &str) -> String {
        fs::read_to_string(self.path(id)).unwrap()
    }

    fn path(&self, id: &str) -> PathBuf {
        self.repo.join(".ank/entities").join(format!("{id}.md"))
    }

    /// Everything a refused call must leave as it found it: the entity
    /// files, and every ref under `refs/ank` here and on the origin.
    fn written(&self, ids: &[String]) -> (Vec<String>, String, String) {
        (
            ids.iter().map(|id| self.file(id)).collect(),
            self.git(&["for-each-ref", "refs/ank"]),
            git_in(&self.origin, &["for-each-ref", "refs/ank"]),
        )
    }

    /// `check`: its exit code and the findings about `id`.
    fn check(&self, id: &str) -> (i32, Vec<Finding>) {
        let out = self.ank(&["check"]);
        assert!(
            [0, 8].contains(&code(&out)),
            "check: {}{}",
            text(&out.stdout),
            text(&out.stderr)
        );
        (code(&out), findings(&text(&out.stdout), id))
    }
}

/// The amend and the rescope, end to end, in both signing regimes.
#[test]
fn rescope_moves_the_perimeter_and_check_verifies_the_chain() {
    for signed in [false, true] {
        let c = Corpus::new(
            if signed {
                "accept-rescope-signed"
            } else {
                "accept-rescope-unsigned"
            },
            signed,
        );
        let ids = c.ratified();
        c.git(&["mv", "src/old.rs", "src/new.rs"]);
        c.commit("move it");
        let moved = c.git(&["rev-parse", "--short=12", "HEAD"]);

        for id in &ids {
            let what = format!("{id} (signed: {})", c.signed);

            // The rename proposes the rescope, and no supersession.
            let (exit, found) = c.check(id);
            assert_eq!(exit, 0, "{what}: a rename git records is no fault");
            assert_eq!(
                found,
                vec![Finding {
                    level: "signal".into(),
                    message: "dead scope 'src/old.rs': no file matches it".into(),
                    note: vec![
                        format!("git records src/old.rs renamed to src/new.rs in {moved}"),
                        format!(
                            "ank amend {id} --drop-scope \"src/old.rs\" --scope \"src/new.rs\", \
                             then ank accept {id} --rescope"
                        ),
                    ],
                }],
                "{what}"
            );

            // amend edits the accepted scope, and check says the file has not
            // caught up with its ratification.
            let before = c.file(id);
            let previous_anchor = field(&before, "ratified").expect("ratified");
            let previous_tag = rev(&c.repo, &ratify_ref(id)).expect("the ratify ref");
            let out = c.ank(&[
                "amend",
                id,
                "--drop-scope",
                "src/old.rs",
                "--scope",
                "src/new.rs",
            ]);
            assert_eq!(
                code(&out),
                0,
                "{what}: amend takes an accepted scope: {}",
                text(&out.stderr)
            );
            let (exit, found) = c.check(id);
            assert_eq!(exit, 0, "{what}: an amended scope is not a fault");
            let file = format!(".ank/entities/{id}.md");
            assert_eq!(
                found,
                vec![Finding {
                    level: "signal".into(),
                    message: format!(
                        "its scope was amended after its ratification, and {file} has not \
                         caught up with it: ank accept {id} --rescope"
                    ),
                    note: Vec::new(),
                }],
                "{what}"
            );

            // The rescope replaces the ref, here and on the origin.
            let out = c.ank(&["accept", id, "--rescope"]);
            assert_eq!(
                code(&out),
                0,
                "{what}: {}{}",
                text(&out.stdout),
                text(&out.stderr)
            );
            let tag = rev(&c.repo, &ratify_ref(id)).expect("the ratify ref");
            assert_ne!(tag, previous_tag, "{what}: a new tag");
            assert_eq!(
                text(&out.stdout),
                format!(
                    "rescoped {id} -> {} (from {})\n",
                    &tag[..7],
                    &previous_tag[..7]
                ),
                "{what}"
            );
            assert_eq!(
                rev(&c.origin, &ratify_ref(id)),
                Some(tag.clone()),
                "{what}: the ref reached the origin"
            );
            let after = c.file(id);
            let anchor = field(&after, "ratified").expect("ratified");
            assert_ne!(anchor, previous_anchor, "{what}: the anchor moved");
            assert_eq!(field(&after, "anchor"), Some(tag.clone()), "{what}");
            assert_eq!(field(&after, "status").as_deref(), Some("accepted"));
            assert_eq!(field(&after, "id").as_deref(), Some(id.as_str()));

            let object = c.git(&["cat-file", "tag", &tag]);
            let key = if id.starts_with("ADR-") {
                "constraint+scope"
            } else {
                "body+scope"
            };
            let message = object.split_once("\n\n").expect(&object).1;
            assert!(
                message.starts_with(&format!(
                    "ratify {id}\n\n{key}: {anchor}\nprevious: {previous_anchor}\n\
                     previous-scope: src/old.rs\nprevious-scope: src/keep.rs\nby: {AGENT}"
                )),
                "{what}: {object}"
            );
            assert_eq!(
                object.contains("-----BEGIN SSH SIGNATURE-----"),
                c.signed,
                "{what}: signed where the repository signs: {object}"
            );

            // And check finds nothing to say about it.
            let (exit, found) = c.check(id);
            assert_eq!(exit, 0, "{what}: {found:?}");
            assert_eq!(found, Vec::new(), "{what}");
        }
    }
}

/// Each refusal writes nothing: not the file, not a ref, here or there.
#[test]
fn rescope_refuses_with_nothing_written() {
    let c = Corpus::new("accept-rescope-refusals", false);
    let ids = c.ratified();
    let [adr, spec] = ids.clone();
    let proposed = c.created(&[
        "new",
        "adr",
        "--title",
        "Not yet",
        "--constraint",
        "A rule nobody signed.",
        "--scope",
        "src/keep.rs",
    ]);
    c.commit("propose another");
    let every = [adr.clone(), spec.clone(), proposed.clone()];

    let refused = |args: &[&str], exit: i32, sentence: &str| {
        let before = c.written(&every);
        let out = c.ank(args);
        assert_eq!(code(&out), exit, "{args:?}: {}", text(&out.stderr));
        assert!(
            text(&out.stderr).contains(sentence),
            "{args:?}: {}",
            text(&out.stderr)
        );
        assert_eq!(c.written(&every), before, "{args:?} wrote something");
    };

    // Nothing to rescope: the scope is the one ratified.
    for id in [&adr, &spec] {
        refused(
            &["accept", id, "--rescope"],
            6,
            &format!(
                "{id} declares the scope its ratification carries: there is nothing to rescope"
            ),
        );
    }

    // Not accepted.
    refused(
        &["accept", &proposed, "--rescope"],
        6,
        &format!("{proposed} is proposed, and only an accepted document has a scope to rescope"),
    );

    // The decision itself changed: the constraint, or a spec's body.
    for id in [&adr, &spec] {
        c.ok(&["amend", id, "--scope", "src/new.rs"]);
    }
    let edit = |id: &str, from: &str, to: &str| {
        let text = c.file(id);
        assert!(text.contains(from), "{id}: {text}");
        fs::write(c.path(id), text.replacen(from, to, 1)).unwrap();
    };
    edit(&adr, "A binding rule.", "A different rule.");
    let body = c.file(&spec);
    fs::write(
        c.path(&spec),
        format!("{body}\nA paragraph added after the signature.\n"),
    )
    .unwrap();
    refused(
        &["accept", &adr, "--rescope"],
        6,
        &format!(
            "{adr}'s constraint changed since its ratification: a rescope only moves the perimeter"
        ),
    );
    refused(
        &["accept", &spec, "--rescope"],
        6,
        &format!(
            "{spec}'s body changed since its ratification: a rescope only moves the perimeter"
        ),
    );
}

/// A tag whose previous anchor is not the current constraint with the
/// previous scope is a fault, even when its new anchor matches the file.
#[test]
fn check_faults_on_a_forged_previous_anchor() {
    let c = Corpus::new("accept-rescope-forged", false);
    let ids = c.ratified();
    for id in &ids {
        c.ok(&["amend", id, "--scope", "src/new.rs"]);
        fs::write(c.repo.join("src/new.rs"), "pub fn new() {}\n").unwrap();
        c.ok(&["accept", id, "--rescope"]);
    }
    for id in &ids {
        let (exit, found) = c.check(id);
        assert_eq!((exit, found), (0, Vec::new()), "{id}: the honest chain");
    }
    for id in &ids {
        // The same blob and the same message, with the previous anchor
        // swapped: everything else about the tag still holds.
        let tag = rev(&c.repo, &ratify_ref(id)).unwrap();
        let object = c.git(&["cat-file", "tag", &tag]);
        let blob = object
            .lines()
            .find_map(|l| l.strip_prefix("object "))
            .unwrap()
            .to_string();
        let message = object.split_once("\n\n").unwrap().1;
        let previous = message
            .lines()
            .find_map(|l| l.strip_prefix("previous: "))
            .unwrap()
            .to_string();
        let forged_message = message.replace(&previous, "0123456789ab");
        c.git(&["tag", "-a", "-m", &forged_message, "forged", &blob]);
        let forged = c.git(&["rev-parse", "refs/tags/forged"]);
        c.git(&["update-ref", "-d", "refs/tags/forged"]);
        c.git(&["update-ref", &ratify_ref(id), &forged, &tag]);
        let file = c.file(id).replace(&tag, &forged);
        fs::write(c.path(id), file).unwrap();

        let (exit, found) = c.check(id);
        assert_eq!(exit, 8, "{id}: {found:?}");
        let what = if id.starts_with("ADR-") {
            "constraint"
        } else {
            "body"
        };
        assert!(
            found.contains(&Finding {
                level: "fault".into(),
                message: format!(
                    "its rescope {} names the previous anchor 0123456789ab, but its {what} with \
                     the previous scope hashes to {previous}: the decision changed under a rescope",
                    &forged[..12]
                ),
                note: Vec::new(),
            }),
            "{id}: {found:?}"
        );
    }
}

/// The page names the flag, as the golden does.
#[test]
fn help_accept_names_rescope() {
    let c = Corpus::new("accept-rescope-help", false);
    let out = c.ok(&["help", "accept"]);
    assert!(out.contains("--rescope"), "{out}");
    assert!(
        out.contains("--rescope ratifies the scope an accepted ADR or spec now declares"),
        "{out}"
    );
}

/// The swap is checked on the remote: an origin whose ref moved since this
/// clone read it refuses the rescope, and nothing is kept here either.
#[test]
fn rescope_pushes_with_compare_and_swap() {
    let c = Corpus::new("accept-rescope-swap", false);
    let ids = c.ratified();
    let id = &ids[0];
    fs::write(c.repo.join("src/new.rs"), "pub fn new() {}\n").unwrap();
    c.ok(&["amend", id, "--scope", "src/new.rs"]);
    // Another clone's rescope, as the origin would hold it: any object other
    // than the tag this clone read.
    let elsewhere = git_in(&c.origin, &["rev-parse", "main"]);
    git_in(&c.origin, &["update-ref", &ratify_ref(id), &elsewhere]);

    let before = c.written(&ids);
    let out = c.ank(&["accept", id, "--rescope"]);
    assert_eq!(code(&out), 9, "{}", text(&out.stderr));
    assert!(
        text(&out.stderr).contains(&format!(
            "{} did not reach origin, and nothing was kept: {id} is as it was (the remote \
             already holds {elsewhere} there)",
            ratify_ref(id)
        )),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(c.written(&ids), before, "nothing kept");
}

/// An anchor made by a `ratify <id>` commit has no ref to replace: amend
/// refuses its scope and names `--reanchor`, and the rename proposes the three
/// commands in the order that works, which run as printed.
#[test]
fn a_commit_anchor_is_reanchored_before_its_rescope() {
    let c = Corpus::new("accept-rescope-commit-anchor", false);
    let ids = c.ratified();
    for id in &ids {
        crate::legacy_anchor::commit(&|a| c.git(a), id, false);
    }
    c.git(&["mv", "src/old.rs", "src/new.rs"]);
    c.commit("move it");

    for id in &ids {
        let out = c.ank(&["amend", id, "--drop-scope", "src/old.rs"]);
        assert_eq!(code(&out), 6, "{id}: {}", text(&out.stderr));
        assert!(
            text(&out.stderr).contains(&format!(
                "{id} is ratified by a ratify {id} commit and no ref, and only a ref's scope is \
                 rescoped"
            )),
            "{id}: {}",
            text(&out.stderr)
        );

        let (_, found) = c.check(id);
        let amend = format!("ank amend {id} --drop-scope \"src/old.rs\" --scope \"src/new.rs\"");
        let proposed =
            format!("ank accept {id} --reanchor, then {amend}, then ank accept {id} --rescope");
        assert_eq!(
            found.first().and_then(|f| f.note.get(1)),
            Some(&proposed),
            "{id}: {found:?}"
        );

        c.ok(&["accept", id, "--reanchor"]);
        c.ok(&[
            "amend",
            id,
            "--drop-scope",
            "src/old.rs",
            "--scope",
            "src/new.rs",
        ]);
        c.ok(&["accept", id, "--rescope"]);
        let (exit, found) = c.check(id);
        assert_eq!(
            (exit, found),
            (0, Vec::new()),
            "{id}: the proposal ran clean"
        );
    }
}
