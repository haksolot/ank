//! Running a verb, which means running **the binary** and not a copy of it.
//!
//! **The passthrough spawns `ank`.** That is the whole design and it is not
//! convenience: this repository already argues the point in its golden harness,
//! where every fixture is "captured from the process and never from a function,
//! because what §4 promises is what leaves the process". A surface that spawns
//! inherits every refusal, every exit code, every stderr warning and every
//! `--json` document by construction. A surface that linked `ank-cli` would
//! re-derive them, and anything re-derived can differ.
//!
//! So there is no dispatch here. There is an argv, built from the table, and a
//! process.

use ank_contract::json::Obj;
use ank_contract::{CommandSpec, ExitCode};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where the `ank` binary is, given that this one is its sibling.
///
/// Beside this executable first, because ADR-372b82af1ec7 says the protocol
/// surface ships from this repository as a sibling binary: the two travel
/// together, so the copy next to us is the copy we were released with. `PATH` is
/// the fallback, and `ANK_BIN` overrides both for a caller that has a reason.
///
/// Getting this wrong silently would be the worst failure available here: a
/// server answering from a different build than the one beside it would report
/// verbs the installed CLI does not have.
pub fn ank_binary() -> PathBuf {
    if let Some(explicit) = std::env::var_os("ANK_BIN") {
        return PathBuf::from(explicit);
    }
    let exe = if cfg!(windows) { "ank.exe" } else { "ank" };
    if let Ok(here) = std::env::current_exe() {
        if let Some(dir) = here.parent() {
            let beside = dir.join(exe);
            if beside.is_file() {
                return beside;
            }
        }
    }
    PathBuf::from("ank")
}

/// What one call produced.
pub struct Outcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Outcome {
    /// Whether the call refused. A refusal is a fact about the corpus, and it is
    /// reported to the client as an error result carrying the code rather than as
    /// a protocol error: the request was well formed and the answer is no.
    pub fn refused(&self) -> bool {
        self.code != ExitCode::Ok.code()
    }

    /// The MCP result. `structuredContent` carries the document the verb
    /// returned, `content` the same bytes as text for a client that reads only
    /// that, and `isError` says a refusal happened without hiding what it was.
    ///
    /// The exit code is always present, including on success, because a caller
    /// that has to branch on it should not have to tell absence from zero. `check`
    /// exits 8 with findings and that is not a failure of the call.
    pub fn to_result(&self) -> String {
        let text = match self.stdout.trim().is_empty() {
            true => self.stderr.trim(),
            false => self.stdout.trim(),
        };
        let block = Obj::new().str("type", "text").str("text", text).finish();
        let mut doc = Obj::new()
            .array("content", [block])
            .bool("isError", self.refused())
            .num("exitCode", self.code);
        if !self.stderr.trim().is_empty() {
            // Warnings live on stderr precisely so a caller's parser keeps
            // reading stdout (§4). Carried separately here for the same reason.
            doc = doc.str("stderr", self.stderr.trim());
        }
        doc.finish()
    }
}

/// The argv for a call, from the table and the client's arguments.
///
/// `--repo` and `--json` are the server's, added here and never accepted from a
/// caller: one process speaks for one corpus, and the machine document is the
/// only shape this surface can describe.
pub fn argv(spec: &CommandSpec, repo: &Path, args: &Arguments) -> Vec<String> {
    let mut out = vec![spec.name.to_string()];
    out.extend(args.positionals.iter().cloned());
    for (name, values) in &args.flags {
        for value in values {
            out.push(format!("--{name}"));
            if !value.is_empty() {
                out.push(value.clone());
            }
        }
    }
    if !spec.refuses_globals.contains(&"--repo") {
        out.push("--repo".to_string());
        out.push(repo.display().to_string());
    }
    out.push("--json".to_string());
    out
}

/// A call's arguments, already validated against the table.
#[derive(Default)]
pub struct Arguments {
    pub positionals: Vec<String>,
    /// Flag name without dashes, and its values. A switch carries one empty
    /// string, which is what makes `argv` above emit the flag and no value.
    pub flags: Vec<(String, Vec<String>)>,
}

/// Runs the verb and returns what the process said.
///
/// The identity is the server's and is typed (§3, ADR-3877d2b7c0f5). One stdio
/// server serves one client, so one process is one caller: nothing here pools
/// several clients under a single identity, which ADR-372b82af1ec7 forbids, and
/// nothing holds a claim on a client's behalf either. The claim a call takes is
/// the claim the CLI would have taken in that clone, on the same ref, arbitrated
/// by the same compare-and-swap.
pub fn run(spec: &CommandSpec, repo: &Path, args: &Arguments) -> std::io::Result<Outcome> {
    let out = Command::new(ank_binary())
        .args(argv(spec, repo, args))
        .env("ANK_AGENT", identity())
        .current_dir(repo)
        .output()?;
    Ok(Outcome {
        code: out.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    })
}

/// The typed identity this process writes under.
///
/// `$ANK_AGENT` from the environment wins, so a deployment that already names its
/// agents keeps naming them; otherwise the process names itself and its version,
/// which is the convention of §3 rather than a hostname.
pub fn identity() -> String {
    std::env::var("ANK_AGENT")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| format!("ank-mcp/{}", env!("CARGO_PKG_VERSION")))
}
