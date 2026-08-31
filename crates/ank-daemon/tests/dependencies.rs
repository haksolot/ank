//! **The watcher is not a second implementation of anything**, made mechanical
//! (ADR-24e21cb83793, TASK-9dd22f2b0430).
//!
//! The file folded into `ank`; the read did not. That is the clause the decision
//! rests its first paragraph on, and its reason is not symmetry: a process that
//! has to run the CLI to learn anything has no dispatch of its own to be a
//! second one, where a process that linked `ank-core` would carry a second
//! parser, a second set of hashes and a second answer to what a corpus holds --
//! and it would drift silently, because a stale cache answers rather than fails.
//! Now that this crate is a library of the very binary it spawns, the link it
//! must not take is one `cargo add` away and would compile.
//!
//! A rule that lives only in prose is a rule the second contributor breaks for a
//! good reason, on a Tuesday, in a commit whose message explains why it is fine.
//! So the rule is read back out of the build, the way
//! `crates/ank-mcp/tests/dependencies.rs` and `crates/ank-tui/tests/dependencies.rs`
//! read theirs: `cargo tree` answers what this crate links, and the crate's own
//! sources answer what it reaches for.
//!
//! **What this crate may do that those two may not is spawn `git`**, and the
//! assertions below say so rather than copying a prohibition that does not
//! belong here. Mirroring `refs/ank/*` into a tracking namespace is the one
//! thing ADR-24e21cb83793 has this process write into somebody else's
//! repository, and it is a fetch and nothing else -- which is a claim about
//! *which* git commands are reachable, so that is what is asserted.

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
            "ank-daemon",
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

/// The read stays on the other side of a process boundary.
///
/// `ank-core` is what reads a corpus and `ank-cli` is what dispatches into it;
/// either one in this graph is a second road to the index, and the cache this
/// process warms would then be written by code the CLI never ran.
#[test]
fn the_graph_carries_neither_the_dispatch_nor_the_core() {
    let tree = tree();
    let names = crates_of(&tree);
    assert!(
        names.iter().any(|n| n == "ank-daemon"),
        "the graph is not this crate's:\n{tree}"
    );
    for forbidden in ["ank-cli", "ank-core"] {
        assert!(
            !names.iter().any(|n| n == forbidden),
            "ank-daemon links {forbidden}, and ADR-24e21cb83793 forbids it: what \
             folded into the one binary is the file and never the read, so the \
             watcher warms the index by running `ank` or it is a second \
             implementation of one.\n{tree}"
        );
    }
    // The refs are git's and git is asked for by name, on the plumbing rule §8
    // already holds the CLI to (ADR-9307e5d214a7). A library that spoke the
    // object format would be a second implementation of the same kind as the
    // two above. Not a list of the git libraries there are, which would go
    // stale the day a new one is published: anything whose name carries `git`
    // is refused, and a legitimate dependency that happened to be named that
    // way would be worth stopping to argue about.
    for name in &names {
        assert!(
            !name.to_ascii_lowercase().contains("git"),
            "{name} is in ank-daemon's graph, and this process reaches git by \
             running it.\n{tree}"
        );
    }
}

/// Nothing else arrives either, and that is the half `cargo tree` alone would
/// let through: a crate can be added without being `ank-cli`, `ank-core` or a
/// git library. A file-watching crate is the one this would be spent on, and
/// §13 spends a dependency on necessity -- what a subscription buys over a stat
/// of a directory holding a few hundred files is latency on a process whose
/// entire purpose is latency nobody is required to care about.
///
/// The list is the manifest's argument written as an assertion.
/// `ank-contract` carries the exit codes and the event stream both ends share
/// (ADR-6fd69efb629c); `serde` and `serde_yaml` read the declaration, and both
/// are already in this workspace's lockfile through `ank-core`.
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
        ["ank-contract", "serde", "serde_yaml"],
        "a dependency arrived. It may well be the right call -- and the \
         argument for it belongs in the manifest beside the three that are \
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
        "the crate declares no library target, and the verb `ank watch` links \
         one (ADR-24e21cb83793, ADR-1ea31c2f3c5a)"
    );
    assert!(
        !declarations.contains(&"[[bin]]"),
        "the crate declares an executable again: ank ships one, and a surface \
         that is not a verb has to be distributed, documented and discovered as \
         a third thing (ADR-8bd76e8d7c4e, ADR-1ea31c2f3c5a)"
    );
    assert!(
        !manifest().join("src/main.rs").exists(),
        "src/main.rs is back, and cargo would build it into a second executable \
         whatever the manifest says"
    );
}

/// The other half of the constraint: what the sources reach for.
///
/// A graph clean of `ank-core` says nothing about a crate that opens
/// `.ank/entities/` with `std::fs` and parses what it finds. That is the second
/// implementation the decision refuses, and it is plain text in the sources.
///
/// **The assertion is per module, because "opens a file" is not the property.**
/// This process reads exactly one file itself and it is the reader's own
/// declaration, outside every repository -- so a blanket ban on reading would
/// ban the thing the watcher is told what to watch by. What may not happen is a
/// read *inside somebody else's checkout*, and there are exactly two modules
/// that reach into one: `warm` walks the corpus directory and `fetch` mirrors
/// its refs. Neither may open what it finds there. `warm::fingerprint` is a stat
/// walk on purpose -- lengths and mtimes, never contents -- because the question
/// it answers is "is it worth spawning a read", and the CLI hashes the files
/// itself on the next one.
#[test]
fn nothing_that_reaches_into_a_watched_checkout_opens_what_it_finds() {
    let reaching = ["warm.rs", "fetch.rs"];
    let mut seen = Vec::new();
    for (file, source) in sources() {
        if !reaching.contains(&file.as_str()) {
            continue;
        }
        seen.push(file.clone());
        let text = code_of(&source);
        for forbidden in ["read_to_string", "File::open", "fs::read(", "fs::write"] {
            assert!(
                !text.contains(forbidden),
                "{file} calls {forbidden} inside a watched checkout: the \
                 watcher learns what a corpus holds by running the binary, and \
                 a second reader of these files drifts silently because a stale \
                 cache answers rather than fails (ADR-24e21cb83793)"
            );
        }
    }
    seen.sort();
    let mut expected = reaching.to_vec();
    expected.sort();
    assert_eq!(
        seen, expected,
        "a module that reaches into a watched checkout was renamed away from \
         this list, and the check went quiet with it"
    );
}

/// And the parser stays where the one file this process reads is.
///
/// `serde_yaml` in a second module is a second thing that reads a document, and
/// the only document here is the declaration. An entity parser arriving in this
/// crate would arrive exactly this way.
#[test]
fn the_only_document_this_crate_parses_is_the_declaration() {
    for (file, source) in sources() {
        if file == "declare.rs" {
            continue;
        }
        let text = code_of(&source);
        assert!(
            !text.contains("serde_yaml") && !text.contains("Deserialize"),
            "{file} parses a document, and the one document this process reads \
             is the reader's own declaration in declare.rs: what a corpus holds \
             is what the binary answers (ADR-24e21cb83793)"
        );
    }
}

/// One process is spawned that is not `git`, and it is the binary the dispatch
/// named.
///
/// **`current_exe` is not among the things this crate may call**, and that is
/// the shape the fold left behind. While the watcher was a sibling it had to
/// find the `ank` it was released with; the verb hands it
/// [`ank_daemon::Address::exe`] instead, so a search reappearing here would be
/// this crate deciding which build to warm an index with, which is exactly the
/// question that had a wrong answer before.
///
/// The git commands are enumerated rather than counted, because "we only fetch
/// `refs/ank/*`" is a sentence that stays true right up until a second command
/// is added beside it. A subcommand not on this list is a write into somebody's
/// repository that ADR-24e21cb83793 did not authorise -- and the ones that are
/// here are read-only but for the fetch, whose refspec
/// `a_watching_cycle_moves_the_tracking_namespace_and_nothing_else` pins.
#[test]
fn the_sources_spawn_git_and_the_binary_the_dispatch_named() {
    let allowed_git = ["config", "fetch", "for-each-ref", "rev-list"];
    for (file, source) in sources() {
        let text = code_of(&source);
        assert!(
            !text.contains("current_exe"),
            "{file} looks for a binary: the verb hands this crate the one to \
             run, and a search here is a second answer to which build warms an \
             index (ADR-1ea31c2f3c5a)"
        );
        let spawns = text.matches("Command::new").count();
        let git = text.matches("Command::new(\"git\")").count();
        let ank = text.matches("Command::new(ank_bin)").count();
        assert_eq!(
            spawns,
            git + ank,
            "{file} spawns something that is neither git nor the binary the \
             dispatch named: {spawns} call(s), {git} git and {ank} ank"
        );
        for (at, _) in text.match_indices("        .args([") {
            let block = &text[at..];
            let Some(end) = block.find("])") else {
                continue;
            };
            let subcommand = block[..end]
                .split('"')
                .nth(1)
                .unwrap_or_default()
                .to_string();
            if subcommand.is_empty() || !text.contains("Command::new(\"git\")") {
                continue;
            }
            assert!(
                allowed_git.contains(&subcommand.as_str()),
                "{file} runs `git {subcommand}`, which is not one of \
                 {allowed_git:?}: the one thing this process writes into a \
                 repository is a fetch into the tracking namespace \
                 (ADR-24e21cb83793)"
            );
        }
    }
}

/// A source with its comments removed.
///
/// The prose is allowed to name what the code may not -- it has to, since what
/// it is explaining is that the crate does not do those things -- and the code
/// is not. A line whose first non-space characters are a slash pair is prose
/// whole; a trailing comment is cut only where no string opened before it, so a
/// literal carrying two slashes is never mistaken for one.
fn code_of(source: &str) -> String {
    without_tests(source)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| match line.find("//") {
            Some(at) if !line[..at].contains('"') => &line[..at],
            _ => line,
        })
        .collect::<Vec<&str>>()
        .join("\n")
}

/// The crate's code, with its own unit tests cut off.
///
/// A test module may read a file it wrote, spawn what it likes and name what the
/// crate may not -- `stream.rs` reads back the line it just emitted, which is
/// the only way to assert that it emitted one. What is being constrained here is
/// the code that runs in production.
///
/// **Cut at `#[cfg(test)]` and to the end of the file, which is checked rather
/// than assumed**: every source in this crate puts its test module last, and a
/// file that stopped doing so would have real code silently exempted by this
/// helper. So a second `#[cfg(test)]` fails instead.
fn without_tests(source: &str) -> &str {
    assert!(
        source.matches("#[cfg(test)]").count() <= 1,
        "a source carries more than one test module, and this helper cuts at \
         the first: move them together at the end, or this exempts code nobody \
         meant to exempt"
    );
    match source.find("#[cfg(test)]") {
        Some(at) => &source[..at],
        None => source,
    }
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
