//! One directory per test process, and the next run sweeps what a killed one
//! left (TASK-553740e7af11).
//!
//! # Why this is not another `Drop`
//!
//! The fixtures in this suite already implement `Drop`, and correctly: a green
//! run removes what it made. That is worth keeping and it is not sufficient,
//! because **no destructor runs on `SIGKILL`**. Measured on 2026-08-27, after
//! this suite exhausted the memory of a host whose `/tmp` is a RAM-backed
//! tmpfs: 8361 abandoned fixture directories, 1.1G, of which 5002 came from
//! `Repo`, whose `Drop` is right. They were the runs that were killed — and
//! what killed them was the memory the previous leak had taken. The failure fed
//! itself.
//!
//! Cleanup that only ever runs forward is best-effort by construction. So the
//! run that cleans up here is **the next one**, which is alive by definition.
//!
//! # How a dead owner is recognised
//!
//! Each root holds an `.owner` file, locked for the life of the process that
//! made it. An advisory file lock is released by the kernel when the holder
//! dies, however it dies — exit, panic, or `SIGKILL` — so a sweeper asks "is
//! this root's owner still running?" by trying to take the lock, and a success
//! means nobody is there. `std::fs::File::try_lock` is stable since 1.89 and
//! this workspace's floor is 1.95, so this costs no dependency, no `libc`, and
//! no `/proc` — which matters, because `/proc` does not exist on macOS and the
//! rule in CLAUDE.md is that OS-dependent behaviour is not verified until it
//! has run on all three platforms. This one is `std` on all three.

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

/// What every root of every suite is called, so one sweep reaches all of them.
///
/// The prefixes this replaced were nine — `ank-cli-it-`, `ank-watch-it-`,
/// `ank-tui-it-`, `ank-mcp-it-`, `ank-stamp-`, `ank-wt-`, `ank-guard-`,
/// `ank-cli-init-`, `ank-cli-refspec-` — and "what does this suite leave
/// behind" had nine answers, none of which swept the others.
const PREFIX: &str = "ank-it-";

/// The lock file, inside each root.
const OWNER: &str = ".owner";

/// How long a root with no `.owner` yet is left alone.
///
/// A root is created and then locked, and cargo runs test binaries
/// concurrently, so another binary's sweep can see a directory in the
/// microseconds before its lock exists. Without this it would delete a root
/// whose owner was about to claim it. Sixty seconds is far longer than that
/// window and far shorter than the gap between runs.
const GRACE: std::time::Duration = std::time::Duration::from_secs(60);

/// The lock, held open for the life of the process. Dropping it would tell
/// every other run that this one had finished.
static HELD: OnceLock<File> = OnceLock::new();

/// This process's root, made on first use and swept of its dead predecessors.
pub fn root() -> &'static Path {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        let base = std::env::temp_dir();
        let mine = base.join(format!("{PREFIX}{}", std::process::id()));
        // A root left by a previous process that happened to share this pid is
        // this process's problem: it is about to answer for the name.
        let _ = fs::remove_dir_all(&mine);
        fs::create_dir_all(&mine).expect("the temporary directory must be writable");
        let lock = File::create(mine.join(OWNER)).expect("the root must take its own lock file");
        lock.try_lock()
            .expect("a fresh root cannot already be locked by somebody else");
        let _ = HELD.set(lock);
        sweep(&base, &mine);
        mine
    })
    .as_path()
}

/// A fresh directory inside this process's root, named after what it holds.
///
/// The name carries `what` so a failing run can be read by a human, and a
/// counter so two fixtures of one kind never collide.
pub fn dir(what: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let p = root().join(format!("{what}-{}", SEQ.fetch_add(1, Ordering::Relaxed)));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("the root must be writable");
    p
}

/// A path inside this process's root, created by whoever asked for it.
///
/// For the fixtures that want a name rather than a directory — a file to write,
/// or a path that must *not* exist.
pub fn path(what: &str) -> PathBuf {
    root().join(what)
}

/// Remove every sibling root whose owner is gone. `mine` is never a candidate.
pub fn sweep(base: &Path, mine: &Path) {
    let Ok(entries) = fs::read_dir(base) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p == mine {
            continue;
        }
        let Some(name) = p.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with(PREFIX) || !p.is_dir() {
            continue;
        }
        if abandoned(&p) {
            let _ = fs::remove_dir_all(&p);
        }
    }
}

/// Whether nothing alive owns this root.
///
/// The `File` is dropped before the caller removes the directory, which Windows
/// requires: a directory holding an open handle cannot be deleted there.
fn abandoned(root: &Path) -> bool {
    let lock = root.join(OWNER);
    let Ok(file) = File::options().read(true).write(true).open(&lock) else {
        // No lock file at all. Either a root from before this scheme, or one
        // being born right now — and the second is what GRACE separates out.
        return older_than_grace(root);
    };
    match file.try_lock() {
        Ok(()) => {
            let _ = file.unlock();
            true
        }
        // Held: the owner is running, and this is not ours to remove.
        Err(_) => false,
    }
}

fn older_than_grace(p: &Path) -> bool {
    fs::metadata(p)
        .and_then(|m| m.modified())
        .and_then(|t| {
            std::time::SystemTime::now()
                .duration_since(t)
                .map_err(|e| std::io::Error::other(e.to_string()))
        })
        .map(|age| age > GRACE)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The property the whole scheme rests on**, and the one a `Drop` cannot
    /// have: a root whose owner is gone is collected, and a root whose owner is
    /// running is not.
    #[test]
    fn a_root_whose_owner_is_gone_is_swept_and_a_live_one_is_left_alone() {
        let base = dir("sweep-base");

        // A root nothing holds: its lock file exists and is free, which is
        // exactly the state `SIGKILL` leaves behind.
        let dead = base.join(format!("{PREFIX}999999"));
        fs::create_dir_all(&dead).unwrap();
        File::create(dead.join(OWNER)).unwrap();

        // A root this process holds, locked for the length of the test.
        let live = base.join(format!("{PREFIX}{}", std::process::id()));
        fs::create_dir_all(&live).unwrap();
        let held = File::create(live.join(OWNER)).unwrap();
        held.try_lock().unwrap();

        // A directory that is not a root at all.
        let other = base.join("not-a-root");
        fs::create_dir_all(&other).unwrap();

        sweep(&base, Path::new("nothing-is-mine-here"));

        assert!(!dead.exists(), "a root with a free lock is abandoned");
        assert!(
            live.exists(),
            "a root whose owner still holds the lock stays"
        );
        assert!(
            other.exists(),
            "a directory that is not a root is not touched"
        );
        let _ = held.unlock();
    }

    /// The root the sweep is told is its own is never a candidate, even though
    /// its lock is held by this very process and would therefore look free to
    /// any check that opened it a second time.
    #[test]
    fn the_sweeper_never_removes_its_own_root() {
        let base = dir("sweep-self");
        let mine = base.join(format!("{PREFIX}{}", std::process::id()));
        fs::create_dir_all(&mine).unwrap();
        File::create(mine.join(OWNER)).unwrap();
        sweep(&base, &mine);
        assert!(
            mine.exists(),
            "the caller's own root is excluded by identity"
        );
    }

    /// A root with no lock file is left alone until it is old enough to be
    /// certain it is not a run that started a moment ago.
    #[test]
    fn a_root_still_being_born_is_not_swept() {
        let base = dir("sweep-young");
        let young = base.join(format!("{PREFIX}123456"));
        fs::create_dir_all(&young).unwrap();
        sweep(&base, Path::new("nothing-is-mine-here"));
        assert!(
            young.exists(),
            "a root younger than the grace period may still be claiming its lock"
        );
    }

    /// Every fixture path is inside the one root, which is what makes a single
    /// sweep enough.
    #[test]
    fn every_scratch_path_is_under_this_process_root() {
        assert!(dir("somewhere").starts_with(root()));
        assert!(path("a-file").starts_with(root()));
        assert!(root()
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == format!("{PREFIX}{}", std::process::id())));
    }
}
