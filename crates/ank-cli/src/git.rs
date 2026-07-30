//! Minimal git runner (§12).
//!
//! Ank calls the git binary, never a library: `accept` and `check` rest on
//! signing, and `git commit -S` / `git verify-commit` are three lines where a
//! cryptographic reimplementation would be a project. The counterpart is the
//! discipline imposed by ADR-b8884edcebe3: **plumbing only**, never porcelain,
//! whose output offers no stability contract across versions.
//!
//! A broken git environment is not a failure of the agent's work: absence, too
//! old a version and a directory outside a repository all exit with code 9,
//! with the exact command to run.

use crate::cli::CliError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Floor imposed by SSH signing and `gpg.ssh.allowedSignersFile`.
pub const MIN_VERSION: (u32, u32) = (2, 34);

const INSTALL_URL: &str = "https://git-scm.com/downloads";

/// Allowed plumbing subcommands. The list is closed so that adding a
/// porcelain command is a visible act in review, not an oversight.
const PLUMBING: &[&str] = &[
    "update-ref",
    "rev-parse",
    "verify-commit",
    "hash-object",
    "cat-file",
    "for-each-ref",
    "config",
    "--version",
];

pub type Result<T> = std::result::Result<T, CliError>;

fn env_missing() -> CliError {
    CliError::new(9, "git not found in PATH").with_hint(INSTALL_URL)
}

/// Runs git in `cwd`. Returns standard output with trailing whitespace
/// trimmed. A non-zero exit code yields the error along with stderr.
pub fn run(cwd: &Path, args: &[&str]) -> Result<String> {
    debug_assert!(
        args.first().map(|a| PLUMBING.contains(a)).unwrap_or(false),
        "porcelain forbidden (ADR-b8884edcebe3): {args:?}"
    );
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git {}: {e}", args.join(" ")))
            }
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(CliError::new(
            9,
            format!("git {} failed: {stderr}", args.join(" ")),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// The installed version, as `(major, minor)`.
pub fn version() -> Result<(u32, u32)> {
    let out = Command::new("git").arg("--version").output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            env_missing()
        } else {
            CliError::new(9, format!("git --version: {e}"))
        }
    })?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_version(&text).ok_or_else(|| {
        CliError::new(9, format!("unreadable git version: {}", text.trim())).with_hint(INSTALL_URL)
    })
}

/// `git version 2.43.0.windows.1` -> `(2, 43)`. Tolerates distribution
/// suffixes, which vary between platforms.
pub fn parse_version(text: &str) -> Option<(u32, u32)> {
    let rest = text.split_whitespace().find(|w| {
        w.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && w.contains('.')
    })?;
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub fn check_version(found: (u32, u32)) -> Result<()> {
    if found < MIN_VERSION {
        return Err(CliError::new(
            9,
            format!(
                "git {}.{} too old, {}.{} required for SSH signing",
                found.0, found.1, MIN_VERSION.0, MIN_VERSION.1
            ),
        )
        .with_hint(INSTALL_URL));
    }
    Ok(())
}

/// Root of the git repository containing `cwd`.
pub fn toplevel(cwd: &Path) -> Result<PathBuf> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git rev-parse: {e}"))
            }
        })?;
    if !out.status.success() {
        return Err(CliError::new(
            9,
            format!("{} is not inside a git repository", cwd.display()),
        )
        .with_hint("git init"));
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// Checks the whole git environment: presence, version, and repository.
pub fn ensure_usable(cwd: &Path) -> Result<PathBuf> {
    check_version(version()?)?;
    toplevel(cwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_parsed_with_its_distribution_suffixes() {
        assert_eq!(parse_version("git version 2.43.0"), Some((2, 43)));
        assert_eq!(parse_version("git version 2.43.0.windows.1"), Some((2, 43)));
        assert_eq!(
            parse_version("git version 2.34.1 (Apple Git-141)"),
            Some((2, 34))
        );
        assert_eq!(parse_version("git version 3.0.0"), Some((3, 0)));
        assert_eq!(parse_version("not a version"), None);
    }

    #[test]
    fn the_version_floor_is_refused_with_code_9_and_the_link() {
        let err = check_version((2, 33)).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains("2.33"), "{}", err.message);
        assert!(err.message.contains("2.34"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some(INSTALL_URL));

        assert!(check_version((2, 34)).is_ok(), "the floor is inclusive");
        assert!(check_version((2, 45)).is_ok());
    }

    #[test]
    fn outside_a_git_repository_exits_with_code_9_and_git_init() {
        let dir = std::env::temp_dir().join(format!("ank-nogit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // A temporary directory may itself sit under a git repository; we only
        // assert when it does not, otherwise the assertion would be wrong.
        if toplevel(&dir).is_err() {
            let err = toplevel(&dir).unwrap_err();
            assert_eq!(err.code, 9);
            assert_eq!(err.hint.as_deref(), Some("git init"));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
