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

/// The newest config schema this build reads (ADR-756e8b0fe416): 2 is the
/// schema of a file carrying `requires_ank`, and a file without it may stay at
/// [`MIN_SCHEMA`].
pub const SCHEMA: u32 = 2;
/// The oldest config schema this build reads, which no binary ever read below.
pub const MIN_SCHEMA: u32 = 1;
/// The schema a file carrying `requires_ank` declares, and is refused below.
pub const REQUIRES_ANK_SCHEMA: u32 = 2;
pub const DEFAULT_CONTEXT_BUDGET: usize = 8000;
pub const DEFAULT_CLAIM_TTL_MAX: &str = "2h";
/// What `claim` grants without `--ttl` (§3).
pub const DEFAULT_CLAIM_TTL: &str = "30m";
pub const DEFAULT_VERIFIER_TIMEOUT: &str = "10m";
/// How old a passing run of the local registry may be and still be reused by
/// `done` (ADR-a2faf59f6ce0): long enough to span a working day, short enough
/// that a toolchain upgraded on the machine meanwhile is not trusted for long.
pub const DEFAULT_PROOF_REUSE_MAX_AGE: &str = "24h";
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
/// What the hook does with a file tool aimed under the corpus directory
/// (ADR-43cbb20dcbf1): refuses it unless the corpus says otherwise, which is
/// ADR-3c8d enforced rather than taught.
pub const DEFAULT_GUARD_CORPUS: &str = "deny";
/// The values `guard.corpus` takes, tightest first.
pub const GUARD_CORPUS_VALUES: &[&str] = &["deny", "off"];
/// What the hook does with an edit no live claim of the identity covers: says
/// so and lets it through, because refusing is the corpus's explicit choice.
pub const DEFAULT_GUARD_EDIT: &str = "warn";
/// The values `guard.edit` takes, loosest first.
pub const GUARD_EDIT_VALUES: &[&str] = &["off", "warn", "claim"];

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
        "requires_ank",
        "version",
        Absent::None,
        "`MAJOR.MINOR.PATCH`: the oldest ank this corpus needs, compared with the running binary and never with a release; a file carrying it declares `schema: 2`; absent, no requirement; moved by `ank config requires_ank`",
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
        "a peer corpus a scope entry reaches by name: a path to a checkout on disk, relative to this root or absolute, and never a URL; a reader whose layout differs overrides it in their own corpora.yml with `ank config --user peers.<identity>.<name> <path>`",
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
        "verifiers.<name>.inputs",
        "list of globs",
        Absent::None,
        "the files whose content can change the result, relative to the root; absent, the whole tree; written as one flow sequence, never empty",
    ),
    key(
        "proof.reuse_max_age",
        "duration",
        Absent::Text(DEFAULT_PROOF_REUSE_MAX_AGE),
        "how old a passing run recorded under `<git-common-dir>/ank/runs/` may be for `done` to reuse it rather than run the verifier again; `done --fresh` reuses nothing",
    ),
    key(
        "verifier_jobs",
        "integer",
        Absent::None,
        "how many verifiers one `done` runs at once, at least 1; absent, the machine's available parallelism",
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
    key(
        "guard.corpus",
        "`deny` or `off`",
        Absent::Text(DEFAULT_GUARD_CORPUS),
        "what `ank hooks run` does with a file tool aimed under the corpus directory: `deny` refuses it, `off` lets it through; a launch environment may set `ANK_GUARD_CORPUS` tighter, never looser (ADR-43cbb20dcbf1)",
    ),
    key(
        "guard.edit",
        "`off`, `warn` or `claim`",
        Absent::Text(DEFAULT_GUARD_EDIT),
        "what `ank hooks run` does with an edit no live claim of the identity covers: `off` says nothing, `warn` lets it through naming the path, `claim` refuses it; a launch environment may set `ANK_GUARD_EDIT` tighter, never looser (ADR-43cbb20dcbf1)",
    ),
    key(
        "signers.<principal>.keys[].key",
        "string",
        Absent::Required,
        "a key allowed to ratify, `gpg <fingerprint>` or `<ssh key type> <base64>`; the block is written by `ank signers`, never by `ank config` (ADR-5c6db7a161b8)",
    ),
    key(
        "signers.<principal>.keys[].note",
        "string",
        Absent::None,
        "what the key is, for the reader; `ank signers migrate` puts the comments of a former `.ank/allowed_signers` here",
    ),
    key(
        "signers.<principal>.keys[].valid_before",
        "date",
        Absent::None,
        "`YYYY-MM-DD`: the key ratifies nothing committed from that day on, UTC, and what it signed before stays verified; absent, the key has no end",
    ),
];
