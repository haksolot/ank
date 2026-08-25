//! `ank-daemon`: it keeps declared corpora warm, and that is the whole of it
//! (ADR-a22cd3196529).
//!
//! **It answers no verb.** There is no query surface here, no socket, no
//! protocol and no subset of the CLI wearing a different name. A caller that
//! wants an answer runs `ank`, which finds a cache already warm.
//! ADR-fd98f4bc6dea settles that a second surface is a generated full
//! passthrough or it does not exist, and a daemon answering the three questions a board finds convenient
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
//! **It mirrors `refs/ank/*` and nothing else.** The one thing it writes into
//! somebody else's repository is a fetch into [`fetch::TRACKING`], a namespace
//! no verb writes and every reader of the plane skips. No branch, no tag, no
//! working tree, no index of git's, and no local `refs/ank/claims`: a
//! background process that moved any of those in a repository where somebody is
//! working would be the source of surprises a coordination tool has no business
//! being. What it buys is that `ank status` stops reporting who holds what out
//! of somebody's last manual fetch.
//!
//! **A change it notices becomes an event** (TASK-2f7777a1fdff). The line on
//! stderr is for the person reading the log; the event is for the program that
//! wanted to know, and it goes onto the stream of
//! [`ank_contract::events`] -- a file beside `watch.yml`, which any reader may
//! follow and which asks nothing of this process. An event states which corpus
//! changed and what kind of change it was, and nothing else: no title, no
//! status, no entity content of any kind, because a watcher that carried those
//! would be a source of corpus data nothing generated from `COMMANDS` ever
//! validated. What changed is here; what it means is what the CLI answers.

mod declare;
mod fail;
mod fetch;
mod stream;
mod warm;

use ank_contract::events;
use ank_contract::ExitCode;
use declare::Declaration;
use fail::{Fail, Result};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

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

It declares nothing, answers no verb and holds no claim. The only thing it
writes into a repository is that repository's own index and a mirror of
refs/ank/* under refs/ank/watch/, on the interval watch.yml states; it moves no
branch, no tag and no claim of yours. A change it sees becomes a line on
events.jsonl beside watch.yml, which says which corpus moved and what kind of
change it was, and never what to do about it. Every verb behaves the same with
it stopped, which is why stopping it is always safe.
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
        // The shape `ank --version` uses, and the same number: both come out of
        // one build, so a caller holding both reads one answer rather than two
        // it has to reconcile.
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
    let declared = declare::resolve(&text, &path, &identity_of)?;

    if opts.list {
        report(&declared, out);
        return Ok(ExitCode::Ok);
    }

    let ank = warm::locate_ank()?;
    watch(
        &declared,
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
fn report(declared: &Declaration, out: &mut dyn Write) {
    let watched = &declared.watch;
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
    // The number the reader stated, or the one they took by not stating it,
    // printed either way: an interval a listing leaves out is one a reader
    // discovers from their forge's rate limit.
    let _ = writeln!(
        out,
        "mirroring {}* every {}s",
        fetch::TRACKING,
        declared.fetch.as_secs()
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
///
/// **Two kinds of change, because there are two things to see.** The files
/// under `.ank/` moving is one; the mirror of somebody else's `refs/ank/*`
/// moving is the other, and they are not the same news -- the first is work in
/// this checkout, the second is a claim taken or released somewhere the reader
/// cannot see. Both go onto the stream under their own word, and neither says
/// what to do about it.
fn watch(declared: &Declaration, ank: &Path, once: bool, interval: Duration, err: &mut dyn Write) {
    // Resolved once, and never per event: the directory a reader declared their
    // watch in is the directory their stream belongs in, and asking the
    // environment again every half-second would let the two answer differently.
    // A home the environment does not name is not a failure -- the declaration
    // was read out of one, so this is `None` only where something removed it --
    // and a watcher that stopped for want of a stream would have made the
    // stream a condition of the thing nothing depends on.
    let stream = stream::path();
    // Flattened once, so the state and the checkout it belongs to are one
    // value. The loop below indexes nothing.
    let mut posts: Vec<Post> = Vec::new();
    for corpus in &declared.watch {
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
                mirrored: None,
                refs: None,
            });
        }
    }
    loop {
        for post in &mut posts {
            // **The mirror first, and on its own clock.** The warm poll is a
            // stat of a local directory twice a second; this is a round trip
            // against somebody's forge, so it runs on the interval the
            // declaration states and not on the one the poll uses.
            //
            // Per checkout rather than per corpus, because two roots under one
            // identity are two *clones* as readily as two worktrees, and two
            // clones are two repositories each holding its own mirror. Two
            // worktrees share a repository and so pay for one redundant fetch a
            // minute, which is the cheaper of the two ways to be wrong.
            if post
                .mirrored
                .is_none_or(|at| at.elapsed() >= declared.fetch)
            {
                if let Err(reason) = fetch::mirror(&post.root) {
                    // Reported and never fatal: a dead network is a normal
                    // Tuesday, and nothing depends on this process.
                    let _ = writeln!(
                        err,
                        "{} {}: fetch: {reason}",
                        post.identity,
                        post.root.display()
                    );
                }
                // Stamped whichever way it went. A remote that is down stays
                // down for a while, and retrying it every poll would turn one
                // failure into a spin against a network that is not answering.
                post.mirrored = Some(Instant::now());
                // What the fetch left behind, compared with what was there
                // before it. git answers zero on a remote that had nothing new,
                // so "did anything move" is a question only the refs themselves
                // answer. The first reading is a sighting and not a change, for
                // the reason the first warming is not one.
                let now = fetch::mirrored(&post.root);
                if post.refs.as_ref().is_some_and(|before| before != &now) {
                    report_change(&stream, post, events::Change::Refs, err);
                }
                post.refs = Some(now);
            }
            let now = warm::fingerprint(&warm::ank_dir(&post.root));
            let first = post.seen.is_none();
            let moved = post.seen.as_ref() != Some(&now);
            post.seen = Some(now);
            if !moved {
                continue;
            }
            if !first {
                report_change(&stream, post, events::Change::Entities, err);
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

/// Says what changed, to the person reading the log and to the program that
/// asked to be told.
///
/// Both, and in that order, because they are two audiences and not one. The
/// line on stderr names the checkout, which is what somebody debugging a
/// watcher needs; the event names the corpus and never the path, because a
/// corpus reached by two paths is one corpus (ADR-621a7fd96ce1) and a field
/// carrying a path is an invitation to key on one.
///
/// A stream that cannot be written costs a line and nothing else. Nothing
/// depends on this process, so nothing may be broken by the one part of it that
/// writes outside a repository.
fn report_change(
    stream: &Option<std::path::PathBuf>,
    post: &Post,
    change: events::Change,
    err: &mut dyn Write,
) {
    // What changed, and never what to do about it.
    let _ = writeln!(
        err,
        "changed {} {} {}",
        change.word(),
        post.identity,
        post.root.display()
    );
    let Some(path) = stream else { return };
    if let Err(reason) = stream::emit(path, &post.identity, change) {
        let _ = writeln!(err, "stream: {reason}");
    }
}

/// One checkout being watched, and what its `.ank/` looked like last time.
struct Post {
    identity: String,
    root: std::path::PathBuf,
    seen: Option<Vec<(String, u64, u128)>>,
    /// When `refs/ank/*` was last mirrored, or `None` for a checkout this
    /// process has not yet reached. `None` is due immediately, so `--once`
    /// mirrors once and a fresh start does not leave the reader a minute behind.
    mirrored: Option<Instant>,
    /// The tracking refs as the last mirror left them, or `None` for a checkout
    /// this process has not yet fetched for. A first reading is a sighting and
    /// not a change.
    refs: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use declare::Watched;
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
        let declared = Declaration {
            fetch: Duration::from_secs(60),
            watch: vec![Watched {
                identity: "a".repeat(40),
                roots: vec![PathBuf::from("/one"), PathBuf::from("/two")],
            }],
        };
        let mut out = Vec::new();
        report(&declared, &mut out);
        let text = String::from_utf8(out).unwrap();
        assert!(text.contains("watching 1 corpus, 2 checkouts"), "{text}");
        assert!(
            text.contains("mirroring refs/ank/watch/origin/* every 60s"),
            "{text}"
        );
    }
}
