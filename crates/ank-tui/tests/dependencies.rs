//! The constraint made mechanical (ADR-8bd76e8d7c4e, ADR-0b55983421dd).
//!
//! ADR-8bd76e8d7c4e forbids the terminal reader from linking `ank-core`, from
//! reading `.ank/` and from touching `refs/ank/*`, so that the refusals it shows
//! are the refusals the CLI gave and there is no second dispatch path to keep in
//! step. ADR-0b55983421dd adds one of the same kind on the other side: the
//! reader is drawn with ratatui over crossterm, and **no FFI enters this tree
//! for any of it, on any platform**. A rule that lives only in prose is a rule
//! the second contributor breaks for a good reason, on a Tuesday, in a commit
//! whose message explains why it is fine.
//!
//! So the rule is read back out of the build. `cargo tree` answers what this
//! crate links, and the crate's own sources answer what it reaches for -- and
//! either one going wrong fails here rather than surviving review.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The dependency graph of this crate, as cargo resolves it.
///
/// `--target all` on purpose: a dependency that only appears on one platform is
/// still a dependency, and a graph read for the host alone would have called
/// this crate clean while a `git2` sat behind a `cfg(windows)`.
///
/// `--offline` because a test must not reach the network. The lockfile is
/// complete by the time anything is compiled, so there is nothing to fetch.
fn tree() -> String {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let out = Command::new(cargo)
        .args([
            "tree",
            "-p",
            "ank-tui",
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

/// The crate names the graph carries, one per line, without their versions.
fn crates_of(tree: &str) -> Vec<String> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| !name.is_empty())
        .map(|name| name.to_string())
        .collect()
}

#[test]
fn the_graph_carries_neither_ank_core_nor_git() {
    let tree = tree();
    let names = crates_of(&tree);
    assert!(
        names.iter().any(|n| n == "ank-tui"),
        "the graph is not this crate's:\n{tree}"
    );
    assert!(
        !names.iter().any(|n| n == "ank-core"),
        "ank-tui links ank-core, and ADR-8bd76e8d7c4e forbids it: the reader \
         reaches the corpus by running the CLI, or there are two dispatch \
         paths.\n{tree}"
    );
    // Not a list of the git libraries there are, which would go stale the day a
    // new one is published: anything whose name carries `git` is refused, and a
    // legitimate dependency that happened to be named that way would be worth
    // stopping to argue about.
    for name in &names {
        assert!(
            !name.to_ascii_lowercase().contains("git"),
            "{name} is in ank-tui's graph, and the reader touches no git: \
             claims are refs, and refs are the CLI's (ADR-4e7c25b1f639).\n{tree}"
        );
    }
}

/// Nothing else arrives either, and that is the half `cargo tree` alone would
/// let through: a crate can be added without being `ank-core` or a git library.
///
/// The list is the manifest's argument written as an assertion. `ank-contract`
/// is the machine contract every surface consumes (ADR-6fd69efb629c);
/// `serde_yaml` is already in the tree four times over and is what reads the
/// CLI's `--json`; `ratatui` and `crossterm` are what ADR-0b55983421dd spends
/// and the whole of what it spends; the rest is what those four bring with them
/// and nothing this crate chose.
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
        ["ank-contract", "crossterm", "ratatui", "serde_yaml"],
        "a dependency arrived. It may well be the right call -- and the \
         argument for it belongs in the manifest beside the two that are \
         there, and in this list.\n{tree}"
    );
}

/// No `extern` block, on any platform (ADR-0b55983421dd).
///
/// **This is the assertion the whole decision rests on.** What sent the reader
/// to a line discipline in the first place was that raw mode is `tcsetattr` on
/// Unix and `SetConsoleMode` on Windows, each behind an `extern` this workspace
/// does not otherwise have, and that only one of the two could be run from
/// where the code was written. Taking crossterm is worth what it costs
/// precisely because it answers that on all three platforms without this crate
/// declaring a single foreign symbol -- so if one ever arrives, the trade this
/// decision made has quietly stopped being the trade that was made.
///
/// Read off the sources with their prose removed, like the check below it: this
/// file and the module headers have to be able to say the word.
#[test]
fn no_foreign_symbol_is_declared_in_this_tree() {
    for (file, source) in sources() {
        let text = code_of(&source);
        for forbidden in ["extern \"C\"", "extern \"system\"", "#[link"] {
            assert!(
                !text.contains(forbidden),
                "{file} declares {forbidden}: the reader reaches raw mode, the \
                 window and a keystroke through crossterm, which is what \
                 ADR-0b55983421dd bought and the only reason it was worth \
                 buying"
            );
        }
        // `unsafe` is the wider net, and it catches a foreign call reached
        // through a dependency's own binding as well as one declared here.
        assert!(
            !text.contains("unsafe "),
            "{file} is unsafe: nothing this reader does needs to be"
        );
    }
}

/// The other half of the constraint: what the sources reach for.
///
/// A graph clean of `ank-core` says nothing about a crate that opens
/// `.ank/entities/` with `std::fs`, or that spawns `git` by name. Both would be
/// a second road to the corpus, and both are plain text in the sources.
#[test]
fn the_sources_reach_for_nothing_but_the_binary() {
    for (file, source) in sources() {
        let text = code_of(&source);
        for forbidden in [".ank/", "refs/ank", "index.db"] {
            assert!(
                !text.contains(forbidden),
                "{file} names {forbidden}: the reader reads the corpus through \
                 the CLI and never through the filesystem (ADR-8bd76e8d7c4e)"
            );
        }
        // One process is spawned by this crate and it is the CLI, addressed by
        // the path the dispatch resolved. A second `Command::new` is a second
        // road out.
        let spawns: Vec<&str> = text.match_indices("Command::new").map(|(_, m)| m).collect();
        let allowed = text.matches("Command::new(&self.address.exe)").count();
        assert!(
            spawns.len() == allowed,
            "{file} spawns something other than the CLI: {} call(s), {allowed} \
             of them the binary",
            spawns.len()
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
