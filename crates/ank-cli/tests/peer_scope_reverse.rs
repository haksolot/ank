//! `scope` inside a peer names the constraints bound to it from elsewhere
//! (issue #500, TASK-2f5d6af5de36).
//!
//! A binding across corpora is resolved at both ends: `b` reads `a` through its
//! own declaration `peers.aa`, and `a`'s entry `bb:**` is resolved through
//! `a`'s declaration `peers.bb` back to `b`. `context` inside `b` already
//! served such a rule as `<id>@aa`; `scope` listed nothing from `a`, so two
//! verbs answered differently about what binds one path.
//!
//! **Through the binary, over three corpora**: `a` declares `bb` and `cc`, `b`
//! declares `a` as `aa`, and `c` exists so the declaration `cc` resolves to a
//! readable corpus that is not `b`.

mod scratch;

use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const AGENT: &str = "claude-code@peer-scope-reverse";

/// In `a`, scoped `bb:**`: binds every path of `b`.
const BOUND: &str = "ADR-dddddddddddd";
/// In `a`, scoped `cc:**`: binds `c`, and nothing in `b`.
const ELSEWHERE: &str = "ADR-eeeeeeeeeeee";

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

fn corpus(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
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
fn scope_inside_the_governed_corpus_lists_what_context_serves() {
    let base = scratch::dir("peer-scope-reverse");
    let a = base.join("a");
    let b = base.join("b");
    let c = base.join("c");
    corpus(&a);
    corpus(&b);
    corpus(&c);
    ok(&a, &["config", "peers.bb", "../b"]);
    ok(&a, &["config", "peers.cc", "../c"]);
    ok(&b, &["config", "peers.aa", "../a"]);
    adr(&a, BOUND, "bb:**");
    adr(&a, ELSEWHERE, "cc:**");

    let bound = format!("{BOUND}@aa");
    let elsewhere = format!("{ELSEWHERE}@aa");

    // The reference: what `context` already serves, measured rather than assumed.
    let context = ok(&b, &["context", "README.md"]);
    assert!(
        context.contains(&bound),
        "context serves {bound}:\n{context}"
    );
    assert!(!context.contains(ELSEWHERE), "context:\n{context}");

    // `scope` answers the same, in the same form.
    let scope = ok(&b, &["scope", "README.md"]);
    assert!(scope.contains(&bound), "scope lists {bound}:\n{scope}");
    assert!(!scope.contains(ELSEWHERE), "scope:\n{scope}");
    assert!(
        !scope.contains("nothing covers this path"),
        "scope:\n{scope}"
    );

    // And `--json` names the same entity by the same form.
    let json = ok(&b, &["scope", "README.md", "--json"]);
    assert!(json.contains(&format!("\"id\":\"{bound}\"")), "{json}");
    assert!(!json.contains(&elsewhere), "{json}");

    let _ = fs::remove_dir_all(&base);
}

#[test]
fn help_config_says_a_binding_across_corpora_needs_both_declarations() {
    let base = scratch::dir("peer-scope-reverse-help");
    corpus(&base);
    let config = ok(&base, &["help", "config"]);
    assert!(
        config.contains(
            "a binding across corpora needs both declarations: the governing corpus names the \
             governed one in its scope, <peer>:<glob> through its own peers.<peer>, and the \
             governed corpus declares the governing one in peers.<name> to read it"
        ),
        "{config}"
    );
    let scope = ok(&base, &["help", "scope"]);
    assert!(
        scope.contains("reads the declared peers as context does"),
        "{scope}"
    );
    let _ = fs::remove_dir_all(&base);
}
