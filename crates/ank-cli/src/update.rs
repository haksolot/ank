//! `update`: whether a newer release of this binary exists, and the route that
//! installs it (ADR-64f32c74a0f9, §4).
//!
//! **Only this verb reaches the network for the question.** No other verb
//! checks for a newer release or announces one: a notice printed by every verb
//! would be traffic and output every agent pays for on every run (ADR-f3d1).
//!
//! **The latest release is read with `git ls-remote --tags --refs`**, which is
//! plumbing ADR-9307e5d214a7 admits, against the repository releases are
//! published from. `ANK_UPDATE_REPOSITORY` names another one, a mirror
//! included, and it is the same seam the suite points at a bare repository.

use crate::cli::{CliError, Invocation, Result};
use crate::json::Obj;
use ank_contract::ExitCode;
use std::io::Write;
use std::path::Path;

/// Where releases are published, and so where their tags are read.
pub const REPOSITORY: &str = "https://github.com/haksolot/ank";

/// The environment variable naming another repository to read releases from.
pub const REPOSITORY_VAR: &str = "ANK_UPDATE_REPOSITORY";

/// A release version, compared per component as numbers: 0.10.0 is above
/// 0.9.0, which a comparison of strings gets backwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u64, u64, u64);

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

impl Version {
    /// `MAJOR.MINOR.PATCH`, three runs of digits and nothing else.
    pub fn parse(text: &str) -> Option<Version> {
        let mut parts = text.split('.');
        let mut next = || {
            let part = parts.next()?;
            if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            part.parse::<u64>().ok()
        };
        let version = Version(next()?, next()?, next()?);
        parts.next().is_none().then_some(version)
    }

    /// A tag of the exact release form `vMAJOR.MINOR.PATCH`, or `None` for any
    /// other tag: a pre-release, a two-component tag or a name like `latest`
    /// is not a release, however high it reads.
    pub fn of_tag(tag: &str) -> Option<Version> {
        Version::parse(tag.strip_prefix('v')?)
    }

    /// The version this binary was built with, which `ank --version` prints
    /// first.
    pub fn running() -> Version {
        Version::parse(env!("CARGO_PKG_VERSION")).expect("the package version is MAJOR.MINOR.PATCH")
    }
}

/// The highest release among `tags`, every tag of another form ignored.
pub fn latest<S: AsRef<str>>(tags: &[S]) -> Option<Version> {
    tags.iter()
        .filter_map(|t| Version::of_tag(t.as_ref()))
        .max()
}

/// The repository releases are read from.
pub fn repository() -> String {
    std::env::var(REPOSITORY_VAR)
        .ok()
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| REPOSITORY.to_string())
}

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    if !inv.has("--check") {
        return Err(CliError::new(
            ExitCode::Generic,
            "update installs nothing yet: installing lands with TASK-1c8c100554a1",
        )
        .with_hint("ank update --check"));
    }
    if inv.has("--version") {
        return Err(CliError::new(
            ExitCode::Generic,
            "--version with --check: --check reports the latest release and installs none",
        )
        .with_hint("ank update --check"));
    }
    let repository = repository();
    let running = Version::running();
    let latest = latest(&crate::git::tags_of(cwd, &repository)?);
    let newer = latest.is_some_and(|l| l > running);
    if inv.json() {
        let latest = latest.map(|l| l.to_string());
        let doc = Obj::document()
            .str("current", &running.to_string())
            .opt_str("latest", latest.as_deref())
            .bool("newer", newer)
            .finish();
        let _ = writeln!(out, "{doc}");
        return Ok(ExitCode::Ok);
    }
    let _ = writeln!(out, "{}", report(running, latest, newer, &repository));
    Ok(ExitCode::Ok)
}

/// The two versions in a column, then the answer in one line.
fn report(running: Version, latest: Option<Version>, newer: bool, repository: &str) -> String {
    let latest_text = latest.map_or_else(|| "none".to_string(), |l| l.to_string());
    let answer = match latest {
        None => format!("no release is published at {repository}"),
        Some(_) if newer => "a newer release exists: ank update installs it".to_string(),
        Some(_) => "up to date".to_string(),
    };
    format!("running  {running}\nlatest   {latest_text}\n{answer}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_per_component_as_numbers() {
        assert!(Version::of_tag("v0.10.0") > Version::of_tag("v0.9.0"));
        assert_eq!(
            latest(&["v0.9.0", "v0.10.0", "v0.2.0"]),
            Some(Version(0, 10, 0))
        );
    }

    #[test]
    fn only_the_exact_release_form_is_a_release() {
        for tag in [
            "v1.0.0-rc.1",
            "v1.0",
            "1.0.0",
            "v1.0.0.0",
            "latest",
            "v1..0",
            "v+1.0.0",
            "vx.0.0",
        ] {
            assert_eq!(Version::of_tag(tag), None, "{tag}");
        }
        assert_eq!(latest::<&str>(&[]), None);
    }

    #[test]
    fn the_report_answers_in_its_last_line() {
        let r = report(Version(0, 7, 0), Some(Version(0, 8, 0)), true, "x");
        assert_eq!(
            r,
            "running  0.7.0\nlatest   0.8.0\na newer release exists: ank update installs it"
        );
        let r = report(Version(0, 7, 0), None, false, "https://example.com/ank");
        assert!(r.ends_with("no release is published at https://example.com/ank"));
    }
}
