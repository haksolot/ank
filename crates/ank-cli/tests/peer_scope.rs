//! A local glob never covers a peer-qualified path (issue #502,
//! TASK-c666eb306102).
//!
//! `bb:src/x.rs` names a file under the root of the peer `bb`, and §7 makes the
//! `<peer>:<glob>` form the only thing that crosses. The perimeter used to be
//! matched as one string against every glob, so a bare `**` bound every peer,
//! declared or not, and declaring a peer silently widened every entity scoped
//! `**` in the corpus.
//!
//! **Through the binary, over two corpora**: `a` declares `bb` as `../b`, and
//! carries one entity per side of the boundary. `b` exists so the declaration
//! resolves to a readable corpus, which is the shape the issue was reported in.

mod scratch;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-scope";

/// Scoped `**`: every path here, and nothing in a peer.
const STAR: &str = "ADR-aaaaaaaaaaaa";
/// Scoped `src/**`, the local glob that happened not to match `bb:src/x.rs`
/// only because of the prefix.
const SRC: &str = "ADR-bbbbbbbbbbbb";
/// Scoped `bb:**`: every path in the peer `bb`, and nothing here.
const PEER: &str = "ADR-cccccccccccc";

fn ank(dir: &Path, args: &[&str]) -> Output {
    let root = scratch::root();
    Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("ANK_AGENT", AGENT)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .env("NO_COLOR", "1")
        .output()
        .expect("the binary runs")
}

fn ok(dir: &Path, args: &[&str]) -> String {
    let out = ank(dir, args);
    assert!(
        out.status.success(),
        "ank {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn git_init(dir: &Path) {
    let out = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn corpus(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    git_init(dir);
    ok(dir, &["init"]);
}

/// An accepted ADR, written by hand: `accept` signs and commits, which is a
/// different verb's test. What matters here is a binding rule and its scope.
fn adr(repo: &Path, id: &str, glob: &str) {
    fs::write(
        repo.join(format!(".ank/entities/{id}.md")),
        format!(
            "---\nid: {id}\ntype: adr\nslug: s-{id}\ntitle: Scoped {glob}\n\
             created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - '{glob}'\n\
             constraint: |\n  A rule.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
        ),
    )
    .unwrap();
}

#[test]
fn a_local_glob_never_covers_a_peer_path() {
    let base = scratch::dir("peer-scope");
    let a = base.join("a");
    let b = base.join("b");
    corpus(&b);
    corpus(&a);
    ok(&a, &["config", "peers.bb", "../b"]);
    adr(&a, STAR, "**");
    adr(&a, SRC, "src/**");
    adr(&a, PEER, "bb:**");

    let listed = |args: &[&str]| -> Vec<&'static str> {
        let text = ok(&a, args);
        [STAR, SRC, PEER]
            .into_iter()
            .filter(|id| text.contains(&id[..8]))
            .collect()
    };

    // A declared peer, on a path that exists nowhere: only the entry that
    // spells the peer.
    assert_eq!(listed(&["scope", "bb:does/not/exist.txt"]), [PEER]);
    assert_eq!(listed(&["scope", "bb:src/x.rs"]), [PEER]);
    assert_eq!(listed(&["context", "bb:src/x.rs"]), [PEER]);
    // A name no declaration spells: nothing at all, `**` included.
    assert_eq!(listed(&["scope", "zz:whatever"]), Vec::<&str>::new());
    assert_eq!(listed(&["context", "zz:whatever"]), Vec::<&str>::new());
    // The local side is unchanged: `**` and `src/**` bind here, `bb:**` does not.
    assert_eq!(listed(&["scope", "src/x.rs"]), [STAR, SRC]);
    assert_eq!(listed(&["context", "src/x.rs"]), [STAR, SRC]);

    let _ = fs::remove_dir_all(&base);
}
