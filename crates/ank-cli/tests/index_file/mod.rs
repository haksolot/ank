//! Where the binary under test keeps its index, named once for the suite
//! (TASK-a8d3219e84fb).
//!
//! The index is one file per schema, `.ank/index.db.<N>` (ADR-3db9735a7036),
//! and `N` is a constant of the binary. A test cannot import it, the binary
//! being no library, so it is read off the source that declares it: one place
//! states the number, and every test that forces a cold rebuild or looks at the
//! file goes through here instead of spelling a name that moves with each bump.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// The index schema version the binary under test was built with.
pub fn schema() -> u32 {
    const SOURCE: &str = include_str!("../../src/index.rs");
    SOURCE
        .lines()
        .find_map(|l| l.strip_prefix("pub const SCHEMA_VERSION: u32 = "))
        .and_then(|rest| rest.trim_end_matches(';').trim().parse().ok())
        .expect("src/index.rs declares `pub const SCHEMA_VERSION: u32 = <N>;`")
}

/// The file name of the binary's own index: `index.db.<N>`.
pub fn name() -> String {
    format!("index.db.{}", schema())
}

/// The binary's own index, for the `.ank/` directory `ank`.
pub fn path(ank: &Path) -> PathBuf {
    ank.join(name())
}

/// Removes every index under `ank`, of every schema, with its WAL siblings:
/// what `rm .ank/index.db*` does, and so a cold rebuild on the next verb.
pub fn remove_all(ank: &Path) {
    let Ok(entries) = std::fs::read_dir(ank) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with("index.db") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}
