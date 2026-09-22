//! The exit-code page is what the command prints, byte for byte
//! (ADR-2b62b9a1fe67, TASK-0fad53f56d70).
//!
//! The page is generated, never typed: `docs/exit-codes.md` is the output of
//! the `exit-codes` binary this crate builds, which renders the table in
//! `src/exit.rs`. This test runs that binary, the command a contributor runs to
//! regenerate the page, and fails when the committed page differs from it --
//! so a code added to the enum and not to the page turns the suite red.

use std::path::Path;
use std::process::Command;

const REGENERATE: &str = "cargo run -q -p ank-contract --bin exit-codes > docs/exit-codes.md";

fn page_path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/exit-codes.md")
}

fn command_output() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_exit-codes"))
        .output()
        .expect("the exit-codes binary runs");
    assert!(out.status.success(), "exit-codes exited {:?}", out.status);
    assert!(out.stderr.is_empty(), "exit-codes wrote to stderr");
    String::from_utf8(out.stdout).expect("exit-codes prints UTF-8")
}

#[test]
fn the_exit_code_page_is_what_the_command_prints() {
    let page = std::fs::read_to_string(page_path()).unwrap_or_else(|e| {
        panic!("docs/exit-codes.md unreadable ({e}); regenerate: {REGENERATE}")
    });
    // Git on Windows may check the page out with CRLF; the bytes that matter
    // are the lines.
    let page = page.replace("\r\n", "\n");
    assert!(
        page == command_output(),
        "docs/exit-codes.md differs from what the command prints; regenerate: {REGENERATE}"
    );
}

/// The command renders every code of the table, each on a row of its own,
/// written out here rather than read back off the enum.
#[test]
fn the_command_prints_a_row_for_each_of_the_ten_codes() {
    let out = command_output();
    let rows: Vec<&str> = out
        .lines()
        .filter(|l| l.starts_with("| ") && !l.starts_with("| Code"))
        .collect();
    let codes: Vec<&str> = rows
        .iter()
        .map(|r| r.trim_start_matches("| ").split(' ').next().unwrap())
        .collect();
    assert_eq!(
        codes,
        ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
        "rows: {rows:#?}"
    );
}
