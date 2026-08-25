//! `ank-daemon`: it keeps declared corpora warm, and that is the whole of it
//! (ADR-a22cd3196529).
//!
//! **It answers no verb.** There is no query surface here, no socket, no
//! protocol and no subset of the CLI wearing a different name. A caller that
//! wants an answer runs `ank`, which finds a cache already warm. ADR-372b82af1ec7
//! settled that a second surface is a generated full passthrough or it does not
//! exist, and a daemon answering the three questions a board finds convenient
//! would be the curated subset that decision refused, arrived at from the other
//! direction.
//!
//! **Nothing depends on it.** Every verb behaves identically with this process
//! absent; its absence is never an error and no route makes running it a
//! condition of using ank. That is not modesty, it is what keeps one product
//! from becoming two: the installation without a daemon is the one every CI
//! runner, every container and every agent has, so it has to be the normal one,
//! made slower rather than made lesser.
//!
//! **It watches what it was handed.** The declaration is
//! [`declare::WATCH_FILE`], keyed on the repository identity of
//! ADR-621a7fd96ce1, held outside every repository. Nothing here walks a
//! filesystem looking for a corpus, in any direction, under any flag.
//!
//! What this build does not yet do, and where it is decided: it fetches
//! nothing (TASK-a73bd7f1c8f0 carries ADR-a22cd3196529's `refs/ank/*` clause),
//! and the change it notices reaches the reader as a line on stderr rather than
//! as an event a program subscribes to (TASK-2f775b1cb32b). Both are additions
//! to this floor and neither changes what a verb answers.

mod declare;
mod fail;
mod warm;

use ank_contract::ExitCode;
use declare::Watched;
use fail::{Fail, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// How often the declared corpora are looked at, when the caller names nothing.
///
/// A poll and not a subscription, because a subscription is a third-party crate
/// and §13 spends one on necessity: what this buys over a stat of a directory
/// holding a few hundred files is latency measured in milliseconds, on a
/// process whose entire purpose is latency nobody is required to care about.
///
/// Half a second is under the time a person takes to move from one command to
/// the next, and far above the cost of the walk, so the warm case is warm and
/// the idle case is invisible.
const DEFAULT_INTERVAL: Duration = Duration::from_millis(500);

const USAGE: &str = "\
ank-daemon                keeps the corpora you declared warm, so the ank you run answers sooner

  --list                  print what would be watched, and watch nothing
  --once                  warm every declared corpus once, then exit
  --interval <ms>         how often to look (default 500)
  --where                 print the declaration file this reader's environment names
  --version               the build
  --help                  this

It declares nothing, answers no verb and holds no claim. Every verb behaves the
same with it stopped, which is why stopping it is always safe.
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    match run(&args, &mut out, &mut err) {
        Ok(code) => std::process::exit(code.code()),
        Err(fail) => {
            fail.report(&mut err);
            std::process::exit(fail.code.code());
        }
    }
}

#[derive(Debug, Default)]
struct Options {
    list: bool,
    once: bool,
    location: bool,
    interval: Option<Duration>,
}

/// **An unknown argument is refused**, and refused with the environment code
/// rather than passed along. A watcher that ignored a flag it did not know
/// would be watching under a configuration its caller does not have.
fn parse(args: &[String]) -> Result<Options> {
    let mut opts = Options::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--list" => opts.list = true,
            "--once" => opts.once = true,
            "--where" => opts.location = true,
            "--interval" => {
                let ms = it.next().ok_or_else(|| {
                    Fail::new(
                        ExitCode::Environment,
                        "--interval expects a number of milliseconds",
                    )
                })?;
                let ms: u64 = ms.parse().map_err(|_| {
                    Fail::new(
                        ExitCode::Environment,
                        format!("--interval {ms} is not a number of milliseconds"),
                    )
                })?;
                if ms == 0 {
                    return Err(Fail::new(
                        ExitCode::Environment,
                        "--interval 0 would spin rather than watch",
                    )
                    .with_hint("give it a number of milliseconds, or take the default of 500"));
                }
                opts.interval = Some(Duration::from_millis(ms));
            }
            other => {
                return Err(
                    Fail::new(ExitCode::Environment, format!("unknown argument: {other}"))
                        .with_hint("ank-daemon --help"),
                )
            }
        }
    }
    Ok(opts)
}

fn run(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> Result<ExitCode> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        let _ = write!(out, "{USAGE}");
        return Ok(ExitCode::Ok);
    }
    if args.iter().any(|a| a == "--version") {
        // The shape `ank --version` uses, and the same number: the binaries come
        // out of one build (ADR-e39a44f80e0e), so a caller holding both reads
        // one answer rather than two it has to reconcile.
        let _ = writeln!(out, "ank-daemon {}", env!("CARGO_PKG_VERSION"));
        return Ok(ExitCode::Ok);
    }
    let opts = parse(args)?;
    let path = declare::watch_path()?;
    if opts.location {
        let _ = writeln!(out, "{}", path.display());
        return Ok(ExitCode::Ok);
    }

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(Fail::new(
                ExitCode::Environment,
                format!("{} does not exist, so nothing is declared", path.display()),
            )
            .with_hint(
                "write schema: 1 and a watch: map of repository identity to the path of a \
                 checkout; ank status --json prints the identity under \"corpus\"",
            ))
        }
        Err(e) => {
            return Err(Fail::new(
                ExitCode::Environment,
                format!("{}: {e}", path.display()),
            ))
        }
    };
    let watched = declare::resolve(&text, &path, &identity_of)?;

    if opts.list {
        report(&watched, out);
        return Ok(ExitCode::Ok);
    }

    let ank = warm::locate_ank()?;
    watch(
        &watched,
        &ank,
        opts.once,
        opts.interval.unwrap_or(DEFAULT_INTERVAL),
        err,
    );
    Ok(ExitCode::Ok)
}

/// What is watched, as text and only as text.
///
/// No colour here at all, under ADR-0c8a12b4e0a1's split: the structure layer is
/// emitted identically to every reader, and this process has no half that
/// depends on who is reading -- its ordinary destination is a log file.
fn report(watched: &[Watched], out: &mut dyn Write) {
    let mut roots = 0;
    for corpus in watched {
        let _ = writeln!(out, "corpus {}", corpus.identity);
        for root in &corpus.roots {
            let _ = writeln!(out, "  {}", root.display());
            roots += 1;
        }
    }
    let _ = writeln!(
        out,
        "watching {} {}, {} {}",
        watched.len(),
        if watched.len() == 1 {
            "corpus"
        } else {
            "corpora"
        },
        roots,
        if roots == 1 { "checkout" } else { "checkouts" },
    );
}

/// The identity a corpus is keyed on: the oldest root commit of `HEAD`
/// (ADR-621a7fd96ce1).
///
/// The same question `ank status --json` answers under `"corpus"`, asked the
/// same way -- `--reverse` so a history with several roots answers with the one
/// the repository began at, and against `HEAD` rather than `--all` so fetching
/// somebody's unrelated branch cannot rename this corpus.
///
/// **git is asked here and nowhere else in this crate.** A tree with no history
/// has no identity and says so, which is a refusal at startup rather than a
/// path silently standing in for a key.
fn identity_of(root: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-list", "--max-parents=0", "--reverse", "HEAD"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .next()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// The loop: look, and read what moved.
///
/// The first pass warms every checkout unconditionally, because a daemon that
/// waited for a change before doing anything would leave the reader cold for
/// exactly as long as they were not editing -- which is most of the time, and
/// all of the time that matters after a `git checkout`. That pass is a warming
/// and not a change, so it says so: an event states what changed
/// (ADR-a22cd3196529), and a first sighting is not one.
fn watch(watched: &[Watched], ank: &Path, once: bool, interval: Duration, err: &mut dyn Write) {
    // Flattened once, so the state and the checkout it belongs to are one
    // value. The loop below indexes nothing.
    let mut posts: Vec<Post> = Vec::new();
    for corpus in watched {
        let _ = writeln!(
            err,
            "watching {} ({} {})",
            corpus.identity,
            corpus.roots.len(),
            if corpus.roots.len() == 1 {
                "checkout"
            } else {
                "checkouts"
            }
        );
        for root in &corpus.roots {
            posts.push(Post {
                identity: corpus.identity.clone(),
                root: root.clone(),
                seen: None,
            });
        }
    }
    loop {
        for post in &mut posts {
            let now = warm::fingerprint(&warm::ank_dir(&post.root));
            let first = post.seen.is_none();
            let moved = post.seen.as_ref() != Some(&now);
            post.seen = Some(now);
            if !moved {
                continue;
            }
            if !first {
                // What changed, and never what to do about it.
                let _ = writeln!(err, "changed {} {}", post.identity, post.root.display());
            }
            // Degrades and never fails: nothing depends on this process, so one
            // corpus the CLI refuses costs a line and the next poll.
            if let Err(reason) = warm::warm(ank, &post.root) {
                let _ = writeln!(err, "{} {}: {reason}", post.identity, post.root.display());
            }
        }
        if once {
            return;
        }
        std::thread::sleep(interval);
    }
}

/// One checkout being watched, and what its `.ank/` looked like last time.
struct Post {
    identity: String,
    root: std::path::PathBuf,
    seen: Option<Vec<(String, u64, u128)>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn an_unknown_argument_is_refused_with_the_environment_code() {
        let err = parse(&["--scan-everything".to_string()]).unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert!(err.message.contains("--scan-everything"), "{err:?}");
    }

    #[test]
    fn an_interval_of_zero_is_refused_rather_than_spun() {
        let err = parse(&["--interval".to_string(), "0".to_string()]).unwrap_err();
        assert!(err.message.contains("would spin"), "{err:?}");
    }

    #[test]
    fn the_listing_counts_corpora_and_checkouts() {
        let watched = vec![Watched {
            identity: "a".repeat(40),
            roots: vec![PathBuf::from("/one"), PathBuf::from("/two")],
        }];
        let mut out = Vec::new();
        report(&watched, &mut out);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("watching 1 corpus, 2 checkouts"), "{text}");
    }
}
