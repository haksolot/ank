//! Following the watcher's change stream (TASK-2f7777a1fdff, ADR-a22cd3196529).
//!
//! The corpus is a working tree and it moves under a screen left open. Until
//! this module the only answer was `r`, and the only alternative anybody ever
//! reaches for is a refresh loop -- which this crate must not have, because the
//! verbs that answer best are verbs that renew a lease (ADR-0bb7ea8991bc) and a
//! screen nobody is sitting at would then keep a claim alive for somebody who
//! went home. So the reader is *told*: `ank-daemon` appends a line when a corpus
//! it watches changes, and this follows that file.
//!
//! **Following is not asking.** Nothing here talks to the watcher, and there is
//! nothing to talk to: the stream is a file, the watcher writes it, and several
//! readers follow the same bytes without it knowing any of them exist. That is
//! what keeps ADR-a22cd3196529's "answers no verb" true with a consumer
//! attached, and it is why the watcher's absence costs a repaint that has to be
//! typed rather than an error.
//!
//! **An event is a repaint and never a read of the entity.** What arrives says
//! which corpus moved and what kind of change it was, and nothing else. What is
//! now true of a task is what the CLI answers, and asking it is
//! [`crate::view::App::repaint`] -- which runs the verbs that read and never the
//! one that writes. See the note there: it is the whole of why an event cannot
//! renew anything.
//!
//! **The file is not the corpus.** This is the reader's own configuration
//! directory, not `.ank/`, and nothing here reads an entity, a ref or a byte of
//! a repository. ADR-8bd76e8d7c4e's rule is untouched: the corpus is reached by
//! running the CLI with `--json` and by nothing else.

use crate::Wake;
use ank_contract::events;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;

/// How often the stream is looked at.
///
/// A poll of one file in the reader's own configuration directory, and never of
/// the corpus: no verb runs on this clock, nothing is spawned, no ref is read
/// and nothing is renewed. It is the same call `ank-daemon` makes about the
/// files it watches, for the same reason §13 gives -- a subscription is a
/// third-party crate, and what it would buy over a stat is latency a person
/// cannot perceive.
///
/// A quarter of a second is under the time a person takes to notice, and four
/// stats a second on one small file is nothing to measure.
const TICK: Duration = Duration::from_millis(250);

/// Whether there is a stream to follow, as the follower last saw it.
///
/// A handle and not a value, because the answer moves: a watcher started after
/// the session opened makes one appear, and a person who wiped their
/// configuration directory makes one go away. The frame reads it at every paint,
/// so what the screen says is what was true when it was drawn.
///
/// It says a stream exists, never that a watcher is running. Nothing here can
/// honestly say the second without polling something, and polling something is
/// the thing this module exists to remove.
#[derive(Clone)]
pub struct Stream {
    live: Arc<AtomicBool>,
}

impl Stream {
    pub fn following(&self) -> bool {
        self.live.load(Ordering::Relaxed)
    }

    /// A stream for a test to state, without a file or a thread.
    #[cfg(test)]
    pub fn stated(following: bool) -> Stream {
        Stream {
            live: Arc::new(AtomicBool::new(following)),
        }
    }
}

/// Starts following the stream on this reader's behalf, or answers `None` where
/// there is nothing to follow.
///
/// `None` is not a fault and is never reported as one: a reader whose
/// environment names no home, or which could not learn its own corpus identity,
/// falls back to what it always did, which is to read when its person asks. The
/// watcher is optional by construction and so is this.
///
/// **The starting offset is the file's length, read here and not in the
/// thread.** Events from before this session opened are old news -- the screen
/// was drawn from the corpus as it is now -- so replaying them would cost a
/// repaint that says nothing. A file that appears *later* is read from its
/// beginning, because then every line in it is news.
pub fn follow(corpus: &str, wake: Sender<Wake>) -> Option<Stream> {
    if corpus.is_empty() {
        return None;
    }
    let path = events::stream_path()?;
    let offset = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    let live = Arc::new(AtomicBool::new(path.is_file()));
    let mine = corpus.to_string();
    let flag = Arc::clone(&live);
    std::thread::spawn(move || {
        let mut follower = Follower {
            path,
            offset,
            carry: String::new(),
            corpus: mine,
        };
        loop {
            let (exists, changed) = follower.look();
            flag.store(exists, Ordering::Relaxed);
            // One wake per look, however many lines arrived: the reader answers
            // an event by reading the corpus once, so ten events in a tick are
            // one repaint and not ten.
            if changed && wake.send(Wake::Changed).is_err() {
                return;
            }
            std::thread::sleep(TICK);
        }
    });
    Some(Stream { live })
}

/// The follower's own state: where it had read to, and the tail of a line it has
/// not yet seen the end of.
struct Follower {
    path: PathBuf,
    offset: u64,
    carry: String,
    corpus: String,
}

impl Follower {
    /// One look: whether the stream is there, and whether it said this corpus
    /// moved.
    fn look(&mut self) -> (bool, bool) {
        let Ok(meta) = std::fs::metadata(&self.path) else {
            return (false, false);
        };
        let len = meta.len();
        if len < self.offset {
            // The watcher started the file over (ADR-a22cd3196529's stream is
            // news and is bounded, not kept). Reading from the beginning again
            // costs at most one repaint of a corpus that has certainly changed.
            self.offset = 0;
            self.carry.clear();
        }
        if len == self.offset {
            return (true, false);
        }
        let Ok(mut file) = std::fs::File::open(&self.path) else {
            return (true, false);
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return (true, false);
        }
        let mut fresh = String::new();
        // Lossy, and deliberately: a line this reader cannot decode is a line it
        // skips, and a stream that stopped being followed because one byte was
        // wrong would be worse than one that missed an event.
        let mut raw = Vec::new();
        if file.read_to_end(&mut raw).is_err() {
            return (true, false);
        }
        self.offset += raw.len() as u64;
        fresh.push_str(&String::from_utf8_lossy(&raw));
        self.carry.push_str(&fresh);
        // Whole lines only. The watcher writes one line per call, but a follower
        // that consumed a half-written one would repaint on a corpus it could
        // not name.
        let mut changed = false;
        while let Some(at) = self.carry.find('\n') {
            let line: String = self.carry.drain(..=at).collect();
            if self.mine(line.trim_end()) {
                changed = true;
            }
        }
        (true, changed)
    }

    /// Whether one line is an event about the corpus this reader is on.
    ///
    /// Read with [`crate::ank::document`], which is how this crate reads every
    /// document the CLI answers with, and by the key names [`events`] declares
    /// rather than by strings written here. Asked of that function rather than
    /// of a parser named here (TASK-f0c6372d8dc0): a watcher's line and a
    /// verb's answer are the same language, and one call is what keeps them
    /// from becoming two readings of it.
    ///
    /// **A line of a schema this build does not read is skipped**, on the rule
    /// `watch.yml` already holds: a reader that guessed at a shape it does not
    /// know would be repainting on something nobody wrote. A line that does not
    /// parse is skipped for the same reason and neither is an error, because
    /// there is nobody to report it to and nothing depends on it.
    fn mine(&self, line: &str) -> bool {
        let Ok(value) = crate::ank::document(line) else {
            return false;
        };
        let schema = value
            .get(events::SCHEMA_KEY)
            .and_then(|v| v.as_u64())
            .unwrap_or_default();
        if schema != events::SCHEMA as u64 {
            return false;
        }
        value
            .get(events::CORPUS_KEY)
            .and_then(|v| v.as_str())
            .is_some_and(|c| c == self.corpus)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ank_contract::events::{Change, Event};

    /// One root per test process, swept by the next run (TASK-553740e7af11).
    ///
    /// **A copy of what `crates/ank-cli/tests/scratch/mod.rs` does**, for the
    /// reason `crates/ank-tui/tests/terminal/mod.rs` gives for the copy it
    /// carries: two crates share a test helper only through a dependency
    /// between them, and none of these three is worth adding for thirty lines.
    /// Nothing under `src/` can name a module of an integration binary in any
    /// case, so the copy lands here rather than there (TASK-ac2ff41162c6).
    ///
    /// The reasoning is written out once, where the original lives, and the
    /// property is tested there. In short: a `Drop` cannot run on `SIGKILL`, so
    /// the run that cleans up is the next one, and a root's `.owner` lock is
    /// free exactly when its owner is gone. `PREFIX` is deliberately the one
    /// every other suite in this workspace already uses: one name, so any run
    /// collects what any other left, and "what does the workspace leave in the
    /// temporary directory" keeps one answer.
    mod scratch {
        use std::fs::{self, File};
        use std::path::{Path, PathBuf};
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::sync::OnceLock;

        const PREFIX: &str = "ank-it-";
        const OWNER: &str = ".owner";
        static HELD: OnceLock<File> = OnceLock::new();

        pub fn dir(what: &str) -> PathBuf {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = root().join(format!("{what}-{}", SEQ.fetch_add(1, Ordering::Relaxed)));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).expect("the root must be writable");
            p
        }

        fn root() -> &'static Path {
            static ROOT: OnceLock<PathBuf> = OnceLock::new();
            ROOT.get_or_init(|| {
                let base = std::env::temp_dir();
                let mine = base.join(format!("{PREFIX}{}", std::process::id()));
                let _ = fs::remove_dir_all(&mine);
                fs::create_dir_all(&mine).expect("the temporary directory must be writable");
                let lock = File::create(mine.join(OWNER)).expect("the root takes its own lock");
                lock.try_lock()
                    .expect("a fresh root cannot already be held");
                let _ = HELD.set(lock);
                sweep(&base, &mine);
                mine
            })
            .as_path()
        }

        fn sweep(base: &Path, mine: &Path) {
            let Ok(entries) = fs::read_dir(base) else {
                return;
            };
            for entry in entries.flatten() {
                let p = entry.path();
                let named = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(PREFIX));
                if p == mine || !named || !p.is_dir() {
                    continue;
                }
                let Ok(file) = File::options().read(true).write(true).open(p.join(OWNER)) else {
                    continue;
                };
                if file.try_lock().is_ok() {
                    let _ = file.unlock();
                    drop(file);
                    let _ = fs::remove_dir_all(&p);
                }
            }
        }
    }

    /// The stream file inside a fresh directory of this process's root.
    fn stream_at(what: &str) -> PathBuf {
        scratch::dir(&format!("tui-stream-{what}")).join(events::STREAM_FILE)
    }

    fn follower(path: PathBuf, corpus: &str) -> Follower {
        Follower {
            offset: std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
            path,
            carry: String::new(),
            corpus: corpus.to_string(),
        }
    }

    fn append(path: &PathBuf, line: &str) {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        f.write_all(line.as_bytes()).unwrap();
    }

    #[test]
    fn a_line_about_another_corpus_is_not_this_readers_news() {
        let path = stream_at("other");
        append(&path, &Event::new("a".repeat(40), Change::Entities).line());
        let mut f = follower(path.clone(), &"z".repeat(40));
        assert_eq!(f.look(), (true, false), "the backlog is skipped anyway");
        append(&path, &Event::new("a".repeat(40), Change::Entities).line());
        assert_eq!(f.look(), (true, false), "another corpus is not mine");
        append(&path, &Event::new("z".repeat(40), Change::Refs).line());
        assert_eq!(f.look(), (true, true));
    }

    /// Ten events in one tick are one repaint: the reader answers an event by
    /// reading the corpus, and reading it ten times would say the same thing
    /// ten times.
    #[test]
    fn many_lines_in_one_look_are_one_wake() {
        let path = stream_at("coalesce");
        let mut f = follower(path.clone(), &"c".repeat(40));
        for _ in 0..10 {
            append(&path, &Event::new("c".repeat(40), Change::Entities).line());
        }
        assert_eq!(f.look(), (true, true));
        assert_eq!(f.look(), (true, false), "and nothing is replayed");
    }

    #[test]
    fn a_half_written_line_waits_for_its_end() {
        let path = stream_at("partial");
        let mut f = follower(path.clone(), &"d".repeat(40));
        let whole = Event::new("d".repeat(40), Change::Entities).line();
        let (head, tail) = whole.split_at(whole.len() / 2);
        append(&path, head);
        assert_eq!(f.look(), (true, false), "half a line is not an event");
        append(&path, tail);
        assert_eq!(f.look(), (true, true));
    }

    #[test]
    fn a_stream_started_over_is_read_from_the_beginning_again() {
        let path = stream_at("rotate");
        let mut f = follower(path.clone(), &"e".repeat(40));
        append(&path, &Event::new("e".repeat(40), Change::Entities).line());
        assert_eq!(f.look(), (true, true));
        // The watcher truncated and wrote one line, so the file is shorter than
        // where this follower had read to.
        std::fs::write(&path, Event::new("e".repeat(40), Change::Refs).line()).unwrap();
        assert_eq!(f.look(), (true, true));
    }

    #[test]
    fn a_stream_that_is_not_there_is_an_absence_and_never_an_error() {
        let mut f = follower(stream_at("absent"), &"f".repeat(40));
        assert_eq!(f.look(), (false, false));
        append(
            &f.path.clone(),
            &Event::new("f".repeat(40), Change::Refs).line(),
        );
        assert_eq!(
            f.look(),
            (true, true),
            "a watcher started later is followed"
        );
    }

    /// A line of a schema this build does not read is skipped rather than
    /// guessed at, and a line that is not a document at all is skipped too.
    #[test]
    fn an_unreadable_line_is_skipped_and_stops_nothing() {
        let path = stream_at("junk");
        let mut f = follower(path.clone(), &"g".repeat(40));
        append(&path, "not a document at all\n");
        append(
            &path,
            &format!(
                "{{\"{}\":99,\"{}\":\"{}\"}}\n",
                events::SCHEMA_KEY,
                events::CORPUS_KEY,
                "g".repeat(40)
            ),
        );
        assert_eq!(f.look(), (true, false));
        append(&path, &Event::new("g".repeat(40), Change::Entities).line());
        assert_eq!(f.look(), (true, true), "and the next good line still lands");
    }

    #[test]
    fn a_reader_with_no_corpus_identity_follows_nothing() {
        let (tx, _rx) = std::sync::mpsc::channel();
        assert!(follow("", tx).is_none());
    }
}
