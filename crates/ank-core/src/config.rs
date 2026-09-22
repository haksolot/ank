//! The schema of `.ank/config.yml`, as a table (§8, ADR-2b62b9a1fe67).
//!
//! The file is read by the CLI, which owns the parser; what lives here is the
//! part a reader needs without it: every key, the type of its value and the
//! default an absent key takes. The `config-keys` binary renders [`KEYS`] into
//! `docs/config-keys.md`, and the CLI's parser is held to this table by a test
//! on its side, which reads the fields serde accepts and the defaults it falls
//! back to, so neither can move without the other.
//!
//! The defaults are declared here and the CLI reads them from here: one number
//! per default, or the page and the parser start to disagree.

/// The config schema this build reads, and the only one.
pub const SCHEMA: u32 = 1;
pub const DEFAULT_CONTEXT_BUDGET: usize = 8000;
pub const DEFAULT_CLAIM_TTL_MAX: &str = "2h";
/// What `claim` grants without `--ttl` (§3).
pub const DEFAULT_CLAIM_TTL: &str = "30m";
pub const DEFAULT_VERIFIER_TIMEOUT: &str = "10m";
/// The hot corpus a reader pays per file for, as `init` declares it
/// (ADR-467ce7e9cda1). Measured on 2026-09-14, this repository held 1979 files
/// under `.ank/entities/`; 3000 leaves it half again, which at August's rate of
/// 1645 entities a month is crossed within a month unless the cold half moves.
pub const DEFAULT_WEIGHT_HOT_FILES: u64 = 3000;
/// The bytes of claim and proof records `check` moves through its batch, as
/// `init` declares it. The same day this repository's batch was 3 285 580
/// bytes, nearly all proofs appended once per CI run; 4 MB is the next growth
/// of that mechanism, not a size a corpus reaches by accumulating facts.
pub const DEFAULT_WEIGHT_PLANE_BYTES: u64 = 4_000_000;

/// What an absent key means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Absent {
    /// The file is refused without it.
    Required,
    /// Absent is a state of its own, stated in the key's note.
    None,
    Number(u64),
    Text(&'static str),
}

/// One key, by its dotted path; `<name>` stands for a key the file chooses.
pub struct ConfigKey {
    pub path: &'static str,
    pub ty: &'static str,
    pub default: Absent,
    pub note: &'static str,
}

const fn key(
    path: &'static str,
    ty: &'static str,
    default: Absent,
    note: &'static str,
) -> ConfigKey {
    ConfigKey {
        path,
        ty,
        default,
        note,
    }
}

/// Every key `config.yml` may carry, in the order `init` writes them.
pub static KEYS: &[ConfigKey] = &[
    key(
        "schema",
        "integer",
        Absent::Required,
        "the version of this file's format",
    ),
    key(
        "context_budget",
        "integer",
        Absent::Number(DEFAULT_CONTEXT_BUDGET as u64),
        "what `context` hands a reader, in characters",
    ),
    key(
        "claim_ttl_max",
        "duration",
        Absent::Text(DEFAULT_CLAIM_TTL_MAX),
        "the longest lease a claim is granted, whatever `--ttl` asks",
    ),
    key(
        "claim_ttl_default",
        "duration",
        Absent::Text(DEFAULT_CLAIM_TTL),
        "the lease `claim` grants without `--ttl`, capped by `claim_ttl_max`",
    ),
    key(
        "default_branch",
        "string",
        Absent::None,
        "the branch carrying the reference state; absent, `refs/remotes/origin/HEAD` names it",
    ),
    key(
        "peers.<name>",
        "path",
        Absent::None,
        "a peer corpus a scope entry reaches by name, relative to this root or absolute",
    ),
    key(
        "verifiers.<name>.run",
        "command",
        Absent::Required,
        "what `done` runs through `sh`; required in a declared verifier",
    ),
    key(
        "verifiers.<name>.timeout",
        "duration",
        Absent::Text(DEFAULT_VERIFIER_TIMEOUT),
        "how long `done` lets the command run",
    ),
    key(
        "verifiers.<name>.default",
        "boolean",
        Absent::Text("false"),
        "`true` writes the verifier into every task `ank new task` creates",
    ),
    key(
        "roles.<name>.can",
        "list of strings",
        Absent::Text("[]"),
        "what the role may do, declared",
    ),
    key(
        "roles.<name>.cannot",
        "list of strings",
        Absent::Text("[]"),
        "what the role may not do, declared",
    ),
    key(
        "identities.<identity>",
        "string",
        Absent::None,
        "the role of an identity; one absent from the table is an `agent`",
    ),
    key(
        "weight.hot_files",
        "integer",
        Absent::Number(DEFAULT_WEIGHT_HOT_FILES),
        "`check` signals a hot corpus holding more entity files",
    ),
    key(
        "weight.plane_bytes",
        "integer",
        Absent::Number(DEFAULT_WEIGHT_PLANE_BYTES),
        "`check` signals claim and proof records weighing more bytes",
    ),
];
