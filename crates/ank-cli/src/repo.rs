//! Repository discovery (§6).
//!
//! An agent is launched in an arbitrary subdirectory of the tree: resolution
//! walks up to the first `.ank/`, the way git does for `.git`.
//! `--repo <path>` short-circuits the walk without ever contradicting it —
//! the given path must contain a `.ank/`.

use crate::cli::CliError;
use std::path::{Path, PathBuf};

pub const ANK_DIR: &str = ".ank";

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    /// The directory containing `.ank/`.
    pub root: PathBuf,
    /// The `.ank/` directory itself.
    pub ank: PathBuf,
}

impl Repo {
    fn at(root: PathBuf) -> Repo {
        let ank = root.join(ANK_DIR);
        Repo { root, ank }
    }

    pub fn config_path(&self) -> PathBuf {
        self.ank.join("config.yml")
    }
}

fn missing(from: &Path) -> CliError {
    CliError::new(1, format!("no {ANK_DIR}/ found from {}", from.display())).with_hint("ank init")
}

/// Walks up from `start` to the first directory containing `.ank/`.
pub fn discover(start: &Path) -> Result<Repo> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(ANK_DIR).is_dir() {
            return Ok(Repo::at(dir.to_path_buf()));
        }
        cur = dir.parent();
    }
    Err(missing(start))
}

/// Explicit resolution through `--repo`. Accepts the directory containing
/// `.ank/`, or the `.ank/` itself — being off by one level is the most likely
/// mistake, and refusing it would gain nothing.
pub fn at(path: &Path) -> Result<Repo> {
    if path.join(ANK_DIR).is_dir() {
        return Ok(Repo::at(path.to_path_buf()));
    }
    if path.file_name().and_then(|n| n.to_str()) == Some(ANK_DIR) && path.is_dir() {
        if let Some(parent) = path.parent() {
            return Ok(Repo::at(parent.to_path_buf()));
        }
    }
    Err(missing(path))
}

/// Full resolution: `--repo` if given, otherwise the walk up from the current
/// directory.
pub fn resolve(repo_flag: Option<&str>, cwd: &Path) -> Result<Repo> {
    match repo_flag {
        Some(p) => at(Path::new(p)),
        None => discover(cwd),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-repo-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join(ANK_DIR).join("tasks")).unwrap();
            Temp(p)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn walks_up_from_a_deep_subdirectory() {
        let t = Temp::new();
        let deep = t.0.join("crates").join("ank-cli").join("src");
        fs::create_dir_all(&deep).unwrap();

        let repo = discover(&deep).unwrap();
        assert_eq!(repo.root, t.0);
        assert_eq!(repo.ank, t.0.join(ANK_DIR));
    }

    #[test]
    fn a_missing_ank_dir_exits_pointing_at_ank_init() {
        let empty = std::env::temp_dir().join(format!("ank-empty-{}", std::process::id()));
        fs::create_dir_all(&empty).unwrap();
        // The walk up may reach a real Ank repository depending on where temp
        // lives; we only assert when resolution does fail.
        if let Err(err) = discover(&empty) {
            assert_eq!(err.code, 1);
            assert_eq!(err.hint.as_deref(), Some("ank init"));
        }
        let _ = fs::remove_dir_all(&empty);
    }

    #[test]
    fn explicit_repo_accepts_the_root_and_the_ank_dir() {
        let t = Temp::new();
        assert_eq!(at(&t.0).unwrap().root, t.0);
        assert_eq!(at(&t.0.join(ANK_DIR)).unwrap().root, t.0);
    }

    #[test]
    fn the_repo_flag_short_circuits_the_walk_up() {
        let t = Temp::new();
        let elsewhere = std::env::temp_dir();
        let via_flag = resolve(Some(t.0.to_str().unwrap()), &elsewhere).unwrap();
        assert_eq!(via_flag.root, t.0);
    }
}
