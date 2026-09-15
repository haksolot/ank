//! `update`: the one verb that asks whether a newer release exists
//! (ADR-64f32c74a0f9, §4).
//!
//! **Through the binary, against a bare repository standing in for the release
//! repository.** The criterion is a statement about a process: what it prints,
//! what it exits with, which processes it starts. `ANK_UPDATE_REPOSITORY` is the
//! seam that points the verb at the stand-in, and it is the seam a mirror uses
//! in production, which is why the verb's help names it.
//!
//! Process counts are read from `GIT_TRACE` at an absolute path and never from
//! a wall clock (CLAUDE.md): a process that did not start leaves no line, on
//! every platform, however loaded the runner is.

mod fixture;
mod scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The version the binary was built with, which is what `update` reports as
/// running and what `ank --version` prints first.
const RUNNING: &str = env!("CARGO_PKG_VERSION");

/// What git prefixes the argument list of every git it runs, under `GIT_TRACE`.
const MARK: &str = "trace: built-in: git ";

/// The route executables `update` could start and must not under `--check`.
const ROUTES: &[&str] = &["npm", "npx", "sh", "curl", "powershell", "pwsh"];

/// git's global and system configuration for every process this suite spawns,
/// so a machine that signs commits by default cannot decide whether a fixture
/// commits.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("update-it-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A bare repository holding one commit and every tag in `tags` pointing at it.
fn release_repository(what: &str, tags: &[&str]) -> PathBuf {
    let bare = scratch::dir(what);
    git(&bare, &["init", "--bare", "-q"]);
    let tree = git(&bare, &["hash-object", "-t", "tree", "-w", "--stdin"]);
    let commit = git(
        &bare,
        &["commit-tree", "--no-gpg-sign", "-m", "release", &tree],
    );
    for tag in tags {
        git(&bare, &["update-ref", &format!("refs/tags/{tag}"), &commit]);
    }
    bare
}

/// A directory of stub route executables.
///
/// Each one appends `start:<name>` and then `arg:<value>` per argument to
/// `$ANK_STUB_RECORD`, and exits `$ANK_STUB_CODE`. Two of them do what the
/// route they stand in for does to the directory, and only when that code is 0
/// and `$ANK_STUB_INSTALLED` names a file: the installer (`sh`, `powershell`)
/// first records whether the running executable is still where it was, as
/// `exists:yes` or `exists:no`, then writes the file into the directory it was
/// given; `npm` writes it over `$ANK_STUB_TARGET`, the path npm owns.
///
/// The POSIX `sh` reads `--dir` out of its own arguments, which is the proof
/// that the flag carries the directory. `powershell` receives its script
/// encoded, so its stub is told the directory in `$ANK_STUB_DIR` instead and
/// the test decodes the script to check it.
fn stubs(what: &str) -> PathBuf {
    let dir = scratch::dir(what);
    for name in ROUTES {
        if cfg!(windows) {
            let rec = "\"%ANK_STUB_RECORD%\"";
            let mut script = format!("@echo off\r\n>> {rec} echo(start:{name}\r\n");
            if *name == "powershell" {
                script.push_str(&format!(
                    "if exist \"%ANK_STUB_DIR%\\ank.exe\" (>> {rec} echo(exists:yes) else (>> {rec} echo(exists:no)\r\n\
                     if \"%ANK_STUB_CODE%\"==\"0\" if defined ANK_STUB_INSTALLED copy /y \"%ANK_STUB_INSTALLED%\" \"%ANK_STUB_DIR%\\ank.exe\" >nul\r\n"
                ));
            }
            if *name == "npm" {
                script.push_str(
                    "if \"%ANK_STUB_CODE%\"==\"0\" if defined ANK_STUB_INSTALLED copy /y \"%ANK_STUB_INSTALLED%\" \"%ANK_STUB_TARGET%\" >nul\r\n",
                );
            }
            script.push_str(&format!(
                ":loop\r\nif \"%~1\"==\"\" goto done\r\n>> {rec} echo(arg:%~1\r\nshift\r\ngoto loop\r\n:done\r\nexit /b %ANK_STUB_CODE%\r\n"
            ));
            fs::write(dir.join(format!("{name}.cmd")), script).unwrap();
        } else {
            let act = match *name {
                "sh" => {
                    "prev=; for a in \"$@\"; do [ \"$prev\" != --dir ] || d=$a; prev=$a; done\n\
                     if [ -e \"$d/ank\" ]; then echo exists:yes; else echo exists:no; fi >> \"$ANK_STUB_RECORD\"\n\
                     if [ \"$ANK_STUB_CODE\" = 0 ] && [ -n \"${ANK_STUB_INSTALLED-}\" ]; then\n\
                     rm -f \"$d/ank\"; cp \"$ANK_STUB_INSTALLED\" \"$d/ank\"; chmod +x \"$d/ank\"; fi\n"
                }
                "npm" => {
                    "if [ \"$ANK_STUB_CODE\" = 0 ] && [ -n \"${ANK_STUB_INSTALLED-}\" ]; then\n\
                     rm -f \"$ANK_STUB_TARGET\"; cp \"$ANK_STUB_INSTALLED\" \"$ANK_STUB_TARGET\"; chmod +x \"$ANK_STUB_TARGET\"; fi\n"
                }
                _ => "",
            };
            let path = dir.join(name);
            fs::write(
                &path,
                format!(
                    "#!/bin/sh\n\
                     {{ echo start:{name}; for a in \"$@\"; do printf 'arg:%s\\n' \"$a\"; done; }} >> \"$ANK_STUB_RECORD\"\n\
                     {act}\
                     exit \"${{ANK_STUB_CODE:-0}}\"\n"
                ),
            )
            .unwrap();
            make_executable(&path);
        }
    }
    dir
}

fn make_executable(_path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(_path, fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// The stubs first on `PATH`, then whatever found git for this suite.
fn path_with(stubs: &Path) -> std::ffi::OsString {
    let mut dirs = vec![stubs.to_path_buf()];
    dirs.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    std::env::join_paths(dirs).unwrap()
}

struct Run {
    out: Output,
    trace: String,
    record: Option<String>,
}

impl Run {
    fn code(&self) -> Option<i32> {
        self.out.status.code()
    }
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.out.stdout).to_string()
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.out.stderr).to_string()
    }
    /// Every git command this run started, as its argument list.
    fn gits(&self) -> Vec<String> {
        self.trace
            .lines()
            .filter_map(|l| l.split_once(MARK).map(|(_, rest)| rest.to_string()))
            .collect()
    }
}

/// `ank <args>` from `dir`, with the route stubs on `PATH`, `GIT_TRACE` at an
/// absolute path, and the release repository named by the seam.
fn ank_in(dir: &Path, repository: &str, args: &[&str]) -> Run {
    ank_as(Path::new(ANK), dir, repository, args, &[])
}

/// The same run, of the executable at `exe` and with `env` added: the stubs
/// read what the test chose from there.
fn ank_as(
    exe: &Path,
    dir: &Path,
    repository: &str,
    args: &[&str],
    env: &[(&str, std::ffi::OsString)],
) -> Run {
    let bin = stubs("update-stubs");
    let record = scratch::dir("update-record").join("record");
    let trace = scratch::dir("update-trace").join("trace");
    let mut command = Command::new(exe);
    command
        .args(args)
        .current_dir(dir)
        .env("PATH", path_with(&bin))
        .env("ANK_STUB_RECORD", &record)
        .env("ANK_STUB_CODE", "0")
        .env("ANK_UPDATE_REPOSITORY", repository)
        .env("GIT_TRACE", &trace)
        .env("GIT_CONFIG_GLOBAL", isolated_git_config())
        .env("GIT_CONFIG_SYSTEM", isolated_git_config())
        .env("ANK_AGENT", "claude-code/update-it")
        .stdin(Stdio::null());
    for (key, value) in env {
        command.env(key, value);
    }
    // **A file this suite just wrote and made executable can be busy for a
    // moment**: another test thread's fork inherits the descriptor it was
    // written through until that child execs, and exec refuses a file open for
    // writing. The suite's race and not the verb's answer, so it is retried --
    // whether the busy file is the copy started here or a stub the verb starts.
    let mut attempts = 0;
    let out = loop {
        attempts += 1;
        let _ = fs::remove_file(&record);
        match command.output() {
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy && attempts < 20 => {}
            Err(e) => panic!("the binary must have been built: {e}"),
            Ok(out)
                if out.status.code() == Some(9)
                    && String::from_utf8_lossy(&out.stderr).contains("busy")
                    && attempts < 20 => {}
            Ok(out) => break out,
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    Run {
        out,
        trace: fs::read_to_string(&trace).unwrap_or_default(),
        record: fs::read_to_string(&record).ok(),
    }
}

fn check(repository: &Path, json: bool) -> Run {
    let empty = scratch::dir("update-cwd");
    let mut args = vec!["update", "--check"];
    if json {
        args.push("--json");
    }
    ank_in(&empty, &repository.to_string_lossy(), &args)
}

/// The latest release is the highest tag of the exact form vMAJOR.MINOR.PATCH,
/// compared per component as numbers, and every other tag is ignored however
/// high it reads.
const NEWER_TAGS: &[&str] = &[
    "v9000.9.0",
    "v9000.10.0",
    "v9999.0.0-rc.1",
    "v99999.0",
    "99999.0.0",
    "v99999.0.0.0",
    "latest",
    "release-99999.0.0",
];

#[test]
fn check_names_the_running_version_and_a_newer_release() {
    let bare = release_repository("update-newer", NEWER_TAGS);
    let run = check(&bare, false);
    assert_eq!(
        run.code(),
        Some(0),
        "a newer release is an answer, not a finding:\n{}{}",
        run.stdout(),
        run.stderr()
    );
    let stdout = run.stdout();
    let lines: Vec<&str> = stdout.lines().map(str::trim_end).collect();
    assert!(
        lines
            .iter()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["running", RUNNING]),
        "the running version is not printed:\n{stdout}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["latest", "9000.10.0"]),
        "the latest release is not v9000.10.0:\n{stdout}"
    );
    assert!(
        stdout.contains("a newer release exists"),
        "it does not say a newer release exists:\n{stdout}"
    );
}

#[test]
fn check_says_up_to_date_when_the_running_version_is_the_latest() {
    let tag = format!("v{RUNNING}");
    let bare = release_repository("update-current", &["v0.0.1", &tag, "v99999.0.0-beta"]);
    let run = check(&bare, false);
    assert_eq!(run.code(), Some(0), "{}{}", run.stdout(), run.stderr());
    let stdout = run.stdout();
    assert!(
        stdout
            .lines()
            .any(|l| l.split_whitespace().collect::<Vec<_>>() == ["latest", RUNNING]),
        "the latest release is not the running one:\n{stdout}"
    );
    assert!(stdout.contains("up to date"), "{stdout}");
    assert!(!stdout.contains("a newer release exists"), "{stdout}");
}

#[test]
fn check_json_carries_current_latest_and_newer() {
    let bare = release_repository("update-json-newer", NEWER_TAGS);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let doc = document(&run);
    assert_eq!(doc["current"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["latest"].as_str(), Some("9000.10.0"), "{doc:?}");
    assert_eq!(doc["newer"].as_bool(), Some(true), "{doc:?}");

    let tag = format!("v{RUNNING}");
    let bare = release_repository("update-json-current", &[&tag]);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let doc = document(&run);
    assert_eq!(doc["current"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["latest"].as_str(), Some(RUNNING), "{doc:?}");
    assert_eq!(doc["newer"].as_bool(), Some(false), "{doc:?}");
}

/// The `--json` document on stdout, which must be the only thing there.
fn document(run: &Run) -> serde_yaml::Value {
    let stdout = run.stdout();
    assert_eq!(
        stdout.trim().lines().count(),
        1,
        "--json printed more than one document:\n{stdout}"
    );
    serde_yaml::from_str(&stdout).unwrap_or_else(|e| panic!("not a document ({e}):\n{stdout}"))
}

/// **It starts no process but git**, and of git exactly one ls-remote: the
/// route stubs record nothing, and the trace names no other git ank started.
/// `upload-pack` is the far end of a local ls-remote, started by git.
#[test]
fn check_starts_one_git_ls_remote_and_nothing_else() {
    let bare = release_repository("update-processes", NEWER_TAGS);
    let run = check(&bare, false);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert_eq!(
        run.record, None,
        "--check started a route executable, which installs"
    );
    let gits = run.gits();
    let started: Vec<&String> = gits
        .iter()
        .filter(|g| !g.starts_with("upload-pack"))
        .collect();
    assert_eq!(
        started.len(),
        1,
        "ank started other git processes than one ls-remote: {gits:#?}"
    );
    assert!(
        started[0].starts_with("ls-remote --tags --refs "),
        "the one git is not ls-remote --tags --refs: {gits:#?}"
    );
    assert!(
        started[0].contains(&*bare.to_string_lossy()),
        "ls-remote did not ask the repository it was pointed at: {gits:#?}"
    );
}

#[test]
fn an_unreachable_repository_exits_9_naming_it() {
    let missing = scratch::path("update-no-such-repository");
    let run = check(&missing, false);
    assert_eq!(run.code(), Some(9), "{}{}", run.stdout(), run.stderr());
    assert!(
        run.stderr().contains(&*missing.to_string_lossy()),
        "the refusal does not name the repository it asked:\n{}",
        run.stderr()
    );
    assert_eq!(run.record, None);
}

/// **No other verb starts git ls-remote.** Counted over a run of `status` and
/// one of `context` in a corpus whose origin is a reachable repository holding
/// releases, so a verb that wanted to ask could have.
#[test]
fn status_and_context_start_no_ls_remote() {
    let bare = release_repository("update-origin", NEWER_TAGS);
    let tree = scratch::dir("update-corpus");
    git(&tree, &["init", "-q"]);
    git(&tree, &["commit", "-q", "--allow-empty", "-m", "root"]);
    git(&tree, &["remote", "add", "origin", &bare.to_string_lossy()]);
    let init = ank_in(&tree, &bare.to_string_lossy(), &["init"]);
    assert_eq!(init.code(), Some(0), "{}{}", init.stdout(), init.stderr());

    for verb in ["status", "context"] {
        let run = ank_in(&tree, &bare.to_string_lossy(), &[verb]);
        assert_eq!(run.code(), Some(0), "{verb}: {}", run.stderr());
        let gits = run.gits();
        assert!(
            !gits.is_empty(),
            "{verb} left no trace, so the count below would prove nothing"
        );
        let asked: Vec<&String> = gits.iter().filter(|g| g.starts_with("ls-remote")).collect();
        assert!(asked.is_empty(), "{verb} started git ls-remote: {asked:#?}");
        assert_eq!(run.record, None, "{verb} started a route executable");
    }
}

/// The seam is named where a caller reads the verb: a mirror is a real use.
#[test]
fn help_names_the_flags_and_the_repository_seam() {
    let out = Command::new(ANK)
        .args(["help", "update"])
        .output()
        .expect("the binary must have been built");
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(0), "{text}");
    for needle in ["--check", "--version", "ANK_UPDATE_REPOSITORY"] {
        assert!(
            text.contains(needle),
            "help update does not name {needle}:\n{text}"
        );
    }
}

/// **The document is pinned by a golden** (ADR-6fd69efb629c), captured from the
/// process. The running version is masked the way `tui.rs` masks what it knows
/// is volatile: it moves at every release and the shape does not, and the test
/// knows the value because the build told it.
#[test]
fn the_check_document_is_pinned_by_a_golden() {
    let bare = release_repository("update-golden", NEWER_TAGS);
    let run = check(&bare, true);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    let masked = run.stdout().replace(
        &format!("\"current\":\"{RUNNING}\""),
        "\"current\":\"<VERSION>\"",
    );
    assert_ne!(
        masked,
        run.stdout(),
        "the running version was not in the document"
    );
    fixture::pin("update-check", &masked);
}

// ---------------------------------------------------------------------------
// Installing, through the route that placed the binary (TASK-1c8c100554a1)
// ---------------------------------------------------------------------------

/// The executable's file name on this platform.
const EXE: &str = if cfg!(windows) { "ank.exe" } else { "ank" };

/// The raw URL each installer is fetched from, as the README documents it.
const INSTALL_SH: &str = "https://raw.githubusercontent.com/haksolot/ank/main/install.sh";
const INSTALL_PS1: &str = "https://raw.githubusercontent.com/haksolot/ank/main/install.ps1";

/// A copy of the built binary at `rel` under a fresh directory, which is where
/// the route under test would have placed it. A copy and never a link: the
/// running executable is resolved from the process, and a link would resolve
/// back into the cargo target directory this suite refuses.
fn placed(what: &str, rel: &str) -> PathBuf {
    let exe = scratch::dir(what).join(rel).join(EXE);
    fs::create_dir_all(exe.parent().unwrap()).unwrap();
    fs::copy(ANK, &exe).expect("the built binary must be copyable");
    make_executable(&exe);
    exe
}

/// Where the npm package puts the platform binary on a global install: the
/// platform package nested inside `@haksolot/ank`'s own `node_modules`.
fn npm_placed(what: &str) -> PathBuf {
    let platform = if cfg!(windows) {
        "ank-win32-x64"
    } else if cfg!(target_os = "macos") {
        "ank-darwin-arm64"
    } else {
        "ank-linux-x64-musl"
    };
    placed(
        what,
        &format!("lib/node_modules/@haksolot/ank/node_modules/@haksolot/{platform}/bin"),
    )
}

/// Each invocation the stubs recorded, in order, as its name and its lines.
fn invocations(record: &Option<String>) -> Vec<(String, Vec<String>)> {
    let mut all: Vec<(String, Vec<String>)> = Vec::new();
    for line in record.as_deref().unwrap_or_default().lines() {
        let line = line.trim_end();
        if let Some(name) = line.strip_prefix("start:") {
            all.push((name.to_string(), Vec::new()));
        } else if let Some((_, lines)) = all.last_mut() {
            lines.push(line.to_string());
        }
    }
    all
}

fn args_of(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter_map(|l| l.strip_prefix("arg:").map(str::to_string))
        .collect()
}

fn install(
    exe: &Path,
    repository: &Path,
    args: &[&str],
    env: &[(&str, std::ffi::OsString)],
) -> Run {
    let cwd = scratch::dir("update-install-cwd");
    let mut argv = vec!["update"];
    argv.extend_from_slice(args);
    ank_as(exe, &cwd, &repository.to_string_lossy(), &argv, env)
}

fn os(s: impl AsRef<std::ffi::OsStr>) -> std::ffi::OsString {
    s.as_ref().to_os_string()
}

/// The script a `powershell -EncodedCommand` was handed, decoded: base64 of
/// UTF-16LE, which is what PowerShell reads there.
fn decoded_command(args: &[String]) -> String {
    let at = args
        .iter()
        .position(|a| a == "-EncodedCommand")
        .unwrap_or_else(|| panic!("powershell was not handed -EncodedCommand: {args:?}"));
    let text = &args[at + 1];
    let mut bits = 0u32;
    let mut count = 0;
    let mut bytes = Vec::new();
    for c in text.bytes().filter(|&c| c != b'=') {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => panic!("not base64: {text}"),
        } as u32;
        bits = (bits << 6) | v;
        count += 6;
        if count >= 8 {
            count -= 8;
            bytes.push((bits >> count) as u8);
        }
    }
    let units: Vec<u16> = bytes
        .chunks(2)
        .map(|p| u16::from_le_bytes([p[0], *p.get(1).unwrap_or(&0)]))
        .collect();
    String::from_utf16(&units).expect("UTF-16LE")
}

/// **Any binary no npm tree holds runs the official installer for its
/// platform**, with the version, the directory of the running executable and
/// `--no-welcome`, and `update` exits with the installer's code.
#[test]
fn a_released_binary_runs_the_installer_and_exits_with_its_code() {
    let bare = release_repository("update-installer", NEWER_TAGS);
    let exe = placed("update-installer-bin", "bin");
    let dir = exe.parent().unwrap().to_path_buf();
    let run = install(
        &exe,
        &bare,
        &[],
        &[("ANK_STUB_CODE", os("42")), ("ANK_STUB_DIR", os(&dir))],
    );
    assert_eq!(
        run.code(),
        Some(42),
        "update exits with the route's code:\n{}{}",
        run.stdout(),
        run.stderr()
    );
    let calls = invocations(&run.record);
    if cfg!(windows) {
        let names: Vec<&str> = calls.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, ["powershell"], "{:?}", run.record);
        let script = decoded_command(&args_of(&calls[0].1));
        for needle in [INSTALL_PS1, "-Version v9000.10.0", "-NoWelcome"] {
            assert!(script.contains(needle), "{needle} is not in: {script}");
        }
        let given = script
            .split_once("-Dir '")
            .and_then(|(_, rest)| rest.split_once("' "))
            .map(|(d, _)| d.replace("''", "'"))
            .unwrap_or_else(|| panic!("no -Dir in: {script}"));
        assert_same_directory(&given, &dir);
    } else {
        let names: Vec<&str> = calls.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names.len(), 2, "{:?}", run.record);
        assert!(
            names.contains(&"curl") && names.contains(&"sh"),
            "{names:?}"
        );
        let curl = &calls.iter().find(|(n, _)| n == "curl").unwrap().1;
        assert_eq!(args_of(curl), ["-fsSL", INSTALL_SH]);
        let sh = &calls.iter().find(|(n, _)| n == "sh").unwrap().1;
        let args = args_of(sh);
        assert_eq!(args.len(), 7, "{args:?}");
        assert_eq!(args[..5], ["-s", "--", "--version", "v9000.10.0", "--dir"]);
        assert_eq!(args[6], "--no-welcome");
        assert_same_directory(&args[5], &dir);
    }
}

/// The directory a route was handed is the one the executable was placed in,
/// compared as the filesystem resolves both: the running executable is
/// reported by the platform, which may spell a temporary directory otherwise
/// than the environment did (`/private/var` on macOS, a short name on Windows).
fn assert_same_directory(given: &str, placed: &Path) {
    let resolve = |p: &Path| fs::canonicalize(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    assert_eq!(
        resolve(Path::new(given)),
        resolve(placed),
        "the route was handed {given}, and the executable is in {}",
        placed.display()
    );
}

/// **A binary inside the npm package runs npm**, with the version and nothing
/// else, and never the installer.
#[test]
fn a_binary_inside_the_npm_package_runs_npm_install() {
    let bare = release_repository("update-npm", NEWER_TAGS);
    let exe = npm_placed("update-npm-bin");
    let run = install(&exe, &bare, &[], &[("ANK_STUB_CODE", os("42"))]);
    assert_eq!(run.code(), Some(42), "{}{}", run.stdout(), run.stderr());
    let calls = invocations(&run.record);
    assert_eq!(calls.len(), 1, "{:?}", run.record);
    assert_eq!(calls[0].0, "npm");
    assert_eq!(
        args_of(&calls[0].1),
        ["install", "-g", "@haksolot/ank@9000.10.0"]
    );
}

/// **At or above the latest release, nothing is installed** and no route
/// starts; `--version` installs what it names, older included.
#[test]
fn an_up_to_date_binary_installs_nothing_unless_a_version_is_named() {
    let tag = format!("v{RUNNING}");
    for tags in [vec![tag.as_str()], vec!["v0.0.1"]] {
        let bare = release_repository("update-uptodate", &tags);
        let exe = placed("update-uptodate-bin", "bin");
        let run = install(&exe, &bare, &[], &[]);
        assert_eq!(
            run.code(),
            Some(0),
            "{tags:?}: {}{}",
            run.stdout(),
            run.stderr()
        );
        assert!(
            run.stdout().contains("up to date"),
            "{tags:?}: {}",
            run.stdout()
        );
        assert_eq!(run.record, None, "{tags:?}: a route started");
    }

    let bare = release_repository("update-older", &[&tag]);
    let exe = placed("update-older-bin", "bin");
    let dir = exe.parent().unwrap().to_path_buf();
    let run = install(
        &exe,
        &bare,
        &["--version", "0.0.1"],
        &[("ANK_STUB_CODE", os("3")), ("ANK_STUB_DIR", os(&dir))],
    );
    assert_eq!(run.code(), Some(3), "{}{}", run.stdout(), run.stderr());
    let record = run.record.clone().unwrap_or_default();
    let wanted = if cfg!(windows) {
        let calls = invocations(&run.record);
        decoded_command(&args_of(&calls[0].1))
    } else {
        record.clone()
    };
    assert!(
        wanted.contains("v0.0.1"),
        "--version 0.0.1 did not reach the route: {wanted}"
    );
}

/// **A binary under a cargo target directory is refused at 7**, naming cargo
/// build, and starts no route. The suite's own binary is exactly that.
#[test]
fn a_binary_under_a_cargo_target_directory_is_refused() {
    let bare = release_repository("update-cargo", NEWER_TAGS);
    let cwd = scratch::dir("update-cargo-cwd");
    let run = ank_in(&cwd, &bare.to_string_lossy(), &["update"]);
    assert_eq!(run.code(), Some(7), "{}{}", run.stdout(), run.stderr());
    assert!(run.stderr().contains("cargo build"), "{}", run.stderr());
    assert_eq!(run.record, None, "a route started");
}

/// **On Windows the running executable is out of the way before the installer
/// starts**; elsewhere it stays where it is until the installer replaces it.
#[test]
fn the_running_executable_is_renamed_aside_on_windows_only() {
    let bare = release_repository("update-aside", NEWER_TAGS);
    let exe = placed("update-aside-bin", "bin");
    let dir = exe.parent().unwrap().to_path_buf();
    let run = install(
        &exe,
        &bare,
        &[],
        &[
            ("ANK_STUB_CODE", os("0")),
            ("ANK_STUB_DIR", os(&dir)),
            ("ANK_STUB_INSTALLED", os(ANK)),
        ],
    );
    assert_eq!(run.code(), Some(0), "{}{}", run.stdout(), run.stderr());
    let record = run.record.clone().unwrap_or_default();
    let expected = if cfg!(windows) {
        "exists:no"
    } else {
        "exists:yes"
    };
    assert!(
        record.lines().any(|l| l.trim_end() == expected),
        "the installer did not find {expected}:\n{record}"
    );
    assert!(
        exe.is_file(),
        "nothing is at {} after the install",
        exe.display()
    );
}

/// **A failed install on Windows puts the running executable back**, so a
/// route that wrote nothing leaves the binary where it was.
#[test]
fn a_failed_install_leaves_the_binary_where_it_was() {
    let bare = release_repository("update-failed", NEWER_TAGS);
    let exe = placed("update-failed-bin", "bin");
    let dir = exe.parent().unwrap().to_path_buf();
    let run = install(
        &exe,
        &bare,
        &[],
        &[("ANK_STUB_CODE", os("4")), ("ANK_STUB_DIR", os(&dir))],
    );
    assert_eq!(run.code(), Some(4), "{}{}", run.stdout(), run.stderr());
    assert!(exe.is_file(), "the binary is gone from {}", exe.display());
}

/// **After the route exits 0, the installed binary is asked its version**, and
/// the same skill revision says nothing about the skills.
#[test]
fn the_same_skill_revision_names_nothing_after_the_install() {
    let bare = release_repository("update-same-skills", NEWER_TAGS);
    let exe = npm_placed("update-same-skills-bin");
    let run = install(
        &exe,
        &bare,
        &[],
        &[
            ("ANK_STUB_INSTALLED", os(ANK)),
            ("ANK_STUB_TARGET", os(&exe)),
        ],
    );
    assert_eq!(run.code(), Some(0), "{}{}", run.stdout(), run.stderr());
    let said = format!("{}{}", run.stdout(), run.stderr());
    assert!(!said.contains("skills --install"), "{said}");
    assert!(
        invocations(&run.record).iter().all(|(n, _)| n != "npx"),
        "update ran npx: {:?}",
        run.record
    );
}

/// **Another skill revision in the installed binary is named in one line**, and
/// `ank skills --install` is never run: no npx starts. The installed binary is
/// a script answering `--version` for another build, which is why this one is
/// POSIX only; Windows runs the same-revision case above.
#[cfg(unix)]
#[test]
fn another_skill_revision_names_skills_install_and_never_runs_it() {
    let bare = release_repository("update-other-skills", NEWER_TAGS);
    let exe = placed("update-other-skills-bin", "bin");
    let other = scratch::dir("update-other-skills-build").join("ank");
    fs::write(
        &other,
        "#!/bin/sh\necho 'ank 9000.10.0 (0123456789ab, skill fedcba987654)'\n",
    )
    .unwrap();
    make_executable(&other);
    let run = install(&exe, &bare, &[], &[("ANK_STUB_INSTALLED", os(&other))]);
    assert_eq!(run.code(), Some(0), "{}{}", run.stdout(), run.stderr());
    let said = format!("{}{}", run.stdout(), run.stderr());
    let lines: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("ank skills --install"))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "one line names ank skills --install:\n{said}"
    );
    assert!(
        invocations(&run.record).iter().all(|(n, _)| n != "npx"),
        "update ran npx: {:?}",
        run.record
    );
}
