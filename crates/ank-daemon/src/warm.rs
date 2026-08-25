//! Keeping a declared corpus's index current, and doing it through the CLI
//! (ADR-a22cd3196529).
//!
//! **The daemon is not a second implementation of the index.** `index.rs` is a
//! cache the CLI rebuilds from the files at read time, comparing a content hash
//! per `.ank/` file against what it stored; warming it is running a read.
//! Anything else here -- a second refresh written against the same schema, a
//! second parser, a second set of hashes -- would be a second thing to keep in
//! step with the format, and it would drift silently, because a stale cache
//! answers rather than fails.
//!
//! So this spawns `ank` and asks it for a listing. That is also what keeps
//! ADR-a22cd3196529's first clause true by construction: a process that has to
//! run the CLI to learn anything is not a second dispatch path, because it has
//! no dispatch of its own to be one. It is the rule ADR-8bd7ea0e8f2b already
//! set for the terminal reader, applied to the one other thing that reads a
//! corpus it did not write.
//!
//! **`--repo` and not a working directory.** The corpus is addressed by the
//! flag §6 gives for exactly that, which short-circuits the walk. A daemon that
//! set a current directory and let `ank` walk up from it would be discovering a
//! corpus, one directory at a time.
//!
//! **The verb is a read and takes no claim.** `find` opens the index, refreshes
//! what diverged, and writes nothing else; ADR-0bb7ea8991bc's rule that a claim
//! is renewed by working and not by reporting is kept here by never touching a
//! claim at all.
//!
//! **Nothing is believed over the files.** This is a cache warmer and its
//! failure mode is a listing that costs what it always cost. So a poll that
//! misses a change costs latency and never correctness -- the CLI hashes the
//! files on the next read regardless -- which is why the fingerprint below is
//! allowed to be as cheap as a stat.

use crate::declare::ANK_DIR;
use crate::fail::{Fail, Result};
use ank_contract::ExitCode;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where the CLI is, told or found.
///
/// **Told first.** `ANK_BIN` is what a test uses and what anybody with the two
/// binaries in unusual places uses; it is checked first so neither has to
/// arrange a `PATH`.
///
/// **Then beside this binary**, which is where the release puts `ank` today:
/// what a route places, it places together. Then `PATH`, which is where a reader who
/// installed one and built the other will have it.
pub fn locate_ank() -> Result<PathBuf> {
    if let Some(told) = std::env::var_os("ANK_BIN").filter(|v| !v.is_empty()) {
        let path = PathBuf::from(told);
        if path.is_file() {
            return Ok(path);
        }
        return Err(Fail::new(
            ExitCode::Environment,
            format!("ANK_BIN names {}, which is not a file", path.display()),
        )
        .with_hint("unset ANK_BIN to look beside this binary and then on PATH"));
    }
    let exe = if cfg!(windows) { "ank.exe" } else { "ank" };
    if let Some(sibling) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join(exe)))
        .filter(|p| p.is_file())
    {
        return Ok(sibling);
    }
    // `PATH`, resolved by the operating system when the name carries no
    // separator. Whether it is there is answered by the first spawn rather than
    // by a walk of `PATH` written here, which would be a third rule for finding
    // an executable and would disagree with the shell on at least one platform.
    Ok(PathBuf::from(exe))
}

/// What a `.ank/` looks like right now: every file under it, with its length
/// and the instant it was last written.
///
/// **`index.db` is excluded, and that is load-bearing.** It is the file the
/// warming writes, so counting it would make every refresh look like a change
/// and the daemon would spin against its own output forever. Its journal and
/// shared-memory siblings go with it for the same reason.
///
/// **A stat and not a hash.** The question this answers is "is it worth
/// spawning a read", not "what is in these files": the CLI hashes them itself
/// on the next read, so a fingerprint that misses a modification costs a
/// listing that is cold, exactly as if no daemon were running, and never a
/// wrong answer.
pub fn fingerprint(ank: &Path) -> Vec<(String, u64, u128)> {
    let mut out = Vec::new();
    walk(ank, ank, &mut out);
    out.sort();
    out
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, u64, u128)>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            walk(root, &path, out);
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("index.db") {
            continue;
        }
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        out.push((rel, meta.len(), mtime));
    }
}

/// One read of `root`'s corpus, which is what leaves its index current.
///
/// Degrades and never fails: a corpus the CLI refuses -- a schema this build
/// does not read, a directory somebody removed under the daemon -- costs one
/// line on stderr and the next poll. Nothing depends on this process, so
/// nothing may be broken by it giving up on one corpus.
pub fn warm(ank_bin: &Path, root: &Path) -> std::result::Result<(), String> {
    let out = Command::new(ank_bin)
        .arg("find")
        .arg("--repo")
        .arg(root)
        .arg("--status")
        .arg("open")
        .arg("--json")
        .arg("--quiet")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("{}: {e}", ank_bin.display()))?;
    if out.status.success() {
        return Ok(());
    }
    let said = String::from_utf8_lossy(&out.stderr);
    Err(said.lines().next().unwrap_or("no reason given").to_string())
}

/// The corpus directory of a declared checkout.
pub fn ank_dir(root: &Path) -> PathBuf {
    root.join(ANK_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fingerprint_ignores_the_file_the_warming_writes() {
        let dir = std::env::temp_dir().join(format!("ank-daemon-fp-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("entities")).unwrap();
        std::fs::write(dir.join("config.yml"), "schema: 4\n").unwrap();
        let before = fingerprint(&dir);
        std::fs::write(dir.join("index.db"), "not a database").unwrap();
        std::fs::write(dir.join("index.db-wal"), "nor this").unwrap();
        assert_eq!(before, fingerprint(&dir));
        std::fs::write(dir.join("entities/TASK-0000.md"), "---\n").unwrap();
        assert_ne!(before, fingerprint(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
