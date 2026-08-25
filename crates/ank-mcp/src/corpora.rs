//! Which corpus a call is addressed to, and which corpora exist to be addressed
//! (ADR-fd98f4bc6dea, ADR-621a7fd96ce1, ADR-96174f1ac2b7).
//!
//! **Multiplexing, and never a merged claim space.** The clause this module
//! implements supersedes "it speaks for exactly one corpus", and it supersedes
//! only that: every call still becomes `ank --repo <one corpus> <verb> --json`,
//! every claim is still arbitrated by the refs of the clone it was taken in, and
//! nothing here ever holds two corpora in one thought. There is no set operation
//! anywhere below, no claim held on a client's behalf, and no pooling of clients
//! under one identity. What a call names is *one* corpus, and what this module
//! does is turn a name into the one path [`crate::call`] hands to `--repo`.
//!
//! **Declared, never discovered.** The set a server may reach is what the reader
//! declared in `corpora.yml` plus the corpus the process was addressed with at
//! startup. Nothing walks a filesystem looking for a corpus, nothing reads a
//! remote, and nothing derives a location from a path or a slug -- the rule
//! ADR-621a7fd96ce1 states and ADR-96174f1ac2b7 restates, for the reason
//! ADR-a1de673043b4 gives about peers: inference is how a corpus starts
//! depending on where somebody happened to check something out. An identity
//! nobody declared is refused by name, and refusing is the point: a corpus
//! quietly not found is the one outcome worse than a refusal.
//!
//! **A name is an identity and never a path**, which is what keeps a declared
//! set from becoming a merged one. `--repo` stays the server's flag and stays
//! refused; a caller says which corpus by naming the root commit of one, and a
//! root commit that nobody declared resolves to nothing. A caller that could
//! write a path here would reach every corpus on the machine, which is the shape
//! the ban exists to prevent.
//!
//! **Nothing is read until a call names a corpus.** A client that never passes
//! the argument is a single-corpus client, and it reads no declaration file,
//! asks the binary no question, and sees byte for byte what it saw before this
//! module existed. That is a property of the code below rather than a hope:
//! both halves of the reachable set sit behind a [`OnceCell`] that the absent
//! argument never touches.

use crate::call::{Arguments, Outcome};
use crate::Address;
use ank_contract::ExitCode;
use std::cell::OnceCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The argument every tool carries, and the one argument of this surface that
/// is not a flag of the verb.
///
/// It is not `--repo` renamed. `--repo` takes a path and belongs to the server;
/// this takes an identity and belongs to the caller, and the difference is the
/// whole of what makes several corpora a set the reader declared rather than
/// every directory on the machine.
pub const ARGUMENT: &str = "corpus";

/// What a client reads beside the argument in every tool's schema.
///
/// It names the command that prints the identity, because a value a caller
/// cannot obtain is an argument that does not exist. `ank status --json` is
/// where ADR-96174f1ac2b7 already sends a reader looking for a key.
pub const HELP: &str = "Optional. The corpus this call runs against, named by the repository \
                        identity of ADR-621a7fd96ce1 -- the root commit, which ank status --json \
                        prints under \"corpus\". Absent, the call goes to the corpus this server \
                        was addressed with at startup. Reachable are that corpus and the corpora \
                        the reader declared in corpora.yml; an identity nobody declared is \
                        refused by name, and nothing is discovered.";

/// The reader's map of repository identity to corpus, under the same home
/// `ank config --user` writes.
///
/// **The literal, and not a constant borrowed from `ank-cli`.** This crate does
/// not link the dispatch (ADR-fd98f4bc6dea) and will not start in order to
/// share a file name. What keeps the two spellings from drifting is not this
/// comment: `two_corpora_through_one_server_land_two_claims_and_no_third` in
/// `crates/ank-cli/tests/mcp.rs` writes this file with the CLI's own verb and
/// then names both corpora through this surface, so a name that stopped
/// matching would fail there rather than in prose.
const CORPORA_FILE: &str = "corpora.yml";

/// The schema of that file, which `ank config --user` writes and refuses to
/// read at any other number.
const SUPPORTED_SCHEMA: u64 = 1;

/// A repository identity as ADR-621a7fd96ce1 defines it: the root commit, and
/// therefore forty lowercase hex characters.
///
/// Applied to what a *caller* passes and not only to what a file holds, which
/// is the check that makes this argument an identity rather than a location. A
/// path, a remote URL or a slug fails it, and each of those is the mistake
/// ADR-96174f1ac2b7 predicts by name.
fn is_identity(key: &str) -> bool {
    key.len() == 40
        && key
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// A path as this corpus renders one: forward slashes, on every platform.
///
/// The same rule `claim` applies to the same sentences, and for the reason
/// Windows CI taught it there: a message naming `C:/corpora/front` in one
/// clause and `C:\Users\...` in the next gives one directory two spellings, and
/// an agent keying on the corpus reads two spellings as two corpora.
fn rendered(path: &str) -> String {
    path.replace('\\', "/")
}

/// Where this reader's declarations live, or `None` where the environment names
/// no home.
///
/// The rule itself comes from `ank_contract::events::user_dir`, which is where
/// `ank-daemon` already reaches for it rather than keeping a second copy of
/// three lines of platform difference. A home that moved would otherwise move
/// for one surface and not the other.
fn corpora_path() -> Option<PathBuf> {
    ank_contract::events::user_dir().map(|d| d.join(CORPORA_FILE))
}

/// The declarations file as a sentence, for a hint that has to name it.
fn corpora_file() -> String {
    corpora_path()
        .map(|p| rendered(&p.display().to_string()))
        .unwrap_or_else(|| CORPORA_FILE.to_string())
}

/// A refusal this surface raises itself, in the vocabulary of §4.
///
/// **It carries an exit code because every other refusal a client sees does.**
/// ADR-fd98f4bc6dea requires the surface to carry the CLI's exit codes as the
/// reason for a refusal, and a client branching on `exitCode` must not have to
/// learn that some refusals arrive with one and some do not. So this renders
/// through [`Outcome::to_result`] -- the same renderer a spawned refusal goes
/// through -- and the two shapes cannot drift apart, because there is one.
///
/// **It is a result and not a JSON-RPC error**, which is the line
/// [`Outcome::refused`] already draws: a request naming a flag the verb does not
/// take is malformed and answers `-32602`, and a request naming a corpus is well
/// formed. The answer is no, and no is a fact about what this reader declared.
pub struct Refusal {
    pub code: ExitCode,
    pub message: String,
    pub hint: String,
}

impl Refusal {
    /// The refusal as a call's outcome: nothing ran, and the exit code says what
    /// would have refused it.
    ///
    /// Written on standard error's side of the outcome because that is where a
    /// refusal leaves the binary (§4), so a client reading `stderr` or reading
    /// the text block finds the same sentence either way.
    pub fn outcome(&self) -> Outcome {
        Outcome {
            code: self.code.code(),
            stdout: String::new(),
            stderr: format!("error[{}]: {}\n  -> {}", self.code, self.message, self.hint),
        }
    }
}

/// What the reader declared, and why the map is empty when it is.
///
/// The reason is kept rather than dropped because it is the difference between
/// two refusals a caller would otherwise have to tell apart by guessing: an
/// identity nobody declared, and an identity declared in a file that could not
/// be read. Both refuse; only one of them is repaired by writing a declaration.
struct Declared {
    map: BTreeMap<String, String>,
    unreadable: Option<String>,
}

impl Declared {
    fn empty() -> Declared {
        Declared {
            map: BTreeMap::new(),
            unreadable: None,
        }
    }

    fn broken(why: String) -> Declared {
        Declared {
            map: BTreeMap::new(),
            unreadable: Some(why),
        }
    }
}

/// The corpora one server may address, resolved on demand and never discovered.
///
/// **Both halves are lazy, and that is the backwards compatibility clause.** A
/// client that passes no `corpus` argument is served by [`Address::repo`] alone:
/// no file is opened, no process is spawned, and the reachable set is never
/// built at all. A client that passes one pays for the map the first time and
/// for the startup corpus's own name at most once per session.
pub struct Reach<'a> {
    address: &'a Address,
    declared: OnceCell<Declared>,
    /// The identity of the corpus the process was addressed with, asked of the
    /// binary once. `None` where the corpus has no root commit to be keyed on,
    /// which ADR-621a7fd96ce1 calls the honest answer rather than a fabricated
    /// one.
    startup: OnceCell<Option<String>>,
}

impl<'a> Reach<'a> {
    pub fn new(address: &'a Address) -> Reach<'a> {
        Reach {
            address,
            declared: OnceCell::new(),
            startup: OnceCell::new(),
        }
    }

    /// Where the surface runs and what it runs, unchanged by any of this.
    pub fn address(&self) -> &Address {
        self.address
    }

    /// The corpus a call is addressed to: the one it named, or the one the
    /// process was addressed with.
    ///
    /// **One corpus comes out, always.** Nothing above this function sees a set,
    /// and nothing below it sees more than a path — which is what keeps the
    /// merged claim space unreachable by construction rather than by review.
    ///
    /// The order is the reader's declaration first, then the startup corpus,
    /// because ADR-96174f1ac2b7 makes the declaration the precedence and because
    /// a map lookup costs nothing where naming the startup corpus costs a
    /// process.
    pub fn resolve(&self, named: Option<&str>) -> Result<PathBuf, Refusal> {
        let Some(named) = named else {
            return Ok(self.address.repo.clone());
        };
        let named = named.trim();
        if !is_identity(named) {
            return Err(not_an_identity(named));
        }
        if let Some(declared) = self.declared().map.get(named) {
            let root = PathBuf::from(declared);
            // **The declaration is confronted with the filesystem here, in the
            // CLI's own sentence.** A directory that is not there would
            // otherwise fail as a process that could not be started, which
            // reports the environment where the fact is a stale entry in a map.
            // A directory that is there and carries no corpus is left to the
            // binary, whose refusal for exactly that is the one a client should
            // see (ADR-fd98f4bc6dea).
            if !root.is_dir() {
                return Err(not_there(named, declared));
            }
            return Ok(root);
        }
        if self.startup_identity() == Some(named) {
            return Ok(self.address.repo.clone());
        }
        Err(undeclared(named, self.declared().unreadable.as_deref()))
    }

    fn declared(&self) -> &Declared {
        self.declared.get_or_init(read_declarations)
    }

    /// The startup corpus's own identity, asked of the binary and cached.
    ///
    /// **Asked, because there is nothing else to ask.** The identity is a root
    /// commit, this crate touches no git and no `.ank/`, and `ank status --json`
    /// is where ADR-96174f1ac2b7 itself sends a reader looking for the key. So
    /// the server asks its own address for its own name, once, through the same
    /// road out of the process every call takes.
    ///
    /// **It is not a spawn of the caller's verb**, which is the thing a refusal
    /// must not perform: a call naming a corpus nobody declared runs nothing,
    /// takes no claim and touches no corpus. What runs is this one reading, of
    /// the corpus the server was already addressed with.
    fn startup_identity(&self) -> Option<&str> {
        self.startup
            .get_or_init(|| self.ask_its_own_name())
            .as_deref()
    }

    fn ask_its_own_name(&self) -> Option<String> {
        let spec = ank_contract::spec_of("status")?;
        let out = crate::call::run(
            spec,
            self.address,
            &self.address.repo,
            &Arguments::default(),
        )
        .ok()?;
        if out.refused() {
            return None;
        }
        let doc: serde_yaml::Value = serde_yaml::from_str(&out.stdout).ok()?;
        doc.get("corpus")
            .and_then(|v| v.as_str())
            .filter(|v| is_identity(v))
            .map(str::to_string)
    }
}

/// The reader's declarations, or an empty map and the reason it is empty.
///
/// **Silent where `claim` is silent** (ADR-96174f1ac2b7, TASK-1317adb617e8).
/// A map that cannot be read declares nothing, and by the time this runs the
/// CLI's own resolution has already read the file once and refused on it —
/// `ank mcp` does not start on a `corpora.yml` that will not parse. What is kept
/// rather than dropped is the reason, so that the one caller who can still meet
/// a broken map, the one who started the server with `--repo`, is told why the
/// identity they named is not there instead of being told nobody declared it.
fn read_declarations() -> Declared {
    let Some(path) = corpora_path() else {
        return Declared::empty();
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        // The ordinary case and not a failure: a reader who has declared
        // nothing has no file, and gets the corpus the server was addressed
        // with.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Declared::empty(),
        Err(e) => return Declared::broken(format!("{}: {e}", corpora_file())),
    };
    parse_declarations(&text)
}

/// The file's own content as a map, split from the reading so a test can hand in
/// bytes instead of a home directory.
fn parse_declarations(text: &str) -> Declared {
    let doc: serde_yaml::Value = match serde_yaml::from_str(text) {
        Ok(doc) => doc,
        Err(e) => return Declared::broken(format!("{}: {e}", corpora_file())),
    };
    if doc.get("schema").and_then(|v| v.as_u64()) != Some(SUPPORTED_SCHEMA) {
        return Declared::broken(format!(
            "{}: schema is not {SUPPORTED_SCHEMA}",
            corpora_file()
        ));
    }
    let mut map = BTreeMap::new();
    if let Some(entries) = doc.get("corpora").and_then(|v| v.as_mapping()) {
        for (key, value) in entries {
            let (Some(key), Some(value)) = (key.as_str(), value.as_str()) else {
                continue;
            };
            // A key that is not an identity is a key no caller can name, since
            // what a caller passes is checked against the same rule before it
            // is looked up. The CLI refuses such a file outright and this
            // surface never starts on one; dropping the row here is what keeps
            // the two from disagreeing about a file only one of them refused.
            if is_identity(key) {
                map.insert(key.to_string(), value.to_string());
            }
        }
    }
    Declared {
        map,
        unreadable: None,
    }
}

/// A `corpus` that is not an identity at all.
///
/// The hint names the command that prints one, because the mistake this catches
/// is a caller reaching for something they can type — a path, a remote, a
/// directory name — where the value is a root commit.
fn not_an_identity(named: &str) -> Refusal {
    Refusal {
        code: ExitCode::Environment,
        message: format!("'{}' is not a repository identity", rendered(named)),
        hint: "a corpus is named by its root commit, never a path, a remote or a slug: \
               ank status --json prints it under \"corpus\""
            .to_string(),
    }
}

/// An identity nobody declared.
///
/// **Refused by name**, and this is the sentence that says the surface discovers
/// nothing: there is no directory it could have looked in, so the only thing it
/// can report is that this reader has not said where that corpus is. The hint is
/// the verb that says it (ADR-96174f1ac2b7).
fn undeclared(named: &str, unreadable: Option<&str>) -> Refusal {
    let message = match unreadable {
        None => format!(
            "no corpus is declared under {named}, and this server reaches no corpus \
             nobody declared"
        ),
        Some(why) => format!("no corpus is declared under {named}: {why}"),
    };
    Refusal {
        code: ExitCode::Environment,
        message,
        hint: format!("ank config --user corpora.{named} <path>"),
    }
}

/// A declaration pointing at a directory that is not there.
///
/// The CLI's own sentence for the same fact, with the CLI's own code and hint
/// (`crates/ank-cli/src/repo.rs`). A stale entry reads the same through either
/// surface, which is what a reader repairing it needs and what keeps this from
/// being a second account of one failure.
fn not_there(named: &str, declared: &str) -> Refusal {
    Refusal {
        code: ExitCode::Generic,
        message: format!(
            "the corpus declared for {named} is not at {}",
            rendered(declared)
        ),
        hint: format!(
            "ank init --at {}, or correct the entry in {}",
            rendered(declared),
            corpora_file()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID: &str = "0123456789abcdef0123456789abcdef01234567";

    /// The rule ADR-621a7fd96ce1 states, applied to what a caller passes.
    ///
    /// The three rejected values are the three ADR-96174f1ac2b7 names as the
    /// mistake: a path, a remote URL and a slug.
    #[test]
    fn an_identity_is_a_root_commit_and_never_a_path_a_remote_or_a_slug() {
        assert!(is_identity(ID));
        for wrong in [
            "",
            "/home/someone/corpora/front",
            "git@github.com:someone/front.git",
            "front",
            "0123456789ABCDEF0123456789ABCDEF01234567",
            "0123456789abcdef0123456789abcdef0123456",
            "0123456789abcdef0123456789abcdef012345678",
        ] {
            assert!(!is_identity(wrong), "{wrong:?} is not an identity");
        }
    }

    /// The map is the file's, and a row that is not keyed on an identity is not
    /// a row a caller could ever name.
    #[test]
    fn the_map_is_read_from_the_file_and_keyed_on_identities() {
        let declared = parse_declarations(&format!(
            "schema: 1\ncorpora:\n  {ID}: /somewhere/front\n  front: /somewhere/slug\n"
        ));
        assert_eq!(declared.unreadable, None);
        assert_eq!(
            declared.map.get(ID).map(String::as_str),
            Some("/somewhere/front")
        );
        assert_eq!(
            declared.map.len(),
            1,
            "a slug is not a key: {:?}",
            declared.map
        );
    }

    /// A file that will not parse declares nothing and says why, so the refusal
    /// a caller meets names the file rather than accusing them of naming a
    /// corpus that does not exist.
    #[test]
    fn a_file_that_will_not_parse_declares_nothing_and_keeps_the_reason() {
        for text in ["", "schema: 2\ncorpora: {}\n", "schema: 1\ncorpora: [\n"] {
            let declared = parse_declarations(text);
            assert!(declared.map.is_empty(), "{text:?} declared something");
            assert!(
                declared.unreadable.is_some(),
                "{text:?} left no reason to report"
            );
        }
    }

    /// A refusal carries a §4 code, names the corpus, and names the command that
    /// resolves it — the three things ADR-fd98f4bc6dea and §4 require of one.
    #[test]
    fn a_refusal_names_the_corpus_the_code_and_the_command() {
        let refusal = undeclared(ID, None);
        assert_eq!(refusal.code, ExitCode::Environment);
        let outcome = refusal.outcome();
        assert!(outcome.refused(), "a refusal is not a success");
        assert_eq!(outcome.code, ExitCode::Environment.code());
        assert!(
            outcome.stderr.contains(ID),
            "refused by name: {}",
            outcome.stderr
        );
        assert!(
            outcome
                .stderr
                .contains(&format!("error[{}]:", ExitCode::Environment)),
            "the code is rendered as §4 renders it: {}",
            outcome.stderr
        );
        assert!(
            outcome
                .stderr
                .contains(&format!("ank config --user corpora.{ID}")),
            "the command that resolves it travels with it: {}",
            outcome.stderr
        );
        let result = outcome.to_result();
        assert!(
            result.contains("\"isError\":true") && result.contains("\"exitCode\":9"),
            "a refusal reaches a client in the shape every other refusal has: {result}"
        );
    }

    /// The reason a broken map is kept: two refusals a caller must be able to
    /// tell apart.
    #[test]
    fn a_broken_map_refuses_differently_from_an_undeclared_identity() {
        let plain = undeclared(ID, None).message;
        let broken = undeclared(ID, Some("corpora.yml: schema is not 1")).message;
        assert_ne!(plain, broken);
        assert!(broken.contains("schema is not 1"), "{broken}");
    }

    /// A value that is not an identity is refused before anything is looked up,
    /// which is what stops a path from reaching `--repo` through the back door.
    #[test]
    fn a_path_in_the_argument_is_refused_and_never_resolved() {
        let refusal = not_an_identity("/home/someone/other-corpus");
        assert_eq!(refusal.code, ExitCode::Environment);
        assert!(refusal.message.contains("/home/someone/other-corpus"));
        assert!(refusal.hint.contains("ank status --json"));
    }
}
