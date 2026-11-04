//! A signature verdict this machine could not check is never cached as final
//! (TASK-432f93acc03f).
//!
//! The index caches git's verdict on a ratification under the commit, the
//! declared keys and the signing configuration. None of those names the
//! machine's gpg or its keyring, so a verdict that was only ever a fact about
//! them -- gpg missing (git answers `N` on a signed commit), or the public key
//! missing (`E`) -- must be asked again on the next run, while a verdict about
//! the commit itself (good, bad, expired, revoked) is served from the cache
//! without starting git's verifier at all.
//!
//! Through the binary throughout. OpenPGP through a stand-in `gpg.program`, as
//! in `signers.rs`, because a runner is not guaranteed a gpg: git is real and
//! speaks to it through `--status-fd` lines. The stand-in's keyring is a file
//! in `$GNUPGHOME`, holding the verdict it gives, and "gpg missing" is the
//! configured `gpg.program` path with nothing at it -- the same configuration
//! either way, which is what the cache key sees on a machine where gpg is
//! installed after the fact.

use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");
const HUMAN: &str = "human:marie";
const A: &str = "ADR-0000000043fa";
const EXE: &str = std::env::consts::EXE_SUFFIX;
const FPR: &str = "0123456789ABCDEF0123456789ABCDEF01234567";

/// The stand-in gpg. Signing prints an armoured block and `SIG_CREATED`;
/// verifying reads `$GNUPGHOME/keyring`: absent is the key not held (`ERRSIG`
/// and `NO_PUBKEY`, git's `E`), otherwise the word in it picks git's letter.
const GPG_STUB: &str = r#"
use std::io::Read;
const FPR: &str = "0123456789ABCDEF0123456789ABCDEF01234567";
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut sink = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut sink);
    let id = &FPR[24..];
    if !args.iter().any(|a| a == "--verify") {
        print!("-----BEGIN PGP SIGNATURE-----\n\niQEzBAABCAAdFiEEASNFZ4mrze8=\n=abcd\n-----END PGP SIGNATURE-----\n");
        eprint!("\n[GNUPG:] SIG_CREATED D 22 8 00 1767225600 {FPR}\n");
        return;
    }
    let held = std::env::var_os("GNUPGHOME")
        .and_then(|h| std::fs::read_to_string(std::path::Path::new(&h).join("keyring")).ok());
    let valid = format!("[GNUPG:] VALIDSIG {FPR} 2026-01-01 1767225600 0 4 0 22 8 00 {FPR}\n");
    let said = match held.as_deref().map(str::trim) {
        None => {
            print!("[GNUPG:] NEWSIG\n[GNUPG:] ERRSIG {id} 22 8 00 1767225600 9 {FPR}\n[GNUPG:] NO_PUBKEY {id}\n");
            std::process::exit(2);
        }
        Some("bad") => format!("[GNUPG:] NEWSIG\n[GNUPG:] BADSIG {id} Marie <marie@example.com>\n"),
        Some("expired") => format!("[GNUPG:] NEWSIG\n[GNUPG:] EXPSIG {id} Marie <marie@example.com>\n{valid}"),
        Some("revoked") => format!("[GNUPG:] NEWSIG\n[GNUPG:] REVKEYSIG {id} Marie <marie@example.com>\n{valid}"),
        _ => format!("[GNUPG:] NEWSIG\n[GNUPG:] GOODSIG {id} Marie <marie@example.com>\n{valid}[GNUPG:] TRUST_ULTIMATE 0 pgp\n"),
    };
    print!("{said}");
}
"#;

fn gpg_stub() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let src = scratch::path("signature-cache-gpg-stub.rs");
        let exe = scratch::path(&format!("signature-cache-gpg-stub{EXE}"));
        fs::write(&src, GPG_STUB).unwrap();
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let out = Command::new(rustc)
            .arg("--edition=2021")
            .arg("-o")
            .arg(&exe)
            .arg(&src)
            .output()
            .expect("rustc builds the suite, so it is there to build the stub");
        assert!(out.status.success(), "{}", text(&out.stderr));
        exe
    })
    .as_path()
}

fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = scratch::path("signature-cache-gitconfig");
        fs::write(
            &p,
            "[commit]\n\tgpgsign = false\n[tag]\n\tgpgsign = false\n[user]\n\tname = t\n\temail = t@example.com\n\
             [gc]\n\tauto = 0\n[maintenance]\n\tauto = false\n[core]\n\tautocrlf = false\n\
             [init]\n\tdefaultBranch = main\n",
        )
        .unwrap();
        p
    })
    .as_path()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).to_string()
}

struct Corpus {
    repo: PathBuf,
    /// The configured `gpg.program`: the stand-in linked here, or nothing.
    gpg: PathBuf,
    gnupghome: PathBuf,
}

impl Corpus {
    /// One proposed ADR, a declared gpg key, and `A` ratified by a commit the
    /// stand-in signed: the anchor the commit walk reads.
    fn ratified(what: &str) -> Corpus {
        let c = Corpus::tagged(what);
        crate::legacy_anchor::commit(&|a| c.git(a), A, true);
        c
    }

    /// The same, ratified by the signed tag `accept` writes now.
    fn tagged(what: &str) -> Corpus {
        let repo = scratch::dir(what);
        let c = Corpus {
            gpg: repo.join("bin").join(format!("gpg{EXE}")),
            gnupghome: repo.join("gnupg"),
            repo,
        };
        fs::create_dir_all(c.repo.join(".ank/entities")).unwrap();
        fs::create_dir_all(c.repo.join("src")).unwrap();
        fs::create_dir_all(c.repo.join("bin")).unwrap();
        fs::create_dir_all(&c.gnupghome).unwrap();
        c.git(&["init", "-q", "-b", "main"]);
        fs::write(
            c.repo.join(".ank/config.yml"),
            format!(
                "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n\
                 signers:\n  marie@laptop:\n    keys:\n      - key: gpg {FPR}\n"
            ),
        )
        .unwrap();
        fs::write(c.repo.join(".gitignore"), ".ank/index.db*\nbin/\ngnupg/\n").unwrap();
        fs::write(c.repo.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        fs::write(
            c.repo.join(".ank/entities").join(format!("{A}.md")),
            format!(
                "---\nid: {A}\ntype: adr\nslug: a-decision\ntitle: A decision\n\
                 created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
                 scope:\n  - src/**\nconstraint: |\n  A binding rule.\nschema: 3\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
        c.git(&["add", "-A"]);
        c.git(&["commit", "-qm", "seed"]);
        c.git(&["config", "gpg.format", "openpgp"]);
        c.git(&["config", "gpg.program", c.gpg.to_str().unwrap()]);
        c.git(&["config", "user.signingkey", FPR]);
        c.install_gpg();

        let out = c.ank(&["accept", A], None);
        assert_eq!(out.status.code(), Some(0), "accept: {}", text(&out.stderr));
        c
    }

    fn spawn(&self, program: &str) -> Command {
        let mut c = Command::new(program);
        let config = isolated_git_config();
        let root = scratch::root();
        c.env("GIT_CONFIG_GLOBAL", config)
            .env("GIT_CONFIG_SYSTEM", config)
            .env("GNUPGHOME", &self.gnupghome)
            .env("TMPDIR", root)
            .env("TMP", root)
            .env("TEMP", root)
            .env_remove("GIT_TRACE")
            .current_dir(&self.repo);
        c
    }

    fn git(&self, args: &[&str]) -> String {
        let out = self
            .spawn("git")
            .args(args)
            .output()
            .expect("git must be on PATH");
        assert!(out.status.success(), "git {args:?}: {}", text(&out.stderr));
        text(&out.stdout).trim().to_string()
    }

    fn ank(&self, args: &[&str], trace: Option<&Path>) -> Output {
        let mut c = self.spawn(ANK);
        c.args(args).env("ANK_AGENT", HUMAN);
        if let Some(t) = trace {
            c.env("GIT_TRACE", t);
        }
        c.output().expect("the binary must have been built")
    }

    /// gpg becomes runnable at the configured path. A hard link and not a
    /// copy: a freshly written executable can be refused with ETXTBSY while
    /// another test thread forks.
    fn install_gpg(&self) {
        let _ = fs::remove_file(&self.gpg);
        fs::hard_link(gpg_stub(), &self.gpg).unwrap();
    }

    /// gpg is gone; the configuration naming it is not.
    fn uninstall_gpg(&self) {
        fs::remove_file(&self.gpg).unwrap();
    }

    /// What the keyring holds: `None` is the public key not imported.
    fn keyring(&self, verdict: Option<&str>) {
        let file = self.gnupghome.join("keyring");
        match verdict {
            Some(v) => fs::write(file, v).unwrap(),
            None => {
                let _ = fs::remove_file(file);
            }
        }
    }

    /// `check`, its report, and how many processes in it verified a
    /// signature: git's `%G?` asks and the gpg it started, one trace line
    /// each, counted from `GIT_TRACE` at an absolute path.
    fn check(&self, what: &str) -> (String, usize) {
        let trace = self.repo.join(format!("../{what}.trace"));
        let _ = fs::remove_file(&trace);
        let out = self.ank(&["check"], Some(&trace));
        let report = text(&out.stdout) + &text(&out.stderr);
        let gpg = self.gpg.to_str().unwrap().to_string();
        let verifies = fs::read_to_string(&trace)
            .unwrap_or_default()
            .lines()
            .filter(|l| l.contains("%G?") || l.contains("verify-tag") || l.contains(&gpg))
            .count();
        (report, verifies)
    }
}

const UNCHECKED: &str = "could not be checked here";

/// The fault lines `check` printed about `A`.
fn faults(report: &str) -> Vec<&str> {
    report
        .lines()
        .filter(|l| l.starts_with("error:") && l.contains(A))
        .collect()
}

/// The measured defect: gpg was not installed, git answered `N` for a signed
/// ratification, the index kept it, and installing gpg and importing the key
/// changed nothing until the index file was deleted.
#[test]
fn a_verdict_reached_without_gpg_is_asked_again_once_gpg_is_installed() {
    let c = Corpus::ratified("signature-cache-no-gpg");
    c.uninstall_gpg();
    c.keyring(None);
    let (report, _) = c.check("no-gpg");
    assert!(report.contains(UNCHECKED), "no gpg: unchecked\n{report}");
    assert!(faults(&report).is_empty(), "and never refused\n{report}");

    c.install_gpg();
    c.keyring(Some("good"));
    let (report, verifies) = c.check("installed");
    assert!(
        !report.contains(UNCHECKED),
        "gpg installed and the key imported, the index left in place: verified\n{report}"
    );
    assert!(faults(&report).is_empty(), "{report}");
    assert!(verifies > 0, "and it was asked again, not read back");
}

/// The same through a ratification tag, which git reads with `verify-tag`.
#[test]
fn a_tag_verdict_reached_without_gpg_is_asked_again_once_gpg_is_installed() {
    let c = Corpus::tagged("signature-cache-tag-no-gpg");
    c.uninstall_gpg();
    c.keyring(None);
    let (report, _) = c.check("tag-no-gpg");
    assert!(report.contains(UNCHECKED), "no gpg: unchecked\n{report}");
    assert!(faults(&report).is_empty(), "and never refused\n{report}");

    c.install_gpg();
    c.keyring(Some("good"));
    let (report, _) = c.check("tag-installed");
    assert!(
        !report.contains(UNCHECKED),
        "gpg installed: verified\n{report}"
    );
    assert!(faults(&report).is_empty(), "{report}");
}

/// gpg runs and the public key is not in the keyring: `E`, a fact about the
/// keyring, asked again once the key is imported.
#[test]
fn a_verdict_reached_without_the_public_key_is_asked_again_once_it_is_imported() {
    let c = Corpus::ratified("signature-cache-no-key");
    c.keyring(None);
    let (report, _) = c.check("no-key");
    assert!(report.contains(UNCHECKED), "no key: unchecked\n{report}");
    assert!(faults(&report).is_empty(), "{report}");

    c.keyring(Some("good"));
    let (report, _) = c.check("imported");
    assert!(
        !report.contains(UNCHECKED),
        "key imported: verified\n{report}"
    );
    assert!(faults(&report).is_empty(), "{report}");
}

/// A verdict about the commit itself is final: the second `check` reads it
/// back and starts no verifier, for each of good, bad, expired and revoked.
#[test]
fn a_verdict_about_the_commit_is_served_from_the_cache_without_a_verifier() {
    for (verdict, letter) in [
        ("good", None),
        ("bad", Some('B')),
        ("expired", Some('X')),
        ("revoked", Some('R')),
    ] {
        let c = Corpus::ratified(&format!("signature-cache-final-{verdict}"));
        c.keyring(Some(verdict));
        let expect = |report: &str| match letter {
            None => assert!(
                faults(report).is_empty() && !report.contains(UNCHECKED),
                "{verdict}: verified\n{report}"
            ),
            Some(l) => assert!(
                faults(report)
                    .iter()
                    .any(|f| f.contains(&format!("git refuses ({l})"))),
                "{verdict}: refused as {l}\n{report}"
            ),
        };
        let (report, first) = c.check(&format!("{verdict}-1"));
        expect(&report);
        assert!(first > 0, "{verdict}: the first check asks git");

        let (report, second) = c.check(&format!("{verdict}-2"));
        expect(&report);
        assert_eq!(
            second, 0,
            "{verdict}: the second check reads the cache and starts no verifier"
        );
        eprintln!("signature-cache {verdict}: verifier lines first={first} second={second}");
    }
}
