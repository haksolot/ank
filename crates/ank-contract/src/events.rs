//! The change stream: what the watcher says happened, and where it says it
//! (ADR-a22cd3196529, TASK-2f7777a1fdff).
//!
//! `ank-daemon` already knows when a corpus it watches moves. Until this module
//! that knowledge reached a person as a line on stderr and reached a program not
//! at all, so a reader that wanted to be current had to ask again on a timer --
//! which is the one thing a long-lived screen must not do, because the verbs
//! that answer best are the verbs that renew a lease (ADR-0bb7ea8991bc).
//!
//! **An event says what changed, and never what to do about it.** That is the
//! line ADR-a22cd3196529 draws to keep the watcher from becoming a third
//! dispatch path, and it is easy to cross by kindness: an event carrying the new
//! state of a task would save its reader a call, and would make the watcher a
//! source of entity content that nothing generated from `COMMANDS` ever
//! validated. So [`Event`] carries a repository identity and a word, there is no
//! constructor that takes anything else, and the test at the foot of this file
//! is what keeps it that way.
//!
//! **A file, and deliberately not a socket.** ADR-a22cd3196529 says the watcher
//! answers no verb and exposes no query surface of its own. A socket is a place
//! a caller connects *to*, and the first convenient question asked over one is
//! the beginning of the surface that decision refused. A file is a place the
//! watcher writes: nothing is asked of it, nothing is negotiated, several
//! readers follow the same bytes without it knowing they exist, and a reader
//! written outside this repository needs a file reader and nothing else. It also
//! costs no platform code, which a socket would on Windows.
//!
//! **This module is shared because a stream has two ends.** `ank-daemon` writes
//! it and `ank-tui` follows it; the key names, the schema number, the vocabulary
//! of changes and the escaper have to be one thing or they are two things that
//! will disagree, and the disagreement would land on whoever wrote a third
//! reader. That is the reason this crate exists, applied to a surface that is
//! not a verb.

use crate::json::Obj;
use std::path::PathBuf;

/// The stream, under [`user_dir`], beside the `watch.yml` that declares what is
/// watched.
///
/// One file for every corpus the watcher was handed, rather than one per
/// corpus: a reader following several corpora then follows one file, and a
/// reader following one skips the lines that are not its own. The line says
/// which corpus it is about, so nothing is inferred from a path.
pub const STREAM_FILE: &str = "events.jsonl";

/// The shape of a line, versioned the way `watch.yml` and `.ank/config.yml` are.
///
/// **Not [`crate::CONTRACT_VERSION`].** That number describes the documents
/// verbs answer with, and this stream is not a verb answering: the two move for
/// different reasons and a single number would tie a change in one to a release
/// of the other. Within this one, a line may *gain* a field and may never lose,
/// rename or retype one -- the same promise, made separately.
pub const SCHEMA: u32 = 1;

/// The key carrying [`SCHEMA`].
pub const SCHEMA_KEY: &str = "schema";

/// The key carrying the repository identity of ADR-621a7fd96ce1.
pub const CORPUS_KEY: &str = "corpus";

/// The key carrying the word from [`Change`].
pub const CHANGE_KEY: &str = "change";

/// How large the stream is allowed to grow before the watcher starts it over.
///
/// A stream is news and not a log: nothing is anchored in it, nothing hashes
/// over it, and a reader that was not running missed what happened while it was
/// not running whatever this number is. So it is bounded rather than rotated
/// into a second file, and a follower that finds the file shorter than the
/// offset it holds reads it from the beginning again -- which costs it a repaint
/// of a corpus that has certainly changed.
pub const CAP: u64 = 64 * 1024;

/// What kind of change the watcher saw. Two, because it can see two.
///
/// **Neither says what it means.** `Entities` is "the files under `.ank/`
/// moved", not "a task was claimed"; `Refs` is "the mirror of `refs/ank/*`
/// moved", not "somebody else took your task". Which entity, and what is now
/// true of it, is what the CLI answers, and a reader that wants to know runs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// A file under the corpus directory was written, added or removed.
    Entities,
    /// The watcher's mirror of the remote's `refs/ank/*` moved.
    Refs,
}

impl Change {
    /// The word this change is written as, and the whole of what it says.
    pub fn word(self) -> &'static str {
        match self {
            Change::Entities => "entities",
            Change::Refs => "refs",
        }
    }

    /// The change a word names, or `None` for a word this build does not know.
    ///
    /// A reader that meets an unknown word has met a watcher newer than itself.
    /// The honest answer there is to repaint anyway -- something changed, and
    /// the CLI is what says what -- so this returns an absence rather than an
    /// error, and no caller is asked to fail on it.
    pub fn of(word: &str) -> Option<Change> {
        CHANGES.iter().copied().find(|c| c.word() == word)
    }
}

/// The whole vocabulary, so a reader can state it and a test can walk it.
pub const CHANGES: &[Change] = &[Change::Entities, Change::Refs];

/// One line of the stream.
///
/// Two fields, and there is no third to be tempted by. A constructor that took
/// an entity, a title or a status would be the kindness this decision refuses,
/// so there is none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    /// The repository identity of ADR-621a7fd96ce1: the root commit, never a
    /// path. Two worktrees of one repository are one corpus and therefore one
    /// value here, which is the whole reason the key is not the path -- and the
    /// reason no path is carried alongside it, since a field naming one is an
    /// invitation to key on it.
    pub corpus: String,
    pub change: Change,
}

impl Event {
    pub fn new(corpus: impl Into<String>, change: Change) -> Event {
        Event {
            corpus: corpus.into(),
            change,
        }
    }

    /// The event as it goes onto the stream: one JSON object, one line.
    ///
    /// Written through [`crate::json`] rather than with `format!`, so the one
    /// escaper this project has is the one escaper this stream has too.
    pub fn line(&self) -> String {
        format!(
            "{}\n",
            Obj::new()
                .num(SCHEMA_KEY, SCHEMA)
                .str(CORPUS_KEY, &self.corpus)
                .str(CHANGE_KEY, self.change.word())
                .finish()
        )
    }
}

/// Where this reader's own files live: the watch declaration, and the stream
/// the watcher writes into it.
///
/// `%APPDATA%\ank` on Windows; `$XDG_CONFIG_HOME/ank` elsewhere, falling back to
/// `$HOME/.config/ank`. An empty value counts as unset, because a shell that
/// exports a variable to nothing has said nothing, and joining onto it would
/// name a relative path under whatever directory the process happens to be in.
///
/// **The rule lives here because two crates now need it.** `ank-daemon` reads
/// its declaration from this directory and writes the stream into it; `ank-tui`
/// follows the stream out of it. A copy in each is a rule written twice, and
/// this crate is where this project puts the thing that must not be written
/// twice. `ank-cli` keeps its own for `ank config --user`, because it is a
/// binary with no library target, and `the_watch_file_sits_beside_the_corpora_file`
/// in the watcher's suite drives both binaries to hold that copy to this one.
pub fn user_dir() -> Option<PathBuf> {
    let var = |key: &str| {
        std::env::var_os(key)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(windows) {
        return var("APPDATA").map(|p| p.join("ank"));
    }
    if let Some(xdg) = var("XDG_CONFIG_HOME") {
        return Some(xdg.join("ank"));
    }
    var("HOME").map(|p| p.join(".config").join("ank"))
}

/// The stream itself, or `None` where the environment names no home.
///
/// An absent home is not an error on either end: the watcher refuses to start
/// because it has no declaration to read, and a reader with nowhere to follow
/// falls back to reading the corpus when its person asks it to.
pub fn stream_path() -> Option<PathBuf> {
    user_dir().map(|d| d.join(STREAM_FILE))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The property the criterion asks to be asserted rather than argued: no
    /// event carries entity content a reader would otherwise get from the CLI.
    ///
    /// Stated as a closed key set, because that is the form an addition fails
    /// against. A field carrying a title, a status or a body cannot arrive
    /// without this test naming it.
    #[test]
    fn a_line_carries_the_three_keys_and_no_other() {
        for change in CHANGES {
            let line = Event::new("a".repeat(40), *change).line();
            assert!(line.ends_with('\n'), "one event is one line: {line:?}");
            let body = line.trim_end();
            let keys: Vec<&str> = body
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split(',')
                .map(|pair| pair.split(':').next().unwrap_or_default().trim_matches('"'))
                .collect();
            assert_eq!(
                keys,
                [SCHEMA_KEY, CORPUS_KEY, CHANGE_KEY],
                "an event states which corpus and what kind of change, and nothing else: {body}"
            );
        }
    }

    #[test]
    fn the_schema_is_stated_and_is_not_the_verb_contract() {
        let line = Event::new("b".repeat(40), Change::Refs).line();
        assert!(
            line.contains(&format!("\"{SCHEMA_KEY}\":{SCHEMA}")),
            "{line}"
        );
        assert!(!line.contains("\"contract\""), "{line}");
    }

    /// A word and the change it names are one mapping, walked in both
    /// directions: a variant added to the enum and forgotten in `of` would
    /// answer `None` for its own word.
    #[test]
    fn every_change_is_read_back_from_the_word_it_is_written_as() {
        for change in CHANGES {
            assert_eq!(Change::of(change.word()), Some(*change));
        }
        assert_eq!(Change::of("claimed"), None, "a vocabulary of two, closed");
        assert_eq!(Change::of(""), None);
    }

    #[test]
    fn the_stream_sits_beside_the_declaration_it_belongs_to() {
        let Some(dir) = user_dir() else {
            // A machine with no home in the environment: the absence is the
            // answer, and it is not a fault.
            return;
        };
        assert_eq!(stream_path(), Some(dir.join(STREAM_FILE)));
    }
}
