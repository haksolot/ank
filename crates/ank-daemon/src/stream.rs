//! Putting a change onto the stream (ADR-24e21cb83793, TASK-2f7777a1fdff).
//!
//! The shape of an event, where it goes and what it may carry are
//! [`ank_contract::events`], because a stream has two ends and they must not
//! drift. What is here is the writing: opening the file, appending one line, and
//! starting it over when it has grown past what news is worth.
//!
//! **Writing is best-effort, in the way everything this process does is.**
//! Nothing depends on the watcher (ADR-24e21cb83793), so a stream that cannot be
//! written is a stream nobody gets, exactly as if no watcher were running. A
//! full disk, a configuration directory somebody removed, a permission somebody
//! changed: each costs a line on stderr and the next poll, and none of them
//! reaches the exit code of anything the person is running.
//!
//! **An event is not a log entry**, and the difference is why this file is
//! allowed to be thrown away. An entity's log is a work trace with a grammar,
//! kept as entities of its own (ADR-67a4ac10c534); this is news, nothing is
//! anchored in it and no hash chains over it, so a reader that was not running
//! missed what happened while it was not running, and the file is bounded
//! rather than kept.

use ank_contract::events::{self, Change, Event};
use std::io::Write;
use std::path::{Path, PathBuf};

/// The stream this watcher writes into, or `None` where the environment names
/// no home.
///
/// Resolved once at start rather than per event: the directory a reader
/// declared their watch in is the directory their stream belongs in, and asking
/// the environment again every half-second would let the two answer differently.
pub fn path() -> Option<PathBuf> {
    events::stream_path()
}

/// Appends one event, or says why it could not.
///
/// The directory is created rather than required, for the one case that
/// actually occurs: a reader whose declaration lives somewhere `ank watch`
/// reached through `XDG_CONFIG_HOME` has that directory, and a reader who has
/// somehow lost it between two polls is better served by a stream that comes
/// back than by a watcher that stops writing one.
pub fn emit(path: &Path, identity: &str, change: Change) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    start_over_if_long(path)?;
    let line = Event::new(identity, change).line();
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("{}: {e}", path.display()))?;
    // One `write_all` per event, and the line carries its own terminator, so a
    // follower reading between two events reads whole lines or nothing. A
    // follower that read a half-written line would repaint on a corpus it could
    // not name, which is the one way this stream could mislead.
    file.write_all(line.as_bytes())
        .map_err(|e| format!("{}: {e}", path.display()))
}

/// Truncates the stream once it has grown past [`events::CAP`].
///
/// Started over rather than rotated into a second file: there is no second
/// reader of the old bytes, nothing refers back to them, and a rotation would
/// leave a directory of files somebody has to clean up. A follower notices
/// because the file is suddenly shorter than the offset it holds, which
/// ADR-24e21cb83793's reader handles by reading from the beginning again.
fn start_over_if_long(path: &Path) -> Result<(), String> {
    let long = std::fs::metadata(path).map(|m| m.len() >= events::CAP);
    if !matches!(long, Ok(true)) {
        return Ok(());
    }
    std::fs::File::create(path)
        .map(|_| ())
        .map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn one_event_is_one_line_and_the_next_goes_after_it() {
        let path = scratch::dir("stream-append").join(events::STREAM_FILE);
        emit(&path, &"a".repeat(40), Change::Entities).unwrap();
        emit(&path, &"b".repeat(40), Change::Refs).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 2, "{text}");
        assert!(lines[0].contains("\"entities\""), "{text}");
        assert!(lines[1].contains("\"refs\""), "{text}");
    }

    #[test]
    fn a_stream_past_the_cap_is_started_over_rather_than_kept() {
        let path = scratch::dir("stream-cap").join(events::STREAM_FILE);
        std::fs::write(&path, vec![b'x'; events::CAP as usize + 1]).unwrap();
        emit(&path, &"c".repeat(40), Change::Entities).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text.lines().count(), 1, "the old bytes are gone");
        assert!(text.contains("\"entities\""), "{text}");
    }

    /// The directory a reader lost between two polls comes back, because the
    /// alternative is a watcher that quietly stops reporting.
    #[test]
    fn a_missing_directory_is_made_rather_than_refused() {
        let path = scratch::dir("stream-mkdir")
            .join("gone")
            .join(events::STREAM_FILE);
        emit(&path, &"d".repeat(40), Change::Refs).unwrap();
        assert!(path.is_file());
    }
}
