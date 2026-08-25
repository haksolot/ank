//! The one road to the corpus: running `ank <verb> --json` (ADR-8bd76e8d7c4e).
//!
//! Four verbs are reached from here and they are the four that only read --
//! `status`, `find`, `show` and `scope`. That list is not a policy stated in
//! prose: it is [`READS`], and [`Ank::json`] refuses a verb the list does not
//! carry before spawning anything, so a later edit that reached for `claim`
//! would fail here rather than write.
//!
//! **The document is read with `serde_yaml`, not with a parser written here.**
//! YAML 1.2 is a superset of JSON, `ank-mcp` already reads JSON-RPC that way,
//! and the escaper on the other side (`ank_contract::json::string`) emits only
//! `\"`, `\\`, `\n` and `\u00xx` -- every one of which a double-quoted YAML
//! scalar carries with the same meaning. So no crate enters the tree for this
//! and there is no second escaping convention to keep in step.

use crate::Address;
use ank_contract::ExitCode;
use std::fmt;
use std::process::Command;

pub use serde_yaml::Value;

/// The verbs this reader may run, and the whole of them.
///
/// Every one of them only reads: `status`, `find`, `show` and `scope` write no
/// file and no ref (§4). The reader therefore cannot write by accident, and the
/// property is enforced one function below rather than asserted in a comment.
pub const READS: &[&str] = &["status", "find", "show", "scope"];

/// A call that did not produce a document, in the three ways it can fail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failed {
    /// The CLI could not be run at all: the binary moved, or the directory it
    /// was to run in went away. An environment to repair.
    Spawn { args: String, error: String },
    /// The CLI ran and refused. The code and the bytes are the CLI's own, which
    /// is the whole point of this crate: a refusal shown here is a refusal the
    /// binary gave.
    Refused {
        args: String,
        code: i32,
        stderr: String,
    },
    /// The CLI answered and the answer did not parse. A defect on this side of
    /// the pipe, and it is reported as one rather than as an empty screen.
    Unreadable { args: String, error: String },
    /// A verb outside [`READS`]. Never reached from a running reader; reached
    /// by the next edit that tries to make one write.
    NotARead { verb: String },
}

impl Failed {
    /// The code a caller exits with, when this failure is what ends a session.
    ///
    /// A refusal carries the CLI's own number through untouched where it is one
    /// of the ten (§4); anything else is an environment to repair, which is what
    /// a missing binary and an unparseable answer both are.
    pub fn code(&self) -> ExitCode {
        match self {
            Failed::Refused { code, .. } => from_i32(*code),
            _ => ExitCode::Environment,
        }
    }
}

/// The exit code of §4 a number stands for, or [`ExitCode::Generic`].
///
/// Written as a list rather than transmuted: the discriminants are the
/// contract, and a `transmute` back would turn a number outside the table into
/// a variant that does not exist.
fn from_i32(code: i32) -> ExitCode {
    for known in [
        ExitCode::Ok,
        ExitCode::Generic,
        ExitCode::NotFound,
        ExitCode::Conflict,
        ExitCode::Unavailable,
        ExitCode::Proof,
        ExitCode::Transition,
        ExitCode::Prerequisite,
        ExitCode::Findings,
        ExitCode::Environment,
    ] {
        if known.code() == code {
            return known;
        }
    }
    ExitCode::Generic
}

impl fmt::Display for Failed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Failed::Spawn { args, error } => write!(f, "cannot run `ank {args}`: {error}"),
            Failed::Refused { args, code, stderr } => {
                let said = stderr.trim();
                if said.is_empty() {
                    write!(f, "`ank {args}` refused with code {code}")
                } else {
                    write!(f, "{said}")
                }
            }
            Failed::Unreadable { args, error } => {
                write!(
                    f,
                    "`ank {args} --json` answered a document this reader cannot read: {error}"
                )
            }
            Failed::NotARead { verb } => {
                write!(f, "'{verb}' is not one of the verbs this reader may run")
            }
        }
    }
}

/// The CLI, addressed.
#[derive(Debug, Clone)]
pub struct Ank {
    address: Address,
}

impl Ank {
    pub fn new(address: Address) -> Ank {
        Ank { address }
    }

    /// One `--json` document, or why there is none.
    ///
    /// `--json` is appended here and never by a caller, so no call site can
    /// forget it and read a human page by mistake. The address flags go in
    /// front of the verb's own arguments, which is where a caller typed them.
    pub fn json(&self, verb: &str, args: &[&str]) -> Result<Value, Failed> {
        if !READS.contains(&verb) {
            return Err(Failed::NotARead {
                verb: verb.to_string(),
            });
        }
        let mut argv: Vec<String> = self.address.flags();
        argv.push(verb.to_string());
        argv.extend(args.iter().map(|a| a.to_string()));
        argv.push("--json".to_string());
        let shown = argv.join(" ");
        let out = Command::new(&self.address.exe)
            .args(&argv)
            .current_dir(&self.address.cwd)
            .output()
            .map_err(|e| Failed::Spawn {
                args: shown.clone(),
                error: e.to_string(),
            })?;
        if !out.status.success() {
            return Err(Failed::Refused {
                args: shown,
                code: out.status.code().unwrap_or(ExitCode::Generic.code()),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        serde_yaml::from_str(&text).map_err(|e| Failed::Unreadable {
            args: shown,
            error: e.to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Reading a document
// ---------------------------------------------------------------------------
//
// Four helpers, because a `Value` walked with `and_then` at forty call sites is
// forty chances to spell the same absence differently. A field the document
// does not carry reads as empty and never as a panic: the contract allows a
// document to *gain* a field (ADR-6fd69efb629c), and a reader that fell over on
// one it did not expect would be the strict parser that ADR warns against.

/// A string field, or `""`.
pub fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string()
}

/// A string field, or `None` where it is absent or null.
pub fn maybe(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// An array field, or an empty slice.
pub fn rows<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(|v| v.as_sequence())
        .map(|s| s.as_slice())
        .unwrap_or(&[])
}

/// A numeric field, or zero.
pub fn count(value: &Value, key: &str) -> u64 {
    value.get(key).and_then(|v| v.as_u64()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(s: &str) -> Value {
        serde_yaml::from_str(s).expect("the CLI's own shape must parse")
    }

    /// The premise this crate rests on: the CLI's escaper and a YAML reader
    /// agree. If they ever stop agreeing, this is where it shows.
    #[test]
    fn the_clis_json_reads_as_flow_yaml() {
        let line = r#"{"contract":1,"total":2,"results":[{"id":"TASK-0001","title":"a \"quoted\" title"},{"id":"ADR-0002","title":"two\nlines and a tab 	 in the middle"}]}"#;
        let v = doc(line);
        assert_eq!(count(&v, "contract"), 1);
        assert_eq!(rows(&v, "results").len(), 2);
        assert_eq!(text(&rows(&v, "results")[0], "title"), "a \"quoted\" title");
        assert_eq!(
            text(&rows(&v, "results")[1], "title"),
            "two\nlines and a tab \t in the middle"
        );
    }

    #[test]
    fn an_absent_field_is_empty_and_never_a_panic() {
        let v = doc(r#"{"a":"x","n":null}"#);
        assert_eq!(text(&v, "missing"), "");
        assert_eq!(maybe(&v, "missing"), None);
        assert_eq!(maybe(&v, "n"), None, "null is absence, not a value");
        assert!(rows(&v, "missing").is_empty());
        assert_eq!(count(&v, "missing"), 0);
    }

    /// A field the reader never asked about does not disturb it, which is what
    /// the contract promises within a version (ADR-6fd69efb629c).
    #[test]
    fn a_document_that_gained_a_field_still_reads() {
        let v = doc(r#"{"contract":1,"branch":"main","invented_later":{"deep":[1,2]}}"#);
        assert_eq!(text(&v, "branch"), "main");
    }

    #[test]
    fn a_verb_that_writes_is_refused_before_anything_is_spawned() {
        let ank = Ank::new(Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        });
        // `/nonexistent/ank` would fail to spawn, so reaching `NotARead` proves
        // the check happens first.
        assert_eq!(
            ank.json("claim", &["TASK-0001"]),
            Err(Failed::NotARead {
                verb: "claim".to_string()
            })
        );
        for verb in READS {
            assert!(
                !matches!(ank.json(verb, &[]), Err(Failed::NotARead { .. })),
                "{verb} is a read and must pass the gate"
            );
        }
    }

    #[test]
    fn a_refusal_carries_the_clis_own_code_and_bytes() {
        let f = Failed::Refused {
            args: "show TASK-0001".to_string(),
            code: 2,
            stderr: "error[2]: no entity matches 'TASK-0001'\n> ank find TASK\n".to_string(),
        };
        assert_eq!(f.code(), ExitCode::NotFound);
        assert!(f.to_string().starts_with("error[2]:"), "{f}");
        // A code outside the table is not invented into a variant.
        assert_eq!(
            Failed::Refused {
                args: String::new(),
                code: 42,
                stderr: String::new()
            }
            .code(),
            ExitCode::Generic
        );
    }
}
