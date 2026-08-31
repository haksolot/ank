//! The one thing this process writes into somebody else's repository
//! (ADR-24e21cb83793).
//!
//! **A tracking namespace, and never the corpus's own.** `init` wires
//! `+refs/ank/*:refs/ank/*`, so the fetch a person runs by hand lands the
//! remote's refs on top of this clone's. That is a person choosing to
//! synchronise. A background process doing the same thing would be rewriting
//! the coordination plane underneath whoever is working in the tree, and
//! `refs/ank/claims/<id>` is the ref a live claim of theirs sits on. So the
//! destination is [`TRACKING`]: a mirror the CLI reads and nobody claims
//! against, in a namespace no verb writes.
//!
//! **Narrow on purpose.** `refs/ank/*` is the namespace this project owns and
//! nobody rebases onto it. A full fetch would keep drift from the default
//! branch accurate too, which is tempting and is refused: a background process
//! that updates branches in a repository where somebody is working is a source
//! of surprises a coordination tool has no business being.
//!
//! **Everything git would otherwise do on the side is turned off.** `--no-tags`
//! because tag auto-following would bring tags in as a side effect of
//! downloading objects, and a tag that appeared while nobody asked is exactly
//! the surprise above. `--no-write-fetch-head` because `FETCH_HEAD` is a file
//! the person's next `git merge FETCH_HEAD` reads. `--prune` because a claim
//! released on the remote has to *disappear* here, and a mirror that only ever
//! grows reports holders who let go an hour ago.
//!
//! **A failure is a line and never an exit.** The daemon is optional by
//! construction, so a dead network downgrades what it offers and never stops
//! it, and never reaches the exit code of anything the person is running.

use std::path::Path;
use std::process::{Command, Stdio};

/// Where the remote's `refs/ank/*` land in a repository this daemon watches.
///
/// Under `refs/ank/` rather than beside it, because ADR-85e6ff5b8b7c gives this
/// project one ref namespace and a second one would be a second answer to
/// "which refs are ank's". Inert to every verb: each reader of the plane strips
/// `refs/ank/claims/` or `refs/ank/proof/` and skips what matches neither, so a
/// build of the CLI that knows nothing about this mirror behaves exactly as it
/// did before one existed.
///
/// The remote is named in the path even though `origin` is the only one ank
/// arbitrates through, for the reason the CLI names it rather than discovering
/// it: a mirror whose refs do not say where they came from is a mirror that
/// cannot gain a second source without renaming the first.
pub const TRACKING: &str = "refs/ank/watch/origin/";

/// Whether this repository has the remote ank arbitrates through.
///
/// **Absence is not a failure**, exactly as it is for the CLI: a repository
/// with no remote is the mode every solo checkout is in, and a watcher that
/// complained once a minute about it would be reporting a configuration nobody
/// got wrong.
fn has_origin(root: &Path) -> bool {
    Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// What one fetch cycle did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fetched {
    /// The remote answered and the mirror is current.
    Ok,
    /// There is no remote to mirror, which is the nominal state of a solo
    /// checkout and not a fault.
    NoRemote,
}

/// Mirrors `refs/ank/*` of `origin` into [`TRACKING`], and touches nothing
/// else.
///
/// `Err` carries the first line of git's own complaint, which is what a person
/// reading the log needs and is never parsed by anything here: the exit code is
/// what decided, as ADR-9307e5d214a7 requires.
pub fn mirror(root: &Path) -> Result<Fetched, String> {
    if !has_origin(root) {
        return Ok(Fetched::NoRemote);
    }
    let spec = format!("+refs/ank/*:{TRACKING}*");
    let out = Command::new("git")
        .args([
            "fetch",
            "--quiet",
            "--prune",
            "--no-tags",
            "--no-write-fetch-head",
            "--no-recurse-submodules",
            "origin",
            spec.as_str(),
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("git fetch: {e}"))?;
    if out.status.success() {
        return Ok(Fetched::Ok);
    }
    let said = String::from_utf8_lossy(&out.stderr);
    Err(said
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("no reason given")
        .to_string())
}

/// What the mirror holds right now: every tracking ref with the object it
/// points at, one per line, sorted as git sorts them.
///
/// This is how a change to somebody else's claims becomes news. The fetch above
/// says whether git succeeded, never whether anything moved -- `--quiet` and a
/// zero exit are the same on a remote that has not changed since the last
/// minute -- so what moved is answered by looking, and looking is one
/// `for-each-ref` per fetch cycle rather than per poll.
///
/// A repository git cannot answer about reads as an empty mirror, which is what
/// a repository with no tracking refs is. The caller compares two readings and
/// nothing else, so a failure costs a comparison that says "unchanged" and never
/// a wrong event.
pub fn mirrored(root: &Path) -> Vec<String> {
    Command::new("git")
        .args([
            "for-each-ref",
            "--format=%(refname)%09%(objectname)",
            TRACKING,
        ])
        .current_dir(root)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tracking_namespace_is_neither_of_the_two_the_cli_reads() {
        // The property that makes this mirror inert to every existing verb:
        // `context::plane` strips these two prefixes and skips whatever matches
        // neither. If this ever started with one of them, a mirrored claim
        // would read as a claim taken in this clone.
        assert!(!TRACKING.starts_with("refs/ank/claims/"));
        assert!(!TRACKING.starts_with("refs/ank/proof/"));
        assert!(TRACKING.starts_with("refs/ank/"));
        assert!(TRACKING.ends_with('/'));
    }
}
