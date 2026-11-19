//! Under `--json` a refusal is one JSON document on stderr, and stdout stays
//! empty (TASK-f51e7f306830, ADR-4d9fb7375dc1, GitHub #782).
//!
//! **Through the binary**, because the criterion is about which bytes leave
//! the process on which stream, and only the process has streams.
//!
//! The documents are compared as literal strings rather than parsed: the
//! criterion names the document whole, key order included, and one writer
//! produces it (ADR-6fd69efb629c), so the bytes are the assertion a parser
//! would be weaker than.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The contract version every document carries, read off `help --json` rather
/// than typed, so a bump does not turn this suite red for the wrong reason.
fn contract() -> String {
    let out = Command::new(ANK).args(["help", "--json"]).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    let rest = text
        .strip_prefix("{\"contract\":")
        .unwrap_or_else(|| panic!("help --json does not lead with its contract: {text}"));
    rest.chars().take_while(|c| c.is_ascii_digit()).collect()
}

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("refusal-json-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

/// A git repository holding an empty corpus.
fn corpus(what: &str, config: &str) -> PathBuf {
    let dir = scratch::dir(what);
    let out = Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .output()
        .expect("git must be on PATH");
    assert!(out.status.success());
    fs::create_dir_all(dir.join(".ank/entities")).unwrap();
    fs::write(dir.join(".ank/config.yml"), config).unwrap();
    dir
}

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn ank(dir: &Path, args: &[&str]) -> Run {
    let out = Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .env("ANK_AGENT", "claude-code/refusal-json-it")
        .env_remove("NO_COLOR")
        .stdin(Stdio::null())
        .output()
        .expect("the binary under test must run");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8(out.stdout).unwrap(),
        stderr: String::from_utf8(out.stderr).unwrap(),
    }
}

/// The criterion's own example, whole.
#[test]
fn a_refusal_under_json_is_one_document_on_stderr_and_nothing_on_stdout() {
    let dir = corpus("refusal-json-show", "schema: 2\n");
    let run = ank(&dir, &["show", "NOPE", "--json"]);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert_eq!(
        run.stdout, "",
        "a refusal writes nothing where a parser reads"
    );
    assert_eq!(
        run.stderr,
        format!(
            "{{\"contract\":{},\"error\":{{\"code\":2,\"name\":\"NotFound\",\"message\":\"entity not found: NOPE\",\"next\":[\"ank find NOPE\"],\"subject\":\"NOPE\"}}}}\n",
            contract()
        )
    );
}

/// Two `->` lines in the prose are two entries of `next`, in the order the
/// prose prints them.
#[test]
fn a_refusal_with_two_next_lines_lists_both_in_order() {
    let dir = corpus(
        "refusal-json-two",
        "schema: 2\nverifiers:\n  unit:\n    run: \"true\"\n",
    );
    fs::write(dir.join("a.txt"), "a\n").unwrap();
    let args = [
        "new",
        "task",
        "--title",
        "t",
        "--scope",
        "a.txt",
        "--criteria",
        "c",
        "--verify",
        "nosuch",
    ];
    let prose = ank(&dir, &args);
    assert_eq!(
        prose.stderr,
        "error[7]: no verifier 'nosuch' in .ank/config.yml\n  -> declared: unit\n  -> or ank config verifiers.nosuch.run \"<command>\"\n",
        "the prose this test reads its expectation from"
    );
    let mut json = args.to_vec();
    json.push("--json");
    let run = ank(&dir, &json);
    assert_eq!(run.code, 7);
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        format!(
            "{{\"contract\":{},\"error\":{{\"code\":7,\"name\":\"Prerequisite\",\"message\":\"no verifier 'nosuch' in .ank/config.yml\",\"next\":[\"declared: unit\",\"or ank config verifiers.nosuch.run \\\"<command>\\\"\"],\"subject\":null}}}}\n",
            contract()
        )
    );
}

/// A refusal about no entity and no path carries `subject: null`, and one
/// with no `->` line carries an empty `next`. A flag the parser does not know
/// is refused before any verb runs, which is also the case where `--json` has
/// to be recognised without a parsed invocation to ask.
#[test]
fn a_refusal_about_nothing_in_particular_has_a_null_subject() {
    let dir = corpus("refusal-json-null", "schema: 2\n");
    let prose = ank(&dir, &["find", "--bogus"]);
    let message = prose
        .stderr
        .strip_prefix("error[1]: ")
        .and_then(|rest| rest.lines().next())
        .unwrap_or_else(|| panic!("not a code-1 refusal: {}", prose.stderr))
        .to_string();
    let next: Vec<String> = prose
        .stderr
        .lines()
        .filter_map(|l| l.strip_prefix("  -> "))
        .map(|n| format!("\"{n}\""))
        .collect();
    let run = ank(&dir, &["find", "--bogus", "--json"]);
    assert_eq!(run.code, 1);
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        format!(
            "{{\"contract\":{},\"error\":{{\"code\":1,\"name\":\"Generic\",\"message\":\"{message}\",\"next\":[{}],\"subject\":null}}}}\n",
            contract(),
            next.join(",")
        )
    );
}

/// A warning under `--json` is one document per line, keyed `warning`, and the
/// verb's answer stays alone on stdout.
#[test]
fn a_warning_under_json_is_one_document_per_line() {
    let dir = corpus("refusal-json-warning", "schema: 2\nrequires_ank: 999.0.0\n");
    let prose = ank(&dir, &["find", "x"]);
    let message = prose
        .stderr
        .lines()
        .next()
        .and_then(|l| l.strip_prefix("warning: "))
        .unwrap_or_else(|| panic!("no warning in prose: {}", prose.stderr))
        .to_string();
    assert!(message.starts_with("this corpus requires ank 999.0.0"));
    let run = ank(&dir, &["find", "x", "--json"]);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.starts_with("{\"contract\":"), "{}", run.stdout);
    assert_eq!(
        run.stderr,
        format!(
            "{{\"contract\":{},\"warning\":{{\"message\":\"{message}\",\"next\":[\"ank update\"],\"subject\":null}}}}\n",
            contract()
        )
    );
}

/// Without `--json` nothing changed: the prose is what it was, byte for byte.
#[test]
fn without_json_a_refusal_is_the_prose_it_was() {
    let dir = corpus("refusal-json-prose", "schema: 2\n");
    let run = ank(&dir, &["show", "NOPE"]);
    assert_eq!(run.code, 2);
    assert_eq!(run.stdout, "");
    assert_eq!(
        run.stderr,
        "error[2]: entity not found: NOPE\n  -> ank find NOPE\n"
    );
    let warned = corpus(
        "refusal-json-prose-warning",
        "schema: 2\nrequires_ank: 999.0.0\n",
    );
    let run = ank(&warned, &["find", "x"]);
    assert!(
        run.stderr
            .starts_with("warning: this corpus requires ank 999.0.0 or newer, and this binary is "),
        "{}",
        run.stderr
    );
    assert!(
        run.stderr.ends_with("\n  -> ank update\n"),
        "{}",
        run.stderr
    );
}
