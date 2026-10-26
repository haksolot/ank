//! `ank init` bootstraps an agent: a teaching block at the end of `AGENTS.md`,
//! and a `CLAUDE.md` that imports it (ADR-5eb0f65c75da, TASK-917a8f9a1440).
//!
//! Measured on 2026-10-26 with agents that had no skill, no hook and no
//! memory: the one-line pointer `init` used to write kept 3 runs out of 8 inside
//! an accepted ADR, and every one that did had first read `.ank/` by hand. A
//! block naming the route, imported by `CLAUDE.md` so that Claude Code loads it,
//! kept 6 out of 6 across three models. What is asserted here is what the
//! decision names the block for, not its wording, which may change.
//!
//! **Through the binary.** The criterion is about what a directory holds after
//! `ank init` exits, so it is read back from the disk after a child process.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

fn spawn(program: &str, dir: &Path) -> Command {
    let mut c = Command::new(program);
    c.current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("ANK_AGENT", "bootstrap@fixture");
    c
}

fn repository(what: &str) -> PathBuf {
    let repo = scratch::dir(what);
    let out = spawn("git", &repo)
        .args(["init", "-q", "."])
        .output()
        .unwrap();
    assert!(out.status.success(), "git init: {out:?}");
    repo
}

fn init(dir: &Path, json: bool) -> Output {
    let mut c = spawn(ANK, dir);
    c.arg("init");
    if json {
        c.arg("--json");
    }
    let out = c.output().unwrap();
    assert!(out.status.success(), "ank init: {out:?}");
    out
}

fn read(path: &Path) -> String {
    String::from_utf8(fs::read(path).unwrap()).expect("written as UTF-8")
}

/// The five things ADR-5eb0f65c75da names, each checked in the text `init`
/// appended and nowhere else.
fn assert_teaches(block: &str) {
    assert!(
        block.contains("Ank is a CLI"),
        "Ank named as a CLI:\n{block}"
    );
    assert!(
        block.contains("ank context <path>") && block.contains("before editing"),
        "ank context <path> before editing:\n{block}"
    );
    assert!(
        block.contains("never read") && block.contains("`.ank/`"),
        "no reading .ank/ by hand:\n{block}"
    );
    for verb in ["ank claim", "ank done", "ank help"] {
        assert!(block.contains(verb), "{verb} named:\n{block}");
    }
    assert!(
        block.contains("https://haksolot.github.io/ank/install.html"),
        "the install page linked:\n{block}"
    );
}

#[test]
fn a_fresh_init_writes_the_block_and_a_claude_md_importing_it() {
    let repo = repository("bootstrap-fresh");
    let out = init(&repo, true);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(
        said.contains(r#""added":["AGENTS.md","CLAUDE.md"]"#),
        "--json lists both under added: {said}"
    );

    let agents = read(&repo.join("AGENTS.md"));
    assert_teaches(&agents);
    assert!(
        agents.ends_with(".html\n"),
        "the block ends the file:\n{agents}"
    );
    assert_eq!(read(&repo.join("CLAUDE.md")), "@AGENTS.md\n");
}

#[test]
fn the_block_is_appended_after_what_agents_md_already_held() {
    let repo = repository("bootstrap-agents-held");
    let before = "# Conventions\n\nTabs, not spaces.";
    fs::write(repo.join("AGENTS.md"), before).unwrap();
    init(&repo, false);
    let agents = read(&repo.join("AGENTS.md"));
    let appended = agents
        .strip_prefix(before)
        .unwrap_or_else(|| panic!("every existing byte kept:\n{agents}"));
    assert_teaches(appended);
    assert!(
        agents.ends_with(".html\n"),
        "the block ends the file:\n{agents}"
    );
}

#[test]
fn a_claude_md_that_already_imports_agents_md_is_left_byte_identical() {
    for (what, held) in [
        ("bare", "# Notes\n@AGENTS.md\nMore notes."),
        ("dotted", "See @./AGENTS.md for the conventions.\n"),
    ] {
        let repo = repository(&format!("bootstrap-imports-{what}"));
        fs::write(repo.join("CLAUDE.md"), held).unwrap();
        let out = init(&repo, true);
        assert_eq!(read(&repo.join("CLAUDE.md")), held, "{what}");
        let said = String::from_utf8_lossy(&out.stdout);
        assert!(
            said.contains(r#""added":["AGENTS.md"]"#),
            "{what}: CLAUDE.md is not reported: {said}"
        );
    }
}

#[test]
fn a_claude_md_without_the_import_gets_it_appended_and_keeps_every_byte() {
    for (what, held) in [
        ("terminated", "# Project\n\nRun the tests.\n"),
        (
            "unterminated",
            "# Project\nMentions AGENTS.md without importing it",
        ),
    ] {
        let repo = repository(&format!("bootstrap-lacks-{what}"));
        fs::write(repo.join("CLAUDE.md"), held).unwrap();
        init(&repo, false);
        let claude = read(&repo.join("CLAUDE.md"));
        let appended = claude
            .strip_prefix(held)
            .unwrap_or_else(|| panic!("{what}: every existing byte kept:\n{claude}"));
        assert!(
            appended.ends_with("\n@AGENTS.md\n"),
            "{what}: @AGENTS.md appended on a line of its own:\n{claude}"
        );
    }
}

#[test]
fn a_second_init_leaves_both_files_byte_identical() {
    let repo = repository("bootstrap-again");
    init(&repo, false);
    let (agents, claude) = (read(&repo.join("AGENTS.md")), read(&repo.join("CLAUDE.md")));
    let out = init(&repo, true);
    assert_eq!(read(&repo.join("AGENTS.md")), agents);
    assert_eq!(read(&repo.join("CLAUDE.md")), claude);
    let said = String::from_utf8_lossy(&out.stdout);
    assert!(said.contains(r#""added":[]"#), "{said}");
}
