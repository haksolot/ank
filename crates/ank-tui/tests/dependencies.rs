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

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The dependency graph of this crate, as the lockfile records it: every
/// package reachable from `ank-tui`, and the lockfile itself for the message.
///
/// **Read out of `Cargo.lock` and not out of `cargo tree`, and the reason is
/// worth stating.** This used to run `cargo tree --target all --offline`, on
/// the argument that a dependency behind a `cfg(windows)` is still a
/// dependency and a host-only graph would have called this crate clean while a
/// `git2` sat behind one. That argument is right and this keeps it. What
/// stopped working is the instrument: `--target all` needs the manifest of
/// every package on every target, `--offline` forbids fetching one, and a host
/// build downloads only what it compiles. Before ratatui the two happened to
/// agree; ratatui's graph reaches `bumpalo` through a `wasm32`-only edge of
/// `time`, nothing on a CI runner ever downloads it, and the test went red
/// asking for the network it is forbidden to use.
///
/// The lockfile answers the same question without asking anything: it records
/// the resolution for every target and every optional dependency, checked in,
/// which is *broader* than `--target all` rather than narrower. For a rule that
/// says a name must not appear, broader is the safe direction -- a package that
/// is locked and never compiled still fails this, and the comment below says
/// why that is the outcome to want.
fn graph() -> (String, Vec<String>) {
    let text = std::fs::read_to_string(lockfile()).expect("the workspace has a lockfile");
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for block in text.split("[[package]]").skip(1) {
        let Some(name) = field(block, "name") else {
            continue;
        };
        edges.entry(name).or_default().extend(dependencies(block));
    }
    // Every package reachable from this crate, which is what "in ank-tui's
    // graph" means: a crate the workspace carries for `ank-cli` alone is not
    // this crate's business.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut walking = vec!["ank-tui".to_string()];
    while let Some(name) = walking.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if let Some(deps) = edges.get(&name) {
            walking.extend(deps.iter().cloned());
        }
    }
    (text, seen.into_iter().collect())
}

fn lockfile() -> PathBuf {
    manifest()
        .parent()
        .and_then(|crates| crates.parent())
        .expect("the crate sits two directories under the workspace root")
        .join("Cargo.lock")
}

/// One `key = "value"` of a lockfile block.
fn field(block: &str, key: &str) -> Option<String> {
    block
        .lines()
        .find_map(|line| line.strip_prefix(&format!("{key} = \"")))
        .and_then(|rest| rest.strip_suffix('"'))
        .map(|value| value.to_string())
}

/// The names in a block's `dependencies` list.
///
/// An entry is `"name"`, or `"name version"` where the lockfile carries two
/// versions of one package. The version is dropped: what every assertion here
/// asks about is which crates are in the graph, never which release of one.
fn dependencies(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in block.lines() {
        if line.starts_with("dependencies = [") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        if line.starts_with(']') {
            break;
        }
        if let Some(entry) = line.trim().trim_end_matches(',').strip_prefix('"') {
            if let Some(entry) = entry.strip_suffix('"') {
                out.push(
                    entry
                        .split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                );
            }
        }
    }
    out
}

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn the_graph_carries_neither_ank_core_nor_git() {
    let (tree, names) = graph();
    // The walk starts at this crate by name, so a rename that lost it would
    // leave an empty graph and every assertion below would pass on nothing.
    assert!(
        names.iter().any(|n| n == "ank-tui"),
        "the lockfile carries no ank-tui, so this graph is nobody's"
    );
    // And the walk reaches what the manifest declares. Without this the two
    // assertions below would pass on a graph that lost its edges: "ank-core is
    // not in it" is worth nothing when nothing is in it.
    for declared in ["ank-contract", "crossterm", "ratatui", "serde_yaml"] {
        assert!(
            names.iter().any(|n| n == declared),
            "the walk did not reach {declared}, which the manifest declares: \
             the lockfile was not read as a graph"
        );
    }
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
    let (tree, _) = graph();
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

/// One function in this crate spawns a verb that writes, and it is the one
/// behind the confirmation (TASK-d4a882345837).
///
/// The criterion says no verb that writes can be spawned without the exact
/// command line having been shown first. What makes that a property of the code
/// rather than a habit of whoever wrote it is that [`ank_tui::ank::Ank::act`]
/// has exactly one caller: `App::confirmed`, which runs only on a command
/// `App::propose` composed and the screen drew, and which takes it rather than
/// borrowing it so the same one cannot be answered twice.
///
/// A second call site would be a second road to a spawn, and it would be a road
/// with no confirmation on it. It fails here rather than in review, on the same
/// reasoning as everything else in this file: a rule that lives only in prose is
/// a rule the second contributor breaks for a good reason, on a Tuesday.
///
/// `ank.rs` is exempt and it is the one exemption: the gate is defined there and
/// its own suite drives it, so the calls in that file are the assertion that the
/// gate refuses what it should rather than a road around it.
#[test]
fn one_function_in_this_crate_spawns_a_verb_that_writes() {
    let calls: Vec<String> = sources()
        .into_iter()
        .filter(|(file, _)| file != "ank.rs")
        .flat_map(|(file, source)| {
            code_of(&source)
                .lines()
                .filter(|line| line.contains("ank.act("))
                .map(|line| format!("{file}  {}", line.trim()))
                .collect::<Vec<String>>()
        })
        .collect();
    assert_eq!(
        calls.len(),
        1,
        "a verb that writes is spawned from {} places, and the confirmation is \
         in front of one of them:\n{}",
        calls.len(),
        calls.join("\n")
    );
    assert!(
        calls[0].starts_with("view.rs"),
        "the spawn moved out of the view, where the confirmation is: {}",
        calls[0]
    );
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
