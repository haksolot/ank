//! **One executable is not one process**, made mechanical (ADR-fd98f4bc6dea,
//! TASK-e655d28c83cb).
//!
//! The file folded into `ank`; the dispatch did not. That is the clause the
//! decision spends a whole section on, and its reason is not symmetry: linking
//! `ank-cli` into this surface would re-derive every refusal, and anything
//! re-derived can differ, where spawning inherits them by construction. Now that
//! this crate is a library of the very binary it spawns, the link it must not
//! take is one `cargo add` away and would compile.
//!
//! A rule that lives only in prose is a rule the second contributor breaks for a
//! good reason, on a Tuesday, in a commit whose message explains why it is fine.
//! So the rule is read back out of the build, the way
//! `crates/ank-tui/tests/dependencies.rs` reads ADR-8bd76e8d7c4e's: `cargo tree`
//! answers what this crate links, and the crate's own sources answer what it
//! reaches for.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The dependency graph of this crate, as cargo resolves it.
///
/// `--target all` on purpose: a dependency that only appears on one platform is
/// still a dependency. `--offline` because a test must not reach the network —
/// the lockfile is complete by the time anything is compiled.
fn tree() -> String {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(cargo)
        .args([
            "tree",
            "-p",
            "ank-mcp",
            "--edges",
            "normal",
            "--target",
            "all",
            "--offline",
            "--prefix",
            "none",
        ])
        .current_dir(manifest())
        .output()
        .expect("cargo must be runnable: it is what built this test");
    assert!(
        out.status.success(),
        "cargo tree failed, and a graph that cannot be read is not a graph that \
         is clean: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn crates_of(tree: &str) -> Vec<String> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .collect()
}

/// The dispatch stays on the other side of a process boundary.
///
/// `ank-cli` is the dispatch itself and `ank-core` is what it dispatches into;
/// either one in this graph is a second road to the corpus, and the refusals a
/// client sees would then be this crate's rather than the binary's.
#[test]
fn the_graph_carries_neither_the_dispatch_nor_the_core() {
    let tree = tree();
    let names = crates_of(&tree);
    assert!(
        names.iter().any(|n| n == "ank-mcp"),
        "the graph is not this crate's:\n{tree}"
    );
    for forbidden in ["ank-cli", "ank-core"] {
        assert!(
            !names.iter().any(|n| n == forbidden),
            "ank-mcp links {forbidden}, and ADR-fd98f4bc6dea forbids it: what \
             folded into the one binary is the file and never the dispatch, so \
             the surface reaches the corpus by running `ank` or there are two \
             dispatch paths.\n{tree}"
        );
    }
    // Claims are refs and refs are the CLI's (ADR-4e7c25b1f639). Not a list of
    // the git libraries there are, which would go stale the day a new one is
    // published: anything whose name carries `git` is refused, and a legitimate
    // dependency that happened to be named that way would be worth stopping to
    // argue about.
    for name in &names {
        assert!(
            !name.to_ascii_lowercase().contains("git"),
            "{name} is in ank-mcp's graph, and this surface touches no git.\n{tree}"
        );
    }
}

/// Nothing else arrives either, and that is the half `cargo tree` alone would
/// let through: a crate can be added without being `ank-cli`, `ank-core` or a
/// git library.
///
/// The list is the manifest's argument written as an assertion. `ank-contract`
/// is the table this surface is generated from (ADR-6fd69efb629c); `serde_yaml`
/// is already in the tree four times over and is what reads JSON-RPC, since
/// `ank-contract::json` writes and does not read.
#[test]
fn the_crate_takes_nothing_the_tree_did_not_already_carry() {
    let tree = tree();
    let mut direct: Vec<String> = std::fs::read_to_string(manifest().join("Cargo.toml"))
        .expect("this crate has a manifest")
        .lines()
        .skip_while(|l| l.trim() != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.trim_start().starts_with('['))
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once('=').map(|(name, _)| name.trim().to_string()))
        .collect();
    direct.sort();
    assert_eq!(
        direct,
        ["ank-contract", "serde_yaml"],
        "a dependency arrived. It may well be the right call -- and the \
         argument for it belongs in the manifest beside the two that are \
         there, and in this list.\n{tree}"
    );
}

/// The manifest declares a library and no executable.
///
/// This is the half of the criterion `cargo build --workspace` shows and a suite
/// cannot: a `[[bin]]` returning would put a second file back beside `ank`, and
/// it would build green. The manifest is where that is decided, so the manifest
/// is what is read.
#[test]
fn the_crate_declares_a_library_and_no_binary() {
    let text =
        std::fs::read_to_string(manifest().join("Cargo.toml")).expect("this crate has a manifest");
    let declarations: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('['))
        .collect();
    assert!(
        declarations.contains(&"[lib]"),
        "the crate declares no library target, and the verb `ank mcp` links one \
         (ADR-fd98f4bc6dea)"
    );
    assert!(
        !declarations.contains(&"[[bin]]"),
        "the crate declares an executable again: ank ships one, and a surface \
         that is not a verb has to be distributed, documented and discovered as \
         a third thing (ADR-fd98f4bc6dea, ADR-1ea31c2f3c5a)"
    );
    assert!(
        !manifest().join("src/main.rs").exists(),
        "src/main.rs is back, and cargo would build it into a second executable \
         whatever the manifest says"
    );
}

/// The other half of the constraint: what the sources reach for.
///
/// A graph clean of `ank-cli` says nothing about a crate that opens
/// `.ank/entities/` with `std::fs`, or that spawns `git` by name. Both would be
/// a second road to the corpus, and both are plain text in the sources.
#[test]
fn the_sources_reach_for_nothing_but_the_binary() {
    for (file, source) in sources() {
        let text = code_of(&source);
        for forbidden in [".ank/", "refs/ank", "index.db"] {
            assert!(
                !text.contains(forbidden),
                "{file} names {forbidden}: the surface reaches the corpus by \
                 running the binary and never through the filesystem \
                 (ADR-fd98f4bc6dea)"
            );
        }
        // One process is spawned by this crate and it is the CLI, addressed by
        // the path the dispatch resolved. A second `Command::new` is a second
        // road out.
        let spawns = text.matches("Command::new").count();
        let allowed = text.matches("Command::new(&address.exe)").count();
        assert_eq!(
            spawns, allowed,
            "{file} spawns something other than the binary: {spawns} call(s), \
             {allowed} of them the binary"
        );
    }
}

/// A source with its comments removed.
///
/// The prose is allowed to name `.ank/` and `refs/ank/*` -- it has to, since
/// what it is explaining is that the crate does not touch them -- and the code
/// is not. A line whose first non-space characters are a slash pair is prose
/// whole; a trailing comment is cut only where no string opened before it, so a
/// literal carrying two slashes is never mistaken for one.
fn code_of(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| match line.find("//") {
            Some(at) if !line[..at].contains('"') => &line[..at],
            _ => line,
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The crate's own sources, tests excluded: a test may name what the crate may
/// not, and this file is the proof of that.
fn sources() -> Vec<(String, String)> {
    let src = manifest().join("src");
    let mut out = Vec::new();
    walk(&src, &mut out);
    assert!(!out.is_empty(), "the crate has sources to read");
    out
}

fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
    for entry in std::fs::read_dir(dir).expect("the source directory must be readable") {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            out.push((name, std::fs::read_to_string(&path).unwrap()));
        }
    }
}
