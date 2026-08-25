//! Putting a change onto the stream (ADR-a22cd3196529, TASK-2f7777a1fdff).
//!
//! The shape of an event, where it goes and what it may carry are
//! [`ank_contract::events`], because a stream has two ends and they must not
//! drift. What is here is the writing: opening the file, appending one line, and
//! starting it over when it has grown past what news is worth.
//!
//! **Writing is best-effort, in the way everything this process does is.**
//! Nothing depends on the watcher (ADR-a22cd3196529), so a stream that cannot be
//! written is a stream nobody gets, exactly as if no watcher were running. A
//! full disk, a configuration directory somebody removed, a permission somebody
//! changed: each costs a line on stderr and the next poll, and none of them
//! reaches the exit code of anything the person is running.
//!
//! **An event is not a log entry**, and the difference is why this file is
//! allowed to be thrown away. `.ank/log/` is a work trace with a grammar and an
//! append-only rule (ADR-ff29); this is news, nothing is anchored in it and no
//! hash chains over it, so a reader that was not running missed what happened
//! while it was not running, and the file is bounded rather than kept.

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
/// ADR-a22cd3196529's reader handles by reading from the beginning again.
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

    fn scratch(what: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ank-daemon-stream-{what}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn one_event_is_one_line_and_the_next_goes_after_it() {
        let path = scratch("append").join(events::STREAM_FILE);
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
        let path = scratch("cap").join(events::STREAM_FILE);
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
        let path = scratch("mkdir").join("gone").join(events::STREAM_FILE);
        emit(&path, &"d".repeat(40), Change::Refs).unwrap();
        assert!(path.is_file());
    }
}
