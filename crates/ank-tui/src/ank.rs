//! The one road to the corpus: running `ank <verb> --json` (ADR-8bd76e8d7c4e).
//!
//! Eleven verbs are reached from here, in two lists that are policy made
//! mechanical rather than policy stated in prose. [`READS`] is the five that
//! only read -- `status`, `find`, `show`, `scope` and `review` -- and
//! [`Ank::json`] refuses anything else before spawning. [`ACTS`] is the six the
//! person at the keyboard may ask for -- `claim`, `log`, `release`, `done`,
//! `amend` and `accept` -- and [`Ank::act`] refuses anything else the same way.
//! Two gates and not one, because the difference between them is the whole of
//! what a reader is allowed to do on its own: a screen repaints by reading, and
//! it writes only where a command was typed.
//!
//! **`accept` is on the acting list, and the reader still never performs one**
//! (TASK-d90e94afca08). ADR-8bd76e8d7c4e lets the reader *drive* a
//! ratification and forbids it performing one unattended, and the line between
//! those two words is drawn here rather than described: driving is spawning
//! `ank accept <id>`, which is the same road `claim` takes and runs only where
//! a word was typed whole. Performing would be supplying a key or answering a
//! passphrase prompt, and [`Ank::spawn`] gives every child `output()`'s null
//! stdin -- so this process has no channel through which a secret could reach
//! one, whatever a later edit intends. The signature is git's and the person's,
//! and it is unreachable from here by construction rather than by restraint.
//!
//! The four that must stay out of both lists are `close`, `check`, `attest` and
//! `init`, and a verb absent from a list is absent silently -- so the absence
//! is asserted below rather than left to whoever reads the two constants next.
//!
//! **An act runs with `--json` like a read does.** ADR-8bd76e8d7c4e and
//! SPEC-93531977642f both say the reader reaches the corpus only by running the
//! CLI with `--json`, and dropping the flag to get a human sentence back would
//! be reading that as decoration. Nothing is lost by keeping it: the CLI renders
//! `error[N]:` and the command that resolves it on stderr whatever `--json`
//! says, so a refusal arrives in the CLI's own bytes either way.
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

/// The verbs this reader may run on its own, and the whole of them.
///
/// Every one of them only reads: `status`, `find`, `show`, `scope` and `review`
/// write no file and no ref (§4). Repainting the screen therefore cannot write
/// by accident, and the property is enforced one function below rather than
/// asserted in a comment.
///
/// **`review` is here and not on the acting list**, which is the answer to the
/// obvious objection that a ratification queue sounds like part of ratifying.
/// The verb answers a question and changes nothing: §4 gives it
/// `renews: Never` and `coordinates: false`, so a screen that redraws the queue
/// on a watcher's news renews no lease and takes no ref -- which is what lets
/// the queue be repainted at all (ADR-0bb7ea8991bc).
pub const READS: &[&str] = &["status", "find", "show", "scope", "review"];

/// The verbs whose exit 8 carries a document rather than a refusal.
///
/// `review` shares `check`'s report and therefore its exit code, and §4 says so
/// on the verb: findings leave 8, a signal alone leaves 0. The document is
/// written all the same, so a reader that read 8 as a refusal would show an
/// empty queue to every corpus with a fault in it -- which is the corpus most
/// in need of one. Named as a list rather than tolerated everywhere: `find`
/// and `status` never answer 8, and a blanket rule would silently swallow the
/// day one of them starts to.
const FINDINGS_ARE_AN_ANSWER: &[&str] = &["review"];

/// The verbs this reader may run *because somebody typed one*, and the whole of
/// them.
///
/// The writing half of the loop, against the entity under the cursor. Each is
/// run as a shell would run it, which is what keeps ADR-052accd6e3b2 naming an
/// intersecting claim and ADR-0bb7ea8991bc holding that a screen left open all
/// night renews nothing: there is no second dispatch path here for either rule
/// to be reimplemented on.
///
/// **`accept` is the sixth, and it arrived by a decision rather than by being
/// typed into the list** (TASK-d90e94afca08). It is here because driving a
/// ratification *is* spawning the verb, and the reader has no other way to do
/// it; what keeps it a human act is stated in the module header and enforced in
/// three places that are not this one -- the grammar takes no tail after the
/// word, so nothing beyond the single identifier is ever passed; `input::parse`
/// takes it only where the document is open on the screen; and the child is
/// given no stdin, so no prompt of git's can be answered from this process.
///
/// `close`, `check`, `attest` and `init` are not here and must not arrive.
pub const ACTS: &[&str] = &["claim", "log", "release", "done", "amend", "accept"];

/// A call that did not produce a document, in the three ways it can fail.
///
/// The `shown` every variant carries is [`Ank::spelling`]'s: the command line
/// as a shell would have had to spell it, program word and `--json` included.
/// One spelling for a refusal, for the chrome over an answer and for the
/// confirmation shown before a write, because three renderings of one command
/// line are three chances for the screen to name something other than what ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failed {
    /// The CLI could not be run at all: the binary moved, or the directory it
    /// was to run in went away. An environment to repair.
    Spawn { shown: String, error: String },
    /// The CLI ran and refused. The code and the bytes are the CLI's own, which
    /// is the whole point of this crate: a refusal shown here is a refusal the
    /// binary gave.
    Refused {
        shown: String,
        code: i32,
        stderr: String,
    },
    /// The CLI answered and the answer did not parse. A defect on this side of
    /// the pipe, and it is reported as one rather than as an empty screen.
    Unreadable { shown: String, error: String },
    /// A verb outside [`READS`], asked for on the reading road. Never reached
    /// from a running reader; reached by the next edit that tries to make a
    /// repaint write.
    NotARead { verb: String },
    /// A verb outside [`ACTS`], asked for on the acting road. Reached by the
    /// next edit that adds a command for a verb the list does not carry --
    /// `accept` above all, which is the one this gate exists for.
    NotAnAct { verb: String },
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
            Failed::Spawn { shown, error } => write!(f, "cannot run `{shown}`: {error}"),
            Failed::Refused {
                shown,
                code,
                stderr,
            } => {
                let said = stderr.trim();
                if said.is_empty() {
                    write!(f, "`{shown}` refused with code {code}")
                } else {
                    write!(f, "{said}")
                }
            }
            Failed::Unreadable { shown, error } => {
                write!(
                    f,
                    "`{shown}` answered a document this reader cannot read: {error}"
                )
            }
            Failed::NotARead { verb } => {
                write!(
                    f,
                    "'{verb}' is not one of the verbs this reader may read with"
                )
            }
            Failed::NotAnAct { verb } => {
                write!(
                    f,
                    "'{verb}' is not one of the verbs this reader may act with"
                )
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

    /// One `--json` document off a verb that only reads, or why there is none.
    ///
    /// The gate is [`READS`] and it is checked before anything is spawned, so a
    /// later edit that reached for `claim` from a repaint would fail here rather
    /// than write.
    pub fn json(&self, verb: &str, args: &[&str]) -> Result<Value, Failed> {
        if !READS.contains(&verb) {
            return Err(Failed::NotARead {
                verb: verb.to_string(),
            });
        }
        let owned: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        Ok(self.spawn(verb, &owned)?.answered)
    }

    /// One verb of the writing half, run because somebody typed it.
    ///
    /// The gate is [`ACTS`], checked before anything is spawned and for the same
    /// reason [`READS`] is: the verbs a reader may reach are a list, and a list
    /// is what an edit has to change in the open.
    ///
    /// What comes back is [`Ran`]: the command as it was run, and the document
    /// the CLI answered. Both are wanted by the caller -- one is the chrome a
    /// screen puts over the answer, the other is the answer -- and a caller that
    /// rebuilt the first from what it asked for could rebuild it wrongly.
    pub fn act(&self, verb: &str, args: &[String]) -> Result<Ran, Failed> {
        if !ACTS.contains(&verb) {
            return Err(Failed::NotAnAct {
                verb: verb.to_string(),
            });
        }
        self.spawn(verb, args)
    }

    /// The argv one call is made with, and the only place one is composed.
    ///
    /// `--json` is appended here and never by a caller, so no call site can
    /// forget it and read a human page by mistake. The address flags go in
    /// front of the verb's own arguments, which is where a caller typed them.
    ///
    /// Separate from [`Ank::spawn`] because [`Ank::spelling`] has to answer
    /// *before* a spawn: the confirmation TASK-d4a882345837 puts in front of
    /// every write shows a command line, and a command line composed twice is
    /// two command lines. This one is composed once and read by both.
    fn argv(&self, verb: &str, args: &[String]) -> Vec<String> {
        let mut argv: Vec<String> = self.address.flags();
        argv.push(verb.to_string());
        argv.extend(args.iter().cloned());
        argv.push("--json".to_string());
        argv
    }

    /// One call, spelled as a shell would have to spell it.
    ///
    /// **This is what a person is shown before anything is spawned**
    /// (TASK-d4a882345837), and it is also the chrome over the answer and the
    /// name in a refusal -- one spelling for all three, because a confirmation
    /// that spelled a command differently from the way it ran would be a
    /// confirmation of something else.
    ///
    /// "As a shell would have to spell it" is [`quoted`]'s whole job and it is
    /// not decoration: a `log` message is one argument with spaces in it, and
    /// showing it bare would show a command line that, typed back into a
    /// terminal, would run something different. The program word is `ank` --
    /// the word a person types -- rather than the absolute path this process
    /// resolved, because the point of the line is that it is checkable against
    /// what they could have typed themselves.
    pub fn spelling(&self, verb: &str, args: &[String]) -> String {
        let mut line = String::from("ank");
        for word in self.argv(verb, args) {
            line.push(' ');
            line.push_str(&quoted(&word));
        }
        line
    }

    /// The one road out of this process, and there is deliberately only one.
    fn spawn(&self, verb: &str, args: &[String]) -> Result<Ran, Failed> {
        let argv = self.argv(verb, args);
        let shown = self.spelling(verb, args);
        let out = Command::new(&self.address.exe)
            .args(&argv)
            .current_dir(&self.address.cwd)
            .output()
            .map_err(|e| Failed::Spawn {
                shown: shown.clone(),
                error: e.to_string(),
            })?;
        if !out.status.success() && !answered_with_findings(verb, &out.status) {
            return Err(Failed::Refused {
                shown,
                code: out.status.code().unwrap_or(ExitCode::Generic.code()),
                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            });
        }
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let answered = serde_yaml::from_str(&text).map_err(|e| Failed::Unreadable {
            shown: shown.clone(),
            error: e.to_string(),
        })?;
        Ok(Ran { shown, answered })
    }
}

/// One word of a command line, quoted where a shell would need it quoted.
///
/// The rule is the conservative one and it is deliberately not clever: a word
/// made only of characters no shell gives a meaning to is left alone, and
/// everything else is wrapped in single quotes, where the only byte that still
/// means anything is the closing quote itself -- so an embedded `'` leaves the
/// quoting, is escaped, and comes back in. That is `'\''`, which is what every
/// POSIX shell reads back as one apostrophe.
///
/// Single quotes rather than double, because inside double quotes `$`, a
/// backtick and a backslash all still act, and a criterion carrying a `$` is
/// not a hypothetical in a corpus whose scopes are globs. An empty word is
/// `''`: `--reason ""` is a caller saying something, and a word that vanished
/// would shift every argument after it.
pub fn quoted(word: &str) -> String {
    const SAFE: &str = "_@%+=:,./-";
    let plain = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || SAFE.contains(c));
    if plain {
        return word.to_string();
    }
    let mut out = String::from("'");
    for c in word.chars() {
        match c {
            '\'' => out.push_str("'\\''"),
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Whether a non-zero exit is this verb saying "there are findings" (§4).
fn answered_with_findings(verb: &str, status: &std::process::ExitStatus) -> bool {
    FINDINGS_ARE_AN_ANSWER.contains(&verb) && status.code() == Some(ExitCode::Findings.code())
}

/// A call that answered: the command line it was, and the document it gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// The invocation, spelled as a shell would have had to spell it
    /// ([`Ank::spelling`]). Shown above the answer so that what a screen
    /// reports is checkable against what a person could have typed themselves
    /// -- and it is the same string the confirmation showed before the verb
    /// ran, because both ask one function.
    pub shown: String,
    pub answered: Value,
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

    fn nowhere() -> Ank {
        Ank::new(Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: None,
            worktree: None,
        })
    }

    #[test]
    fn a_verb_that_writes_is_refused_before_anything_is_spawned() {
        let ank = nowhere();
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

    /// The acting gate, and the same proof that it comes first.
    ///
    /// **`accept` left this list and the other four did not** (TASK-d90e94afca08).
    /// The reader drives a ratification now, so the verb reaches the spawn; that
    /// is a decision about one verb, and it says nothing whatever about `close`,
    /// `check`, `attest` or `init`, which stay out for the reasons they always
    /// had. The list below is the whole of what widened, and it widened by one.
    #[test]
    fn a_verb_outside_the_acting_list_is_refused_before_anything_is_spawned() {
        let ank = nowhere();
        for verb in ["close", "check", "attest", "init"] {
            assert_eq!(
                ank.act(verb, &["TASK-0001".to_string()]),
                Err(Failed::NotAnAct {
                    verb: verb.to_string()
                }),
                "{verb} reached the spawn"
            );
        }
        for verb in ACTS {
            assert!(
                !matches!(ank.act(verb, &[]), Err(Failed::NotAnAct { .. })),
                "{verb} is an act and must pass the gate"
            );
        }
    }

    /// `accept` is an act and never a read, and the two lists do not overlap.
    ///
    /// The half that matters most is the first line. A repaint calls
    /// [`Ank::json`] and nothing else, so `accept` being absent from [`READS`]
    /// is what makes "the reader never performs one unattended" a property of
    /// the code: there is no road from a watcher's news to this verb, whatever
    /// a screen is showing when the news arrives.
    #[test]
    fn accept_is_an_act_and_never_a_read() {
        let ank = nowhere();
        assert!(!READS.contains(&"accept"));
        assert_eq!(
            ank.json("accept", &["ADR-0001"]),
            Err(Failed::NotARead {
                verb: "accept".to_string()
            }),
            "a repaint reached accept"
        );
        assert!(ACTS.contains(&"accept"));
        for verb in ACTS {
            assert!(
                !READS.contains(verb),
                "{verb} is on both roads, and the gates then say nothing"
            );
        }
    }

    /// `review` is a read, and the four the reader may never run stay out of
    /// both lists.
    #[test]
    fn review_reads_and_the_four_that_must_stay_out_are_on_neither_road() {
        assert!(
            READS.contains(&"review"),
            "the queue is read by asking review"
        );
        for verb in ["close", "check", "attest", "init"] {
            assert!(
                !READS.contains(&verb),
                "{verb} is not a read of this reader"
            );
            assert!(!ACTS.contains(&verb), "{verb} is not an act of this reader");
        }
    }

    /// A verb that answers 8 with a document is not refused (§4).
    ///
    /// Asserted on the table rather than on a spawn: what the reader has to get
    /// right is which verbs answer that way, and `review` is the one -- a
    /// corpus with a fault in it is exactly the corpus whose queue a person
    /// most needs to see.
    #[test]
    fn a_verb_that_answers_with_findings_is_the_queue_and_only_the_queue() {
        assert_eq!(FINDINGS_ARE_AN_ANSWER, &["review"]);
        for verb in FINDINGS_ARE_AN_ANSWER {
            assert!(
                READS.contains(verb),
                "{verb} tolerates 8 on a road it is not on"
            );
        }
    }

    #[test]
    fn a_refusal_carries_the_clis_own_code_and_bytes() {
        let f = Failed::Refused {
            shown: "ank show TASK-0001 --json".to_string(),
            code: 2,
            stderr: "error[2]: no entity matches 'TASK-0001'\n> ank find TASK\n".to_string(),
        };
        assert_eq!(f.code(), ExitCode::NotFound);
        assert!(f.to_string().starts_with("error[2]:"), "{f}");
        // A code outside the table is not invented into a variant.
        assert_eq!(
            Failed::Refused {
                shown: String::new(),
                code: 42,
                stderr: String::new()
            }
            .code(),
            ExitCode::Generic
        );
    }

    /// A word a shell would read as one word is left alone, and everything else
    /// is quoted so that reading the line back gives the same argument
    /// (TASK-d4a882345837).
    ///
    /// The cases are the ones this reader actually composes: an identifier, a
    /// proof, a flag, a scope glob, a `log` message, and the empty argument a
    /// `--reason ""` is.
    #[test]
    fn a_word_is_spelled_the_way_a_shell_would_have_to_spell_it() {
        for plain in [
            "claim",
            "--proof",
            "TASK-d4a882345837",
            "commit:2d9c847",
            "crates/ank-tui/src/view.rs",
            "4h",
        ] {
            assert_eq!(quoted(plain), plain, "{plain} was quoted for nothing");
        }
        assert_eq!(quoted(""), "''", "an empty argument is still an argument");
        assert_eq!(quoted("two words"), "'two words'");
        assert_eq!(quoted("crates/ank tui/**"), "'crates/ank tui/**'");
        assert_eq!(quoted("$HOME `id`"), "'$HOME `id`'");
        // The one byte that still means something inside single quotes.
        assert_eq!(quoted("it's"), "'it'\\''s'");
        assert_eq!(quoted("'"), "''\\'''");
    }

    /// The confirmation and the chrome over the answer are one string, composed
    /// once (TASK-d4a882345837).
    ///
    /// This is the property the whole confirmation rests on: what a person is
    /// shown before a write is spelled by the same function that spells what
    /// ran, from the same argv the child is given. A second composition here
    /// would be a screen that can name something other than what it spawned.
    #[test]
    fn the_spelling_shown_is_the_argv_the_child_is_given() {
        let ank = nowhere();
        let args = vec![
            "TASK-49746735127f".to_string(),
            "a message with spaces".to_string(),
        ];
        assert_eq!(
            ank.spelling("log", &args),
            "ank log TASK-49746735127f 'a message with spaces' --json"
        );
        // Word for word, the argv and the spelling are the same call: the
        // program word, then every argument quoted.
        let spelled: Vec<String> = ank.argv("log", &args).iter().map(|w| quoted(w)).collect();
        assert_eq!(
            ank.spelling("log", &args),
            format!("ank {}", spelled.join(" "))
        );
    }

    /// The caller's own address flags are in the line, in front of the verb,
    /// because they are in the argv.
    #[test]
    fn the_spelling_carries_the_address_the_child_is_given() {
        let ank = Ank::new(Address {
            exe: "/nonexistent/ank".into(),
            cwd: ".".into(),
            repo: Some("/two words".to_string()),
            worktree: None,
        });
        assert_eq!(
            ank.spelling("claim", &["TASK-0001".to_string()]),
            "ank --repo '/two words' claim TASK-0001 --json"
        );
    }
}
