//! A reader overrides a peer's declared path in their own `corpora.yml`, and a
//! binding holds on a layout that differs (issue #500, ADR-da2819aef598,
//! TASK-e95c729e5d43).
//!
//! `config.yml` declares a peer by a path relative to the declaring root, which
//! is the layout every clone is expected to follow. On a machine where the two
//! checkouts are not siblings, the peer "is not a corpus" and the binding
//! vanishes. The override is keyed by the declaring corpus's identity and the
//! peer's name, lives in the reader's `corpora.yml`, and is written by
//! `ank config --user peers.<identity>.<name> <path>`.
//!
//! **Both repositories are misplaced, on purpose.** A binding is resolved twice
//! (`Peer::binds`): `b` opens `a` through `b`'s declaration `aa`, and `a`'s scope
//! entry `bb:**` is resolved through `a`'s declaration `bb` back to `b`. An
//! override that fed only the first would leave the second comparing against a
//! path that does not exist here, and the binding would still vanish. So each
//! override is set alone first, and the binding must stay absent until both are.
//!
//! **Through the binary**: the reader's home is addressed through the
//! environment, which only a child process can vary.

mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-override";

/// In `a`, scoped `bb:**`: binds every path of `b`.
const BOUND: &str = "ADR-dddddddddddd";

/// A reader: one home, holding the `corpora.yml` the overrides are written to.
struct Reader {
    home: PathBuf,
}

impl Reader {
    fn run(&self, dir: &Path, args: &[&str]) -> Output {
        let root = scratch::root();
        let home = self.home.to_string_lossy().into_owned();
        Command::new(ANK)
            .args(args)
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env("NO_COLOR", "1")
            // Both, on every platform: `user_dir` reads `APPDATA` on Windows and
            // `XDG_CONFIG_HOME` elsewhere.
            .env("XDG_CONFIG_HOME", &home)
            .env("APPDATA", &home)
            .output()
            .expect("the binary runs")
    }

    /// The answer of a run that must succeed, and every line either stream
    /// carried: `context` prints its warnings above the answer, on stdout.
    fn ok(&self, dir: &Path, args: &[&str]) -> (String, String) {
        let out = self.run(dir, args);
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert!(out.status.success(), "ank {args:?}: {stderr}");
        (stdout.clone(), format!("{stdout}{stderr}"))
    }

    fn corpora(&self) -> String {
        fs::read_to_string(self.home.join("ank").join("corpora.yml")).unwrap_or_default()
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
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

/// A corpus with one commit: the identity an override is keyed on is the root
/// commit (ADR-621a7fd96ce1), so a tree with no history could not be named.
fn corpus(reader: &Reader, dir: &Path, peer: &str, declared: &str, adr: Option<&str>) {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    reader.ok(dir, &["init"]);
    // Named, so that the only warning left in the answer is the peer's.
    reader.ok(dir, &["config", "default_branch", "main"]);
    reader.ok(dir, &["config", &format!("peers.{peer}"), declared]);
    if let Some(glob) = adr {
        fs::write(
            dir.join(format!(".ank/entities/{BOUND}.md")),
            format!(
                "---\nid: {BOUND}\ntype: adr\nslug: s\ntitle: Scoped {glob}\n\
                 created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - '{glob}'\n\
                 constraint: |\n  A rule.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }
    git(dir, &["add", "-A"]);
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
            "-qm",
            "init",
        ],
    );
}

fn warnings(stderr: &str) -> Vec<&str> {
    stderr
        .lines()
        .filter(|l| l.starts_with("warning:"))
        .collect()
}

#[test]
fn an_override_at_both_ends_makes_a_binding_hold_on_a_layout_that_differs() {
    let base = scratch::dir("peer-override");
    let reader = Reader {
        home: base.join("home"),
    };
    // `a` expects `b` beside it; `b` lives two levels away and expects `a`
    // beside *it*, which is wrong here as well.
    let a = base.join("a");
    let b = base.join("far").join("away").join("b");
    corpus(&reader, &a, "bb", "../b", Some("bb:**"));
    corpus(&reader, &b, "aa", "../a", None);
    let id_a = git(&a, &["rev-list", "--max-parents=0", "HEAD"]);
    let id_b = git(&b, &["rev-list", "--max-parents=0", "HEAD"]);
    let config_a = fs::read(a.join(".ank/config.yml")).unwrap();
    let config_b = fs::read(b.join(".ank/config.yml")).unwrap();
    let bound = format!("{BOUND}@aa");

    // The layout as it is: one warning each, and the local answer.
    let local_b = || {
        let (out, err) = reader.ok(&b, &["context", "README.md"]);
        assert!(!out.contains(BOUND), "no binding without overrides:\n{out}");
        let w = warnings(&err);
        assert_eq!(w.len(), 1, "{err}");
        assert!(w[0].contains("peer 'aa'"), "{err}");
        assert!(w[0].contains("not a corpus"), "{err}");
    };
    let local_a = || {
        let (out, err) = reader.ok(&a, &["context", "bb:README.md"]);
        assert!(out.contains("ADR-dddd"), "a's own ADR is local:\n{out}");
        let w = warnings(&err);
        assert_eq!(w.len(), 1, "{err}");
        assert!(w[0].contains("peer 'bb'"), "{err}");
    };
    local_b();
    local_a();

    // The warning names the verb that settles it without touching config.yml.
    let (_, err) = reader.ok(&b, &["context", "README.md"]);
    assert!(
        err.contains(&format!("ank config --user peers.{id_b}.aa <path>")),
        "{err}"
    );

    let key_a = format!("peers.{id_a}.bb");
    let key_b = format!("peers.{id_b}.aa");
    let path_a = a.to_string_lossy().into_owned();
    let path_b = b.to_string_lossy().into_owned();

    // Only the reader's opening: `b` reaches `a`, but `a`'s `bb` still resolves
    // to a path that does not exist here, so the binding is still absent.
    reader.ok(&b, &["config", "--user", &key_b, &path_a]);
    let (out, err) = reader.ok(&b, &["context", "README.md"]);
    assert!(warnings(&err).is_empty(), "b opens a:\n{err}");
    assert!(
        !out.contains(&bound),
        "the back-resolution still misses:\n{out}"
    );

    // Only the peer's back-resolution: `a`'s `bb` names `b`, but `b` cannot
    // open `a`, so nothing is served either.
    reader.ok(&b, &["config", "--user", "--unset", &key_b]);
    reader.ok(&a, &["config", "--user", &key_a, &path_b]);
    let (out, _) = reader.ok(&b, &["context", "README.md"]);
    assert!(!out.contains(&bound), "b cannot open a:\n{out}");

    // Both: the binding holds, in both directions.
    reader.ok(&b, &["config", "--user", &key_b, &path_a]);
    let (out, err) = reader.ok(&b, &["context", "README.md"]);
    assert!(out.contains(&bound), "b serves {bound}:\n{out}\n{err}");
    assert!(warnings(&err).is_empty(), "{err}");
    let (out, err) = reader.ok(&a, &["context", "bb:README.md"]);
    assert!(out.contains("ADR-dddd"), "a serves it too:\n{out}");
    assert!(warnings(&err).is_empty(), "a opens b:\n{err}");

    // Keyed on identity, and held in the reader's file alone.
    let corpora = reader.corpora();
    assert!(
        corpora.contains(&id_a) && corpora.contains(&id_b),
        "{corpora}"
    );
    assert_eq!(fs::read(a.join(".ank/config.yml")).unwrap(), config_a);
    assert_eq!(fs::read(b.join(".ank/config.yml")).unwrap(), config_b);
    assert_eq!(git(&a, &["status", "--porcelain"]), "");
    assert_eq!(git(&b, &["status", "--porcelain"]), "");

    // The overrides read back, and removed they bring the local answer back.
    let (read, _) = reader.ok(&b, &["config", "--user", &key_b]);
    assert_eq!(read.trim(), path_a);
    reader.ok(&a, &["config", "--user", "--unset", &key_a]);
    reader.ok(&b, &["config", "--user", "--unset", &key_b]);
    local_b();
    local_a();
    assert_eq!(git(&a, &["status", "--porcelain"]), "");
    assert_eq!(git(&b, &["status", "--porcelain"]), "");

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn an_override_key_that_names_no_identity_or_no_peer_is_refused() {
    let base = scratch::dir("peer-override-keys");
    let reader = Reader {
        home: base.join("home"),
    };
    let dir = base.join("r");
    fs::create_dir_all(&dir).unwrap();
    for key in [
        "peers.front.bb",
        "peers.0123456789abcdef0123456789abcdef01234567",
        "peers.0123456789abcdef0123456789abcdef01234567.b",
        "peers.0123456789abcdef0123456789abcdef01234567.bb.x",
    ] {
        let out = reader.run(&dir, &["config", "--user", key, "/x"]);
        assert!(!out.status.success(), "{key} accepted");
        assert_eq!(reader.corpora(), "", "{key} wrote");
    }
    let _ = fs::remove_dir_all(&base);
}
