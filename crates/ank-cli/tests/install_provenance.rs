//! install.sh checks build provenance when it can, says which check it made,
//! and refuses when asked to and unable (TASK-1ea826115f7a, GitHub #789).
//!
//! **Run end to end against a staged release.** The archive and its `.sha256`
//! are served from a listener on 127.0.0.1 this file starts, reached through
//! `ANK_BASE_URL` with the real `curl`, so what runs is the path a person's
//! `curl | sh` takes and not a function lifted out of it. `gh` is a stub that
//! records its arguments and answers what the test chose, and `PATH` is a farm
//! of links to every executable the suite's own `PATH` holds except `gh`, so a
//! machine with the real one installed still has a case where there is none.
//!
//! POSIX only: install.sh is the Linux and macOS route, and install.ps1 is
//! exercised the same way on Windows by `.github/workflows/install.yml`.
#![cfg(unix)]

use crate::scratch;

use sha2::{Digest, Sha256};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::OnceLock;

/// What `--signer-workflow` names: the workflow that builds and attests.
const SIGNER: &str = "haksolot/ank/.github/workflows/release.yml";

fn script() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../install.sh")
}

/// The target install.sh resolves on this machine, which is the archive name
/// the staged release has to carry.
fn target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        other => panic!("no release target for {other:?}"),
    }
}

fn executable(path: &Path) {
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// A release directory holding `v<version>/<archive>` and its `.sha256`, the
/// layout release.yml publishes, served on a port of its own.
fn staged(version: &str) -> (String, String) {
    let root = scratch::dir("provenance-release");
    let name = format!("ank-{version}-{}", target());
    let inner = root.join("build").join(&name);
    fs::create_dir_all(&inner).unwrap();
    let ank = inner.join("ank");
    fs::write(
        &ank,
        format!("#!/bin/sh\necho \"ank {version} (staged)\"\n"),
    )
    .unwrap();
    executable(&ank);
    let served = root.join("srv").join(format!("v{version}"));
    fs::create_dir_all(&served).unwrap();
    let archive = format!("{name}.tar.gz");
    let status = Command::new("tar")
        .arg("czf")
        .arg(served.join(&archive))
        .arg("-C")
        .arg(root.join("build"))
        .arg(&name)
        .status()
        .expect("tar must be on PATH");
    assert!(status.success());
    let digest = hex::encode(Sha256::digest(fs::read(served.join(&archive)).unwrap()));
    fs::write(
        served.join(format!("{archive}.sha256")),
        format!("{digest}  {archive}\n"),
    )
    .unwrap();
    (serve(root.join("srv")), archive)
}

/// A static file server on 127.0.0.1, one connection at a time, answering 200
/// with the file a GET names under `root` and 404 otherwise. Its thread lives
/// as long as the test process, which is as long as anything asks it.
fn serve(root: PathBuf) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut line = String::new();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            if reader.read_line(&mut line).is_err() {
                continue;
            }
            loop {
                let mut header = String::new();
                match reader.read_line(&mut header) {
                    Ok(0) | Err(_) => break,
                    Ok(_) if header == "\r\n" || header == "\n" => break,
                    Ok(_) => {}
                }
            }
            let path = line.split_whitespace().nth(1).unwrap_or("/");
            let file = root.join(path.trim_start_matches('/'));
            let (status, body) = match fs::read(&file) {
                Ok(body) if file.is_file() => ("200 OK", body),
                _ => ("404 Not Found", b"not found".to_vec()),
            };
            let _ = write!(
                stream,
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });
    format!("http://{addr}")
}

/// Links to every executable on the suite's `PATH` but `gh`, made once.
fn farm() -> &'static Path {
    static FARM: OnceLock<PathBuf> = OnceLock::new();
    FARM.get_or_init(|| {
        let farm = scratch::dir("provenance-path-farm");
        for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name == "gh" || farm.join(&name).exists() {
                    continue;
                }
                let _ = std::os::unix::fs::symlink(entry.path(), farm.join(&name));
            }
        }
        farm
    })
}

/// A `gh` that appends one line of its arguments to `$ANK_GH_RECORD` and
/// exits `$ANK_GH_AUTH` for `auth status` and `$ANK_GH_VERIFY` for
/// `attestation verify`, 0 when unset.
fn gh_stub() -> PathBuf {
    let dir = scratch::dir("provenance-gh");
    let gh = dir.join("gh");
    fs::write(
        &gh,
        "#!/bin/sh\n\
         printf '%s\\n' \"$*\" >> \"$ANK_GH_RECORD\"\n\
         case \"$1 $2\" in\n\
         'auth status') exit \"${ANK_GH_AUTH:-0}\" ;;\n\
         'attestation verify') exit \"${ANK_GH_VERIFY:-0}\" ;;\n\
         esac\n\
         exit 0\n",
    )
    .unwrap();
    executable(&gh);
    dir
}

struct Install {
    out: Output,
    dir: PathBuf,
    gh: String,
}

impl Install {
    fn code(&self) -> Option<i32> {
        self.out.status.code()
    }
    fn stderr(&self) -> String {
        String::from_utf8_lossy(&self.out.stderr).to_string()
    }
    fn installed(&self) -> bool {
        self.dir.join("ank").exists()
    }
    /// The one line saying which check was made.
    fn verified(&self) -> Vec<String> {
        self.stderr()
            .lines()
            .filter(|l| l.contains("verified: "))
            .map(|l| l.trim().to_string())
            .collect()
    }
}

/// `sh install.sh --version <version> --dir <fresh> <args>`, with or without
/// the `gh` stub ahead of the farm.
fn install(version: &str, with_gh: bool, args: &[&str], env: &[(&str, &str)]) -> Install {
    let (base, _) = staged(version);
    let dir = scratch::dir("provenance-bin").join("bin");
    let record = scratch::dir("provenance-record").join("gh");
    let mut path = Vec::new();
    if with_gh {
        path.push(gh_stub());
    }
    path.push(farm().to_path_buf());
    let mut command = Command::new("sh");
    command
        .arg(script())
        .args(["--version", version, "--dir"])
        .arg(&dir)
        .args(args)
        .env("PATH", std::env::join_paths(path).unwrap())
        .env("ANK_BASE_URL", &base)
        .env("ANK_GH_RECORD", &record)
        .env_remove("ANK_REQUIRE_PROVENANCE")
        .env_remove("GH_TOKEN")
        .env_remove("GITHUB_TOKEN")
        .stdin(Stdio::null());
    for (k, v) in env {
        command.env(k, v);
    }
    let out = command.output().expect("sh must be on PATH");
    Install {
        out,
        dir,
        gh: fs::read_to_string(&record).unwrap_or_default(),
    }
}

/// With no `gh`, the archive is installed on its checksum and the run says so.
#[test]
fn without_gh_the_install_says_sha256_only() {
    let run = install("9.9.9", false, &[], &[]);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert!(run.installed());
    assert_eq!(run.verified().len(), 1, "{}", run.stderr());
    assert!(
        run.verified()[0].starts_with("verified: sha256 only"),
        "{}",
        run.stderr()
    );
}

/// With a signed-in `gh`, the attestation of the downloaded archive is checked
/// against this repository and the workflow that builds it.
#[test]
fn with_gh_the_attestation_is_verified_and_named() {
    let run = install("9.9.9", true, &[], &[]);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert!(run.installed());
    assert_eq!(
        run.verified(),
        ["verified: sha256 and provenance"],
        "{}",
        run.stderr()
    );
    let verify = run
        .gh
        .lines()
        .find(|l| l.starts_with("attestation verify "))
        .unwrap_or_else(|| panic!("gh attestation verify was not run: {}", run.gh));
    let archive = format!("ank-9.9.9-{}.tar.gz", target());
    let words: Vec<&str> = verify.split_whitespace().collect();
    assert!(
        words[2].ends_with(&format!("/{archive}")),
        "it did not verify the archive it downloaded: {verify}"
    );
    assert!(
        verify.contains("--repo haksolot/ank"),
        "the repository is not pinned: {verify}"
    );
    assert!(
        verify.contains(&format!("--signer-workflow {SIGNER}")),
        "the signer workflow is not pinned: {verify}"
    );
}

/// An attestation that does not verify is a refusal, flag or no flag: the
/// archive matched its hash and failed the stronger check.
#[test]
fn an_attestation_that_does_not_verify_refuses_with_4() {
    let run = install("9.9.9", true, &[], &[("ANK_GH_VERIFY", "1")]);
    assert_eq!(run.code(), Some(4), "{}", run.stderr());
    assert!(!run.installed(), "it refused and installed anyway");
    assert!(
        run.stderr().contains("gh attestation verify"),
        "the refusal does not name the command that failed:\n{}",
        run.stderr()
    );
}

/// A `gh` that is not signed in cannot ask the API, so the check is not made,
/// the run says so, and nothing is refused.
#[test]
fn a_gh_that_is_not_signed_in_is_sha256_only() {
    let run = install("9.9.9", true, &[], &[("ANK_GH_AUTH", "1")]);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert!(run.installed());
    assert_eq!(run.verified().len(), 1, "{}", run.stderr());
    assert!(run.verified()[0].starts_with("verified: sha256 only"));
    assert!(
        !run.gh.contains("attestation verify"),
        "verify was attempted with no credential: {}",
        run.gh
    );
}

/// A release at or below 0.9.4 was published before the attestations, so the
/// check is not attempted and the line says why.
#[test]
fn a_release_older_than_the_attestations_is_sha256_only() {
    let run = install("0.9.4", true, &[], &[]);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert!(run.installed());
    assert_eq!(run.verified().len(), 1, "{}", run.stderr());
    assert!(run.verified()[0].starts_with("verified: sha256 only"));
    assert!(!run.gh.contains("attestation verify"), "{}", run.gh);
}

/// `--require-provenance` refuses where the check cannot be made: 5 when there
/// is no `gh` able to make it, 4 when the release carries no attestation, and
/// the environment variable is the same request.
#[test]
fn require_provenance_refuses_when_the_check_cannot_be_made() {
    let run = install("9.9.9", false, &["--require-provenance"], &[]);
    assert_eq!(run.code(), Some(5), "{}", run.stderr());
    assert!(!run.installed());

    let run = install(
        "9.9.9",
        true,
        &["--require-provenance"],
        &[("ANK_GH_AUTH", "1")],
    );
    assert_eq!(run.code(), Some(5), "{}", run.stderr());
    assert!(!run.installed());

    let run = install("0.9.4", true, &["--require-provenance"], &[]);
    assert_eq!(run.code(), Some(4), "{}", run.stderr());
    assert!(!run.installed());

    let run = install("9.9.9", false, &[], &[("ANK_REQUIRE_PROVENANCE", "1")]);
    assert_eq!(run.code(), Some(5), "{}", run.stderr());
    assert!(!run.installed());

    let run = install("9.9.9", true, &["--require-provenance"], &[]);
    assert_eq!(run.code(), Some(0), "{}", run.stderr());
    assert_eq!(run.verified(), ["verified: sha256 and provenance"]);
}

/// `--help` names the flag and the variable.
#[test]
fn help_names_the_provenance_flag() {
    let out = Command::new("sh")
        .arg(script())
        .arg("--help")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&out.stderr);
    assert!(help.contains("--require-provenance"), "{help}");
    assert!(help.contains("ANK_REQUIRE_PROVENANCE"), "{help}");
}
