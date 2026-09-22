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
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Where releases are published, and so where their tags are read.
pub const REPOSITORY: &str = "https://github.com/haksolot/ank";

/// The environment variable naming another repository to read releases from.
pub const REPOSITORY_VAR: &str = ank_contract::env::ANK_UPDATE_REPOSITORY;

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
        return install(inv, cwd, out);
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

/// Where each installer is fetched from: the raw file on the default branch,
/// which is the route the README documents and so the one a person already ran.
const INSTALL_SH: &str = "https://raw.githubusercontent.com/haksolot/ank/main/install.sh";
const INSTALL_PS1: &str = "https://raw.githubusercontent.com/haksolot/ank/main/install.ps1";

/// The npm package, whose tree holds the platform binary on a global install.
const NPM_PACKAGE: &str = "@haksolot/ank";

/// The route that placed a binary, and so the one that replaces it
/// (ADR-64f32c74a0f9).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Route {
    /// Inside the npm package's tree: npm owns the file, and replacing it
    /// underneath would leave npm's record wrong.
    Npm,
    /// Any other released binary: the installer for this platform, told the
    /// directory the running executable is in.
    Installer,
}

/// `--version`, as the release it names: `v0.8.0` and `0.8.0` are one request,
/// as they are to both installers.
fn named_version(raw: &str) -> Result<Version> {
    Version::parse(raw.strip_prefix('v').unwrap_or(raw)).ok_or_else(|| {
        CliError::new(
            ExitCode::Generic,
            format!("--version {raw}: a release is MAJOR.MINOR.PATCH, with or without a leading v"),
        )
        .with_hint("ank update --check")
    })
}

/// Whether `exe` sits under a directory cargo builds into.
///
/// **Recognised by what cargo writes there, not by a directory name**: every
/// target directory carries a `CACHEDIR.TAG` cargo signs as its own, whatever
/// `CARGO_TARGET_DIR` called it, and a directory merely named `target` holds a
/// released binary as legitimately as any other.
fn under_cargo_target(exe: &Path) -> bool {
    exe.ancestors().skip(1).any(|dir| {
        std::fs::read_to_string(dir.join("CACHEDIR.TAG"))
            .is_ok_and(|tag| tag.contains("created by cargo"))
    })
}

/// The route that placed `exe`: npm when a `node_modules/@haksolot/ank`
/// directory holds it, the installer otherwise. Read from where the file is,
/// because that is where npm put it (npm/ank/bin/wrapper.js resolves the
/// platform binary out of the package's own tree).
fn route_of(exe: &Path) -> Route {
    let parts: Vec<String> = exe
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect();
    let (scope, name) = NPM_PACKAGE.split_once('/').expect("a scoped package");
    let npm = parts
        .windows(3)
        .any(|w| w[0] == "node_modules" && w[1] == scope && w[2] == name);
    if npm {
        Route::Npm
    } else {
        Route::Installer
    }
}

/// Installs a release through the route that placed the running binary.
fn install(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    if inv.json() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--json: update without --check returns no document, the route writes to the same stdout",
        )
        .with_hint("ank update --check --json"));
    }
    let exe = std::env::current_exe().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("the running executable cannot be located: {e}"),
        )
    })?;
    if under_cargo_target(&exe) {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            format!(
                "{} is a build under a cargo target directory, placed by no route that could replace it",
                exe.display()
            ),
        )
        .with_hint("cargo build"));
    }
    let running = Version::running();
    let version = match inv.value("--version") {
        Some(raw) => named_version(raw)?,
        None => {
            let repository = repository();
            let latest = latest(&crate::git::tags_of(cwd, &repository)?);
            match latest {
                Some(latest) if latest > running => latest,
                _ => {
                    let _ = writeln!(out, "{}", report(running, latest, false, &repository));
                    return Ok(ExitCode::Ok);
                }
            }
        }
    };
    let route = route_of(&exe);
    let dir = exe
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let (mut children, shown) = match route {
        Route::Npm => npm(version)?,
        Route::Installer => installer(version, &dir)?,
    };
    let _ = writeln!(out, "installing {version} over {running}: {shown}");
    let _ = out.flush();

    let aside = set_aside(&exe, &route)?;
    let code = children.wait(&shown);
    if let Some(aside) = &aside {
        if code != Some(0) && !exe.exists() {
            let _ = std::fs::rename(aside, &exe);
        }
    }
    match code {
        Some(0) => {
            if let Some(line) = skills_line(&exe, out) {
                let _ = writeln!(out, "{line}");
            }
            Ok(ExitCode::Ok)
        }
        // **The route failing is this verb failing, and its code is the
        // answer**, which is `skills --install`'s reasoning about npx: none of
        // ank's codes means what the installer meant.
        Some(code) => {
            eprintln!("{shown} exited {code}");
            let _ = out.flush();
            std::process::exit(code);
        }
        None => Err(CliError::new(
            ExitCode::Generic,
            format!("{shown} was stopped by a signal before it exited"),
        )),
    }
}

/// The processes a route runs, started, and whose exit is the route's.
struct Children(Vec<(String, std::process::Child)>);

impl Children {
    /// Every child waited on; the code is the last non-zero one in pipeline
    /// order, so `curl` failing with `sh` then running an empty script is
    /// still a failure, and `sh` failing is the installer's own code.
    fn wait(&mut self, shown: &str) -> Option<i32> {
        let mut code = Some(0);
        for (name, child) in &mut self.0 {
            match child.wait() {
                Ok(status) => match status.code() {
                    Some(0) => {}
                    other => code = other,
                },
                Err(e) => {
                    eprintln!("{name} ({shown}): {e}");
                    code = Some(ExitCode::Environment.code());
                }
            }
        }
        code
    }
}

fn not_on_path(program: &str, shown: &str) -> CliError {
    CliError::new(
        ExitCode::Environment,
        format!("{program} is not on PATH, and this binary's route needs it"),
    )
    .with_hint(shown.to_string())
}

fn start(program: &Path, shown: &str, command: &mut Command) -> Result<std::process::Child> {
    command.spawn().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot run {}: {e}", program.display()),
        )
        .with_hint(shown.to_string())
    })
}

/// `npm install -g @haksolot/ank@<version>`.
fn npm(version: Version) -> Result<(Children, String)> {
    let spec = format!("{NPM_PACKAGE}@{version}");
    let shown = format!("npm install -g {spec}");
    let program = crate::skills::on_path("npm").ok_or_else(|| not_on_path("npm", &shown))?;
    let child = start(
        &program,
        &shown,
        Command::new(&program)
            .args(["install", "-g", &spec])
            .stdin(Stdio::null()),
    )?;
    Ok((Children(vec![("npm".into(), child)]), shown))
}

/// The installer for this platform, fetched the way the README fetches it.
///
/// **On Linux and macOS, `curl -fsSL <install.sh> | sh -s -- ...`**, as two
/// processes joined by a pipe this one makes, so no shell parses the directory.
/// **On Windows, PowerShell handed the README's line as `-EncodedCommand`**:
/// base64 of UTF-16LE is what it reads there, and no quoting rule of cmd or of
/// PowerShell's own command line can reach inside it. The directory is quoted
/// once, as a PowerShell literal.
fn installer(version: Version, dir: &Path) -> Result<(Children, String)> {
    let tag = format!("v{version}");
    let dir_text = dir.to_string_lossy().to_string();
    if cfg!(windows) {
        let script = format!(
            "& ([scriptblock]::Create((irm {INSTALL_PS1}))) -Version {tag} -Dir '{}' -NoWelcome",
            dir_text.replace('\'', "''")
        );
        let shown = format!("powershell -Command \"{script}\"");
        let program = crate::skills::on_path("powershell")
            .or_else(|| crate::skills::on_path("pwsh"))
            .ok_or_else(|| not_on_path("powershell", &shown))?;
        let child = start(
            &program,
            &shown,
            Command::new(&program)
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-EncodedCommand",
                    &encoded_command(&script),
                ])
                .stdin(Stdio::null()),
        )?;
        return Ok((Children(vec![("powershell".into(), child)]), shown));
    }
    let shown =
        format!("curl -fsSL {INSTALL_SH} | sh -s -- --version {tag} --dir {dir_text} --no-welcome");
    let curl = crate::skills::on_path("curl").ok_or_else(|| not_on_path("curl", &shown))?;
    let sh = crate::skills::on_path("sh").ok_or_else(|| not_on_path("sh", &shown))?;
    let mut fetch = start(
        &curl,
        &shown,
        Command::new(&curl)
            .args(["-fsSL", INSTALL_SH])
            .stdin(Stdio::null())
            .stdout(Stdio::piped()),
    )?;
    let script = fetch.stdout.take().expect("stdout was piped");
    let run = start(
        &sh,
        &shown,
        Command::new(&sh)
            .args(["-s", "--", "--version", &tag, "--dir"])
            .arg(dir)
            .arg("--no-welcome")
            .stdin(Stdio::from(script)),
    );
    let run = match run {
        Ok(run) => run,
        Err(e) => {
            let _ = fetch.kill();
            let _ = fetch.wait();
            return Err(e);
        }
    };
    Ok((
        Children(vec![("curl".into(), fetch), ("sh".into(), run)]),
        shown,
    ))
}

/// `script` as PowerShell's `-EncodedCommand` reads it: base64 of UTF-16LE.
///
/// Padded with trailing spaces to a length whose encoding carries no `=`, which
/// cmd would otherwise read as an argument delimiter if the program found on
/// PATH is a batch file.
fn encoded_command(script: &str) -> String {
    let mut units: Vec<u16> = script.encode_utf16().collect();
    while units.len() % 3 != 0 {
        units.push(u16::from(b' '));
    }
    let bytes: Vec<u8> = units.iter().flat_map(|u| u.to_le_bytes()).collect();
    base64(&bytes)
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut text = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | (u32::from(*b) << (16 - 8 * i)));
        for i in 0..4 {
            if i <= chunk.len() {
                text.push(ALPHABET[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                text.push('=');
            }
        }
    }
    text
}

/// **On Windows, the running executable is renamed aside before the route
/// writes**: a running `.exe` cannot be overwritten there, and it can be
/// renamed. Elsewhere nothing moves, since the route replaces the file and the
/// running process keeps the one it opened.
///
/// Beside itself for the installer. For npm, above the outermost
/// `node_modules`, on the same volume and outside the tree npm removes, whose
/// directories a file still running inside them could hold open. An aside a
/// previous update left is removed first, since the process that held it is
/// gone.
fn set_aside(exe: &Path, route: &Route) -> Result<Option<PathBuf>> {
    if !cfg!(windows) {
        return Ok(None);
    }
    let name = format!(
        "{}.old",
        exe.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "ank.exe".into())
    );
    let beside = exe.with_file_name(&name);
    let aside = match route {
        Route::Installer => beside,
        Route::Npm => exe
            .ancestors()
            .filter(|a| a.file_name().is_some_and(|n| n == "node_modules"))
            .last()
            .and_then(Path::parent)
            .map(|root| root.join(&name))
            .unwrap_or(beside),
    };
    let _ = std::fs::remove_file(&aside);
    std::fs::rename(exe, &aside).map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!(
                "{} cannot be moved aside to {} before the install: {e}",
                exe.display(),
                aside.display()
            ),
        )
    })?;
    Ok(Some(aside))
}

/// The one line naming `ank skills --install`, when the binary now at `exe`
/// carries another skill revision than this one. It is never run from here
/// (ADR-64f32c74a0f9): it chains npx and the network behind a verb that was
/// asked to update a binary.
fn skills_line(exe: &Path, out: &mut dyn Write) -> Option<String> {
    let output = Command::new(exe)
        .arg("--version")
        .stdin(Stdio::null())
        .output();
    let text = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => {
            eprintln!(
                "{} did not answer --version after the install",
                exe.display()
            );
            return None;
        }
    };
    let installed = text.lines().next().unwrap_or_default();
    let _ = writeln!(out, "installed: {installed}");
    let revision = skill_revision(installed)?;
    (revision != env!("ANK_SKILL")).then(|| {
        format!(
            "the skills changed ({} to {revision}): ank skills --install installs them",
            env!("ANK_SKILL")
        )
    })
}

/// The revision in `ank X (commit, skill <revision>)`.
fn skill_revision(line: &str) -> Option<&str> {
    let (_, rest) = line.split_once("skill ")?;
    rest.split(')').next().map(str::trim)
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
    fn the_encoded_command_is_base64_of_utf16le_with_no_padding() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        let encoded = encoded_command("irm x");
        assert!(!encoded.contains('='), "{encoded}");
        // "irm x " in UTF-16LE.
        assert_eq!(encoded, "aQByAG0AIAB4ACAA");
    }

    #[test]
    fn the_npm_tree_is_recognised_by_where_the_file_is() {
        assert_eq!(
            route_of(Path::new(
                "/usr/lib/node_modules/@haksolot/ank/node_modules/@haksolot/ank-linux-x64-musl/bin/ank"
            )),
            Route::Npm
        );
        assert_eq!(
            route_of(Path::new("/home/u/.local/bin/ank")),
            Route::Installer
        );
        assert_eq!(
            route_of(Path::new(
                "/x/node_modules/@haksolot/ank-linux-x64-musl/bin/ank"
            )),
            Route::Installer
        );
    }

    #[test]
    fn the_skill_revision_is_read_off_the_version_line() {
        assert_eq!(
            skill_revision("ank 0.8.0 (abc1234, skill 0123456789ab)"),
            Some("0123456789ab")
        );
        assert_eq!(skill_revision("ank 0.1.0"), None);
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
