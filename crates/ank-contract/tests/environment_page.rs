//! The environment-variable reference is what the command prints, byte for
//! byte (ADR-2b62b9a1fe67, TASK-28290de150ac).
//!
//! `docs/environment.md` is prose and replayed examples, written by hand, with
//! one section that is not: the table of every variable the binary reads, which
//! is the output of the `environment` binary this crate builds, rendering the
//! table in `src/env.rs`. This test runs that binary, the command the page's
//! first lines name, and fails when the section between the page's markers
//! differs from it -- so a variable added to the table and not to the page
//! turns the suite red. `tests/exit_codes_page.rs` does the same for
//! `docs/exit-codes.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

const REGENERATE: &str = "cargo run -q -p ank-contract --bin environment -- docs/environment.md";

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn command_output() -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_environment"))
        .output()
        .expect("the environment binary runs");
    assert!(out.status.success(), "environment exited {:?}", out.status);
    assert!(out.stderr.is_empty(), "environment wrote to stderr");
    String::from_utf8(out.stdout).expect("environment prints UTF-8")
}

fn page() -> String {
    std::fs::read_to_string(workspace().join("docs/environment.md"))
        .unwrap_or_else(|e| {
            panic!("docs/environment.md unreadable ({e}); regenerate: {REGENERATE}")
        })
        // Git on Windows may check the page out with CRLF; the bytes that
        // matter are the lines.
        .replace("\r\n", "\n")
}

#[test]
fn the_first_lines_of_the_page_name_the_command() {
    let page = page();
    let head: String = page.lines().take(3).collect::<Vec<_>>().join("\n");
    assert!(
        head.contains(REGENERATE),
        "the first lines of docs/environment.md do not name `{REGENERATE}`:\n{head}"
    );
}

#[test]
fn the_generated_section_is_what_the_command_prints() {
    let page = page();
    let out = command_output();
    let begin = "<!-- BEGIN environment -->\n";
    let end = "<!-- END environment -->\n";
    let (Some(b), Some(e)) = (page.find(begin), page.find(end)) else {
        panic!(
            "docs/environment.md carries no `{begin}` .. `{end}` section; regenerate: {REGENERATE}"
        );
    };
    let section = &page[b + begin.len()..e];
    assert!(
        section == out,
        "the generated section of docs/environment.md differs from what the command prints; \
         regenerate: {REGENERATE}"
    );
}

/// Every name the binary reads at run time, written out here rather than read
/// back off the table: a test that derived its expectation from the table
/// would agree with a variable dropped from it. The list was measured by
/// reading every `std::env::var` and `var_os` under `crates/*/src`.
#[test]
fn the_command_prints_a_row_for_every_variable_the_binary_reads() {
    let out = command_output();
    let mut names: Vec<&str> = out
        .lines()
        .filter(|l| l.starts_with("| `"))
        .map(|l| l.trim_start_matches("| `").split('`').next().unwrap())
        .collect();
    names.sort_unstable();
    let mut expected = [
        "ANK_AGENT",
        "ANK_INDEX_BUSY_MS",
        "ANK_INDEX_REFRESHED",
        "ANK_INDEX_STEPS",
        "ANK_TRACE_READS",
        "ANK_UPDATE_REPOSITORY",
        "ANSICON",
        "APPDATA",
        "COMPUTERNAME",
        "ConEmuANSI",
        "EDITOR",
        "HOME",
        "HOSTNAME",
        "LOGNAME",
        "NO_COLOR",
        "PATH",
        "PATHEXT",
        "TERM",
        "TERM_PROGRAM",
        "USER",
        "USERNAME",
        "WT_SESSION",
        "XDG_CONFIG_HOME",
    ];
    expected.sort_unstable();
    assert_eq!(names, expected);
}

/// The call sites read their names from the table: no source file of the
/// workspace but the table spells a variable's name as a string literal. A
/// read typed as `std::env::var("NO_COLOR")` beside the table is the second
/// copy the table exists to prevent, and this names the file that holds it.
#[test]
fn no_call_site_spells_a_name_the_table_holds() {
    let names: Vec<&str> = ank_contract::env::VARIABLES
        .iter()
        .map(|v| v.name)
        .collect();
    assert!(names.len() > 20, "the table holds {} names", names.len());
    let mut found = Vec::new();
    for krate in std::fs::read_dir(workspace().join("crates"))
        .unwrap()
        .flatten()
    {
        let src = krate.path().join("src");
        if src.is_dir() {
            walk(&src, &names, &mut found);
        }
    }
    assert!(
        found.is_empty(),
        "these read a variable by a literal name instead of through ank_contract::env:\n{}",
        found.join("\n")
    );
}

fn walk(dir: &Path, names: &[&str], found: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, names, found);
            continue;
        }
        if path.extension().is_none_or(|e| e != "rs") || path.ends_with("ank-contract/src/env.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (n, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for name in names {
                if line.contains(&format!("\"{name}\"")) {
                    found.push(format!("{}:{}: {}", path.display(), n + 1, line.trim()));
                }
            }
        }
    }
}
