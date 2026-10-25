//! Loading `.ank/config.yml` (§4, §8).
//!
//! The file is controlled by the repository, and therefore reviewed like any
//! code change — that is what makes it acceptable for `done` to run its
//! verifiers. An unknown `schema` is cleanly refused rather than misread:
//! that is the counterpart of "the format is the specification".

use crate::cli::{CliError, Invocation};
use ank_contract::ExitCode;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::time::Duration;

// The defaults are declared once, beside the table `docs/config-keys.md` is
// rendered from, and read here under the names every call site already uses.
pub const SUPPORTED_SCHEMA: u32 = ank_core::config::SCHEMA;
pub const MIN_SCHEMA: u32 = ank_core::config::MIN_SCHEMA;
pub const REQUIRES_ANK_SCHEMA: u32 = ank_core::config::REQUIRES_ANK_SCHEMA;
/// The one schema of the reader's `corpora.yml`, which moves on its own and
/// did not move with `config.yml`'s second (ADR-756e8b0fe416).
pub const CORPORA_SCHEMA: u32 = 1;
pub const DEFAULT_CONTEXT_BUDGET: usize = ank_core::config::DEFAULT_CONTEXT_BUDGET;
pub const DEFAULT_CLAIM_TTL_MAX: &str = ank_core::config::DEFAULT_CLAIM_TTL_MAX;
/// What `claim` grants without `--ttl` (§3). The same thirty minutes
/// [`crate::claim::DEFAULT_TTL`] has always been, spelled the way the file
/// spells a duration; a unit test below pins the two to one value, because two
/// spellings of one number is exactly how they start to disagree.
pub const DEFAULT_CLAIM_TTL: &str = ank_core::config::DEFAULT_CLAIM_TTL;
pub const DEFAULT_VERIFIER_TIMEOUT: &str = ank_core::config::DEFAULT_VERIFIER_TIMEOUT;
pub const DEFAULT_WEIGHT_HOT_FILES: u64 = ank_core::config::DEFAULT_WEIGHT_HOT_FILES;
pub const DEFAULT_WEIGHT_PLANE_BYTES: u64 = ank_core::config::DEFAULT_WEIGHT_PLANE_BYTES;
pub const DEFAULT_PROOF_REUSE_MAX_AGE: &str = ank_core::config::DEFAULT_PROOF_REUSE_MAX_AGE;
use ank_core::config::{
    DEFAULT_GUARD_CORPUS, DEFAULT_GUARD_EDIT, GUARD_CORPUS_VALUES, GUARD_EDIT_VALUES,
};

pub type Result<T> = std::result::Result<T, CliError>;

/// `schema:` alone, read before anything else in the file is looked at.
///
/// **The one shape in this module that tolerates an unknown key**, which is the
/// whole reason it exists. `ConfigFile` denies them, so a file one version
/// newer than this binary is reported as *unknown field `channels`* — the
/// reader is told a key is wrong when the truth is that the tool is old, and
/// goes hunting for a typo in a file that has none. `docs/format.md` already
/// legislates this for entities: "Newer is refused, and refused **on the
/// version rather than on the first field it does not recognise**." The
/// version has to be readable without reading the rest, or the rest refuses
/// first and the version never gets asked.
#[derive(Debug, Deserialize)]
struct SchemaProbe {
    schema: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    schema: u32,
    /// The oldest ank this corpus needs (ADR-756e8b0fe416). A YAML value rather
    /// than a string, because `1.2` resolves to a float and `1` to an integer,
    /// and serde's *expected a string* would name neither the key's form nor
    /// what the file wrote.
    #[serde(default)]
    requires_ank: Option<serde_yaml::Value>,
    #[serde(default = "default_budget")]
    context_budget: usize,
    #[serde(default = "default_ttl_max")]
    claim_ttl_max: String,
    #[serde(default = "default_ttl")]
    claim_ttl_default: String,
    #[serde(default)]
    default_branch: Option<String>,
    /// The corpora this repository reads, by name (§7, ADR-a1de673043b4). A key
    /// per peer rather than a list, so `ank config peers.<name> <path>` reaches
    /// exactly one declaration -- and so that adding federation adds a key
    /// instead of relaxing the `deny_unknown_fields` above.
    #[serde(default)]
    peers: BTreeMap<String, String>,
    /// Read as a mapping and not as a `BTreeMap`, because the order the file
    /// declares its verifiers in is part of what is read: `ank new task` seeds
    /// a task's `verify:` list with the marked ones in that order
    /// (ADR-443590981e41), and a sorted map has thrown it away before anyone
    /// can look. `serde_yaml::Mapping` keeps insertion order, and it is the
    /// only shape in this file that does -- everything else here is addressed
    /// by name, where order means nothing.
    #[serde(default)]
    verifiers: serde_yaml::Mapping,
    #[serde(default)]
    roles: BTreeMap<String, Role>,
    #[serde(default)]
    identities: BTreeMap<String, String>,
    #[serde(default)]
    weight: Weight,
    /// `proof:`, how `done` treats what it already ran (ADR-a2faf59f6ce0).
    #[serde(default)]
    proof: ProofFile,
    /// How many verifiers of one `done` run at once. Absent, the machine's
    /// available parallelism: the bound exists for several agents sharing one
    /// machine, and only somebody who knows there are several can set it.
    #[serde(default)]
    verifier_jobs: Option<usize>,
    /// `guard:`, what `ank hooks run` applies before a tool call
    /// (ADR-43cbb20dcbf1).
    #[serde(default)]
    guard: GuardFile,
    /// Who may ratify, keyed by principal (ADR-5c6db7a161b8). A mapping rather
    /// than a `BTreeMap` for the reason `verifiers` is one: the order the file
    /// declares them in is the order `review` lists them in.
    #[serde(default)]
    signers: serde_yaml::Mapping,
}

/// One principal under `signers:`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrincipalFile {
    #[serde(default)]
    keys: Vec<SignerKeyFile>,
}

/// One key of a principal, as written.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SignerKeyFile {
    key: String,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    valid_before: Option<String>,
}

/// One key allowed to ratify (§8, ADR-5c6db7a161b8): a principal, the key in
/// git's `<keytype> <key>` form, and the date after which it ratifies nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signer {
    pub principal: String,
    /// Lowercased: `gpg`, or an ssh key type.
    pub keytype: String,
    pub key: String,
    pub note: Option<String>,
    /// `YYYY-MM-DD`, read as midnight UTC. A commit dated at or after it is
    /// not a ratification this key can make.
    pub valid_before: Option<String>,
}

/// Whether a field names a key type: every ssh type is `ssh-`, `ecdsa-`,
/// `sk-` or `webauthn-`, certificate forms included, and `gpg` is ank's own
/// extension (§8) and the one exact match.
pub fn is_keytype(field: &str) -> bool {
    let lower = field.to_ascii_lowercase();
    lower == "gpg"
        || ["ssh-", "ecdsa-", "sk-", "webauthn-"]
            .iter()
            .any(|p| lower.starts_with(p))
}

/// `YYYY-MM-DD` as days since the epoch, or `None` when it is not a date.
pub fn date_days(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i64 = text[..4].parse().ok()?;
    let m: u32 = text[5..7].parse().ok()?;
    let d: u32 = text[8..].parse().ok()?;
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days_in = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if d == 0 || d > days_in {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = i64::from((m + 9) % 12);
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

/// Reads `signers:` into one row per key, in the order the file declares them.
fn parse_signers(map: serde_yaml::Mapping, path: &Path) -> Result<Vec<Signer>> {
    let refuse =
        |what: String| CliError::new(ExitCode::Generic, format!("{}: {what}", path.display()));
    let mut out = Vec::new();
    for (principal, value) in map {
        let Some(principal) = principal.as_str().map(str::to_string) else {
            return Err(refuse("a signer is named by a string".to_string()));
        };
        let p: PrincipalFile = serde_yaml::from_value(value)
            .map_err(|e| refuse(format!("signers.{principal}: {e}")))?;
        for k in p.keys {
            let fields: Vec<&str> = k.key.split_whitespace().collect();
            let [keytype, key] = fields.as_slice() else {
                return Err(refuse(format!(
                    "signers.{principal}: '{}' is not '<keytype> <key>'",
                    k.key
                )));
            };
            if !is_keytype(keytype) {
                return Err(refuse(format!(
                    "signers.{principal}: '{keytype}' is not a key type (gpg, ssh-..., ecdsa-..., sk-...)"
                )));
            }
            let valid_before = k.valid_before.map(|d| d.trim().to_string());
            if let Some(d) = &valid_before {
                if date_days(d).is_none() {
                    return Err(refuse(format!(
                        "signers.{principal}: valid_before '{d}' is not a YYYY-MM-DD date"
                    )));
                }
            }
            out.push(Signer {
                principal: principal.clone(),
                keytype: keytype.to_ascii_lowercase(),
                key: (*key).to_string(),
                note: k.note,
                valid_before,
            });
        }
    }
    Ok(out)
}

/// `weight:`, what the corpus declares it may grow to before `check` says so
/// (ADR-467ce7e9cda1). A counter left out takes its default, exactly as an
/// absent `context_budget` does, so a corpus that predates the key is still
/// weighed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weight {
    #[serde(default = "default_hot_files")]
    pub hot_files: u64,
    #[serde(default = "default_plane_bytes")]
    pub plane_bytes: u64,
}

impl Default for Weight {
    fn default() -> Weight {
        Weight {
            hot_files: DEFAULT_WEIGHT_HOT_FILES,
            plane_bytes: DEFAULT_WEIGHT_PLANE_BYTES,
        }
    }
}

/// `proof:` as written. A mapping of its own rather than a top-level key, so
/// the next setting about proofs lands beside this one.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProofFile {
    #[serde(default = "default_reuse_max_age")]
    reuse_max_age: String,
}

impl Default for ProofFile {
    fn default() -> ProofFile {
        ProofFile {
            reuse_max_age: default_reuse_max_age(),
        }
    }
}

/// `guard:` as written. Read as strings and resolved by [`parse`], so a value
/// outside the closed set is refused in the sentence [`not_a_guard`] gives
/// rather than in serde's.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct GuardFile {
    #[serde(default = "default_guard_corpus")]
    corpus: String,
    #[serde(default = "default_guard_edit")]
    edit: String,
}

impl Default for GuardFile {
    fn default() -> GuardFile {
        GuardFile {
            corpus: default_guard_corpus(),
            edit: default_guard_edit(),
        }
    }
}

fn default_guard_corpus() -> String {
    DEFAULT_GUARD_CORPUS.to_string()
}

fn default_guard_edit() -> String {
    DEFAULT_GUARD_EDIT.to_string()
}

/// What a file tool aimed under the corpus directory meets (ADR-43cbb20dcbf1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardCorpus {
    Off,
    Deny,
}

/// What an edit no live claim of the identity covers meets
/// (ADR-43cbb20dcbf1). Ordered loosest first, so a launch environment can
/// only tighten it by taking the greater of two levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GuardEdit {
    Off,
    Warn,
    Claim,
}

impl GuardCorpus {
    pub fn parse(text: &str) -> Option<GuardCorpus> {
        match text {
            "deny" => Some(GuardCorpus::Deny),
            "off" => Some(GuardCorpus::Off),
            _ => None,
        }
    }
}

impl GuardEdit {
    pub fn parse(text: &str) -> Option<GuardEdit> {
        match text {
            "off" => Some(GuardEdit::Off),
            "warn" => Some(GuardEdit::Warn),
            "claim" => Some(GuardEdit::Claim),
            _ => None,
        }
    }
}

/// The two guards the corpus declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Guard {
    pub corpus: GuardCorpus,
    pub edit: GuardEdit,
}

/// The refusal a guard value outside its closed set earns, naming the set.
fn not_a_guard(key: &str, written: &str) -> CliError {
    let accepted = if key == "guard.corpus" {
        GUARD_CORPUS_VALUES
    } else {
        GUARD_EDIT_VALUES
    };
    CliError::new(
        ExitCode::Generic,
        format!(
            "{key}: '{written}' is not a level; accepted: {}",
            accepted.join(", ")
        ),
    )
    .with_hint(format!("ank config {key} {}", accepted.join("|")))
}

fn default_reuse_max_age() -> String {
    DEFAULT_PROOF_REUSE_MAX_AGE.to_string()
}

fn default_hot_files() -> u64 {
    DEFAULT_WEIGHT_HOT_FILES
}

fn default_plane_bytes() -> u64 {
    DEFAULT_WEIGHT_PLANE_BYTES
}

fn default_budget() -> usize {
    DEFAULT_CONTEXT_BUDGET
}

fn default_ttl_max() -> String {
    DEFAULT_CLAIM_TTL_MAX.to_string()
}

fn default_ttl() -> String {
    DEFAULT_CLAIM_TTL.to_string()
}

fn default_timeout() -> String {
    DEFAULT_VERIFIER_TIMEOUT.to_string()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifierFile {
    run: String,
    #[serde(default = "default_timeout")]
    timeout: String,
    /// Whether `ank new task` writes this verifier into the task it creates
    /// (ADR-443590981e41). Absent means no, and absent is how the key stays
    /// until somebody marks the verifier: a mark written out for every
    /// verifier would turn "nobody chose" into "chosen against".
    ///
    /// Not carried onto [`Verifier`], and deliberately: the mark decides what
    /// a task is born with, never what running one proves, and
    /// `verify::definition_hash` anchors historical proofs over exactly the
    /// fields that decide the second.
    #[serde(default)]
    default: bool,
    /// Globs relative to the repository root, the files whose content can
    /// change the result (ADR-d8a328e311b9). Absent, the whole tree is the
    /// input; an empty list is refused, because it would say nothing can.
    #[serde(default)]
    inputs: Option<Vec<String>>,
    // No `env`: ADR-7224dea611c1 withdrew it until an executor exists to
    // honour it, and `deny_unknown_fields` refuses it by name like any key
    // the parser does not know.
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Role {
    #[serde(default)]
    pub can: Vec<String>,
    #[serde(default)]
    pub cannot: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verifier {
    pub run: String,
    pub timeout: Duration,
    /// As declared; `None` is the whole tree, never an empty list.
    pub inputs: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub schema: u32,
    /// The oldest ank this corpus needs, `None` where the file declares no
    /// requirement (ADR-756e8b0fe416).
    pub requires_ank: Option<crate::update::Version>,
    pub context_budget: usize,
    pub claim_ttl_max: Duration,
    /// What a claim is granted when `--ttl` says nothing (§3). Capped by
    /// [`Config::claim_ttl_max`] at claim time and not here: the file may state
    /// a default above the cap, and the refusal §3 wants is the cap binding
    /// what is granted, not a configuration that will not load.
    pub claim_ttl_default: Duration,
    /// Branch carrying the reference durable state (§7). Optional: absent, the
    /// resolution falls back to `refs/remotes/origin/HEAD`, and fails rather
    /// than guessing if that is absent too — see
    /// [`crate::git::resolve_default_branch`].
    pub default_branch: Option<String>,
    /// Peer corpora, by the name a scope entry uses to reach them (§7). The
    /// value is the path to the peer's root, resolved against this repository's
    /// root when it is relative — never discovered, never inferred from a
    /// remote, because inference is how a corpus starts depending on where
    /// somebody checked something out.
    pub peers: BTreeMap<String, String>,
    pub verifiers: BTreeMap<String, Verifier>,
    /// The verifiers marked `default: true`, in the order `config.yml`
    /// declares them (ADR-443590981e41). This is what `ank new task` seeds a
    /// task's `verify:` list with when the caller names none.
    ///
    /// A list beside the map rather than a flag inside it, because the answer
    /// this field gives is ordered and the map is not: a reader assembling it
    /// from `verifiers` would get the alphabet back, which is a different
    /// sentence from the one the file wrote.
    pub default_verifiers: Vec<String>,
    pub roles: BTreeMap<String, Role>,
    pub identities: BTreeMap<String, String>,
    pub weight: Weight,
    /// The most verifiers one `done` runs at once, never zero. `None` is the
    /// key absent, resolved at the moment of running by
    /// [`Config::verifier_jobs`], because what it resolves to is the machine
    /// and not the file.
    pub verifier_jobs: Option<usize>,
    /// What `ank hooks run` applies before a tool call (ADR-43cbb20dcbf1).
    pub guard: Guard,
    /// How old a passing run of the local registry may be for `done` to
    /// reuse it, `proof.reuse_max_age` (ADR-a2faf59f6ce0).
    pub reuse_max_age: Duration,
    /// Who may ratify, one row per key (ADR-5c6db7a161b8).
    pub signers: Vec<Signer>,
}

impl Config {
    /// What bounds a `done`: the key when the file sets it, the machine's
    /// available parallelism when it does not, and one when even that cannot
    /// be read.
    pub fn verifier_jobs(&self) -> usize {
        self.verifier_jobs.unwrap_or_else(|| {
            std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get)
        })
    }

    pub fn verifier(&self, name: &str) -> Option<&Verifier> {
        self.verifiers.get(name)
    }
}

/// Durations of the form `<n><unit>`, units `s`, `m`, `h`, `d`. Deliberately
/// narrow: `claim_ttl_max: 2h` must read without documentation, and a richer
/// grammar would only add forms nobody should write.
pub fn parse_duration(text: &str) -> std::result::Result<Duration, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("empty duration".to_string());
    }
    let (digits, unit) = t.split_at(
        t.find(|c: char| !c.is_ascii_digit())
            .unwrap_or_else(|| t.len()),
    );
    if digits.is_empty() {
        return Err(format!("duration '{text}': digit expected before the unit"));
    }
    let n: u64 = digits
        .parse()
        .map_err(|_| format!("duration '{text}': unreadable number"))?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        "" => return Err(format!("duration '{text}': missing unit (s, m, h, d)")),
        other => {
            return Err(format!(
                "duration '{text}': unknown unit '{other}' (s, m, h, d)"
            ))
        }
    };
    Ok(Duration::from_secs(secs))
}

pub fn parse(text: &str, path: &Path) -> Result<Config> {
    // **The version first, and the fields afterwards.** A file newer than this
    // binary is refused on its version, so the sentence a reader gets names
    // the cause — this tool is old — instead of the first key that happens to
    // be new. The probe is only ever allowed to *add* a refusal: where it
    // cannot answer, it says nothing and the full parse below produces exactly
    // the diagnosis it always produced, missing `schema` and a file that is
    // not a mapping included.
    if let Some(found) = serde_yaml::from_str::<SchemaProbe>(text)
        .ok()
        .map(|p| p.schema)
    {
        if found > SUPPORTED_SCHEMA {
            return Err(CliError::new(
                ExitCode::Generic,
                format!(
                    "{}: schema {found}, this binary reads up to {SUPPORTED_SCHEMA}: \
                     the file is newer than this ank",
                    path.display()
                ),
            )
            .with_hint("ank --version names the build, npm install -g @haksolot/ank replaces it"));
        }
    }

    let raw: ConfigFile = serde_yaml::from_str(text)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;

    // Below the range rather than above it: no binary ever read schema 0, so
    // there is no "older tool" to name and the file is simply wrong.
    if !(MIN_SCHEMA..=SUPPORTED_SCHEMA).contains(&raw.schema) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "{}: unknown schema {} (supported: {MIN_SCHEMA} to {SUPPORTED_SCHEMA})",
                path.display(),
                raw.schema
            ),
        )
        .with_hint("update ank, or fix the schema field"));
    }

    let requires_ank = match &raw.requires_ank {
        None => None,
        Some(value) => Some(requires_ank(value, path)?),
    };
    // The key declares the schema it arrived with, so that a binary predating
    // it refuses the file on its version rather than on a key it calls a typo.
    // A file at 1 that carries it was written by hand, and is told both moves.
    if requires_ank.is_some() && raw.schema < REQUIRES_ANK_SCHEMA {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "{}: requires_ank needs schema {REQUIRES_ANK_SCHEMA}, and the file declares schema {}",
                path.display(),
                raw.schema
            ),
        )
        .with_hint(format!(
            "ank config schema {REQUIRES_ANK_SCHEMA}, or ank config --unset requires_ank"
        )));
    }

    let dur = |v: &str, field: &str| -> Result<Duration> {
        parse_duration(v).map_err(|e| {
            CliError::new(
                ExitCode::Generic,
                format!("{}: {field}: {e}", path.display()),
            )
        })
    };

    // Zero would be a `done` that runs nothing and waits forever, which no
    // reader means by writing it.
    if raw.verifier_jobs == Some(0) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "{}: verifier_jobs: 0 runs no verifier, a positive integer is required",
                path.display()
            ),
        ));
    }

    let claim_ttl_max = dur(&raw.claim_ttl_max, "claim_ttl_max")?;
    let claim_ttl_default = dur(&raw.claim_ttl_default, "claim_ttl_default")?;
    let reuse_max_age = dur(&raw.proof.reuse_max_age, "proof.reuse_max_age")?;
    let signers = parse_signers(raw.signers, path)?;
    let in_file = |e: CliError| {
        CliError::new(
            ExitCode::Generic,
            format!("{}: {}", path.display(), e.message),
        )
    };
    let guard = Guard {
        corpus: GuardCorpus::parse(&raw.guard.corpus)
            .ok_or_else(|| in_file(not_a_guard("guard.corpus", &raw.guard.corpus)))?,
        edit: GuardEdit::parse(&raw.guard.edit)
            .ok_or_else(|| in_file(not_a_guard("guard.edit", &raw.guard.edit)))?,
    };
    let mut verifiers = BTreeMap::new();
    let mut default_verifiers = Vec::new();
    for (name, value) in raw.verifiers {
        // A name is what `--verify` types and what a task's `verify:` list
        // holds, so a key YAML resolved to something else -- `on:`, `1:` --
        // names a verifier nothing can ask for.
        let Some(name) = name.as_str().map(str::to_string) else {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("{}: a verifier is named by a string", path.display()),
            ));
        };
        let v: VerifierFile = serde_yaml::from_value(value).map_err(|e| {
            CliError::new(
                ExitCode::Generic,
                format!("{}: verifiers.{name}: {e}", path.display()),
            )
        })?;
        let timeout = dur(&v.timeout, &format!("verifiers.{name}.timeout"))?;
        if v.default {
            default_verifiers.push(name.clone());
        }
        let field = |f: &str| format!("{}: verifiers.{name}.{f}", path.display());
        if let Some(inputs) = &v.inputs {
            if inputs.is_empty() {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!(
                        "{}: an empty list says no file can change the result; \
                         leave the key out and the whole tree is the input",
                        field("inputs")
                    ),
                )
                .with_hint(format!("ank config --unset verifiers.{name}.inputs")));
            }
            for glob in inputs {
                if glob.trim().is_empty()
                    || ank_core::scope::validate_globs(&[glob.clone()]).is_err()
                {
                    return Err(CliError::new(
                        ExitCode::Generic,
                        format!("{}: '{glob}' is not a glob", field("inputs")),
                    ));
                }
            }
        }
        verifiers.insert(
            name,
            Verifier {
                run: v.run,
                timeout,
                inputs: v.inputs,
            },
        );
    }

    // A key present but blank names no branch: it is an absence written out,
    // not a branch called "".
    let default_branch = raw
        .default_branch
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty());

    Ok(Config {
        schema: raw.schema,
        requires_ank,
        context_budget: raw.context_budget,
        claim_ttl_max,
        claim_ttl_default,
        default_branch,
        // A declaration whose path is blank names no corpus. Kept rather than
        // dropped: resolution warns about it by name, and a peer silently
        // removed at parse time is a declaration nobody can see failing.
        peers: raw
            .peers
            .into_iter()
            .map(|(name, path)| (name, path.trim().to_string()))
            .collect(),
        verifiers,
        default_verifiers,
        roles: raw.roles,
        identities: raw.identities,
        weight: raw.weight,
        verifier_jobs: raw.verifier_jobs,
        guard,
        reuse_max_age,
        signers,
    })
}

/// The refusal a `requires_ank` not of the form `MAJOR.MINOR.PATCH` earns.
fn not_a_version(written: &str, path: &Path) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!(
            "{}: requires_ank: '{written}' is not a version of the form MAJOR.MINOR.PATCH",
            path.display()
        ),
    )
    .with_hint("ank config requires_ank <MAJOR.MINOR.PATCH>, three numbers and nothing else")
}

/// `requires_ank` as the file wrote it, read by the rule `ank update` applies
/// to a release tag without its `v`: three runs of digits and nothing else, so
/// `1.2`, `v1.2.3` and `1.2.3-rc1` are each refused rather than guessed at.
fn requires_ank(value: &serde_yaml::Value, path: &Path) -> Result<crate::update::Version> {
    let written = match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_string(),
    };
    crate::update::Version::parse(&written).ok_or_else(|| not_a_version(&written, path))
}

pub fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::new(ExitCode::Generic, format!("{} not found", path.display()))
                .with_hint("ank init")
        } else {
            CliError::new(ExitCode::Generic, format!("{}: {e}", path.display()))
        }
    })?;
    parse(&text, path)
}

/// Content written by `ank init`. The canonical form of the file.
///
/// `default_branch` is deliberately absent: `init` runs where the reference
/// branch is not known yet, and writing `main` here would be exactly the guess
/// §7 refuses. Detection through `refs/remotes/origin/HEAD` covers the case,
/// and the error names the key to add when it does not.
pub fn default_yaml() -> String {
    // The weight is written from the constants and not typed a second time: a
    // default `init` writes and a default the parser falls back to are one
    // number, or a corpus born today and one that predates the key disagree.
    format!(
        "\
schema: 1
context_budget: 8000
claim_ttl_max: 2h
verifiers: {{}}
roles:
  agent:
    can: [context, find, claim, log, done, new:task, new:adr:proposed]
    cannot: [adr:accept, adr:edit-constraint, task:close, delete]
  human:
    can: [\"*\"]
identities: {{}}
weight:
  hot_files: {DEFAULT_WEIGHT_HOT_FILES}
  plane_bytes: {DEFAULT_WEIGHT_PLANE_BYTES}
"
    )
}

// ---------------------------------------------------------------------------
// The reader's declarations, outside every repository (ADR-96174f1ac2b7)
// ---------------------------------------------------------------------------
//
// **Outside the repository is the whole promise.** A pointer committed in the
// code repository would be the more useful design for a team wanting shared
// context, and it is precisely the trace the request refuses. So this file is
// the reader's, one entry per repository identity, and the code repository
// keeps not one byte it did not already have.
//
// **Declared, never discovered.** Nothing here walks a filesystem looking for a
// corpus and nothing reads a git remote, for the reason ADR-a1de673043b4 gives
// about peers: inference is how a corpus starts depending on where somebody
// happened to check something out.

/// The reader's map of repository identity to corpus, under [`user_dir`].
pub const CORPORA_FILE: &str = "corpora.yml";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorporaFile {
    schema: u32,
    #[serde(default)]
    corpora: BTreeMap<String, String>,
    /// The reader's overrides of peer paths (ADR-da2819aef598), keyed
    /// `<identity>.<name>`: the declaring corpus's identity and the name its
    /// `config.yml` gives the peer. Flat rather than nested so that the key the
    /// verb takes, `peers.<identity>.<name>`, is the path the file spells, and
    /// one surgery writes both maps.
    #[serde(default)]
    peers: BTreeMap<String, String>,
}

/// The refusal a `corpora.yml` newer than this binary earns, or `None` where
/// there is none to give.
///
/// **The version first, and the fields afterwards**, which is the rule
/// `parse()` applies to `.ank/config.yml` and `docs/format.md` states for
/// entities: "Newer is refused, and refused **on the version rather than on the
/// first field it does not recognise**." `CorporaFile` denies unknown fields
/// and compares `schema` only after the deserialize has already succeeded, so a
/// file one version ahead was reported as *unknown field `mirrors`* and its
/// reader went looking for a typo in a file that had none.
///
/// Reused rather than inlined because this file is read at two call sites that
/// answer with different exit codes -- [`declarations`] is resolving the
/// reader's environment and [`parse_corpora`] is guarding a write -- and one
/// sentence at both is the whole point. Like the probe in `parse()`, it may
/// only ever *add* a refusal: where it cannot read a version it says nothing
/// and the caller's own parse produces the diagnosis it always produced.
fn newer_than_this_ank(text: &str, path: &Path, code: ExitCode) -> Option<CliError> {
    let found = serde_yaml::from_str::<SchemaProbe>(text)
        .ok()
        .map(|p| p.schema)?;
    if found <= CORPORA_SCHEMA {
        return None;
    }
    Some(
        CliError::new(
            code,
            format!(
                "{}: schema {found}, this binary reads {CORPORA_SCHEMA}: \
                 the file is newer than this ank",
                path.display()
            ),
        )
        .with_hint("ank --version names the build, npm install -g @haksolot/ank replaces it"),
    )
}

/// Where this reader's declarations live, or `None` where the environment names
/// no home.
///
/// **The environment and the standard library, and no third-party crate.** The
/// rule is three lines of platform difference and a dependency for it would be
/// a supply chain for a `getenv`.
///
/// `%APPDATA%\ank` on Windows; `$XDG_CONFIG_HOME/ank` elsewhere, falling back to
/// `$HOME/.config/ank`, which is the same order every other tool on those two
/// platforms applies. An empty value counts as unset: a shell that exports a
/// variable to nothing has said nothing, and joining onto it would name a
/// relative path under the current directory.
pub fn user_dir() -> Option<std::path::PathBuf> {
    let var = |key: &str| {
        std::env::var_os(key)
            .filter(|v| !v.is_empty())
            .map(std::path::PathBuf::from)
    };
    if cfg!(windows) {
        return var(ank_contract::env::APPDATA).map(|p| p.join("ank"));
    }
    if let Some(xdg) = var(ank_contract::env::XDG_CONFIG_HOME) {
        return Some(xdg.join("ank"));
    }
    var(ank_contract::env::HOME).map(|p| p.join(".config").join("ank"))
}

/// The declarations file, wherever this reader's home is.
pub fn corpora_path() -> Option<std::path::PathBuf> {
    user_dir().map(|d| d.join(CORPORA_FILE))
}

/// A repository identity as ADR-621a7fd96ce1 defines it: the root commit, and
/// therefore forty lowercase hex characters.
fn is_identity(key: &str) -> bool {
    key.len() == 40
        && key
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Every corpus this reader has declared, keyed on repository identity.
///
/// **An empty map for a file that is not there**, which is the ordinary case
/// and not a failure: a reader who has declared nothing gets the walk, and the
/// walk is what every corpus written before this existed resolves through.
///
/// **A key that is not an identity is refused by name**, and that is behaviour
/// rather than commentary. The request this comes from proposed a directory
/// named by the slugified remote URL, so this map will be handed slugs, paths
/// and URLs by people who read that thread. Each of them fails the test
/// ADR-96174f1ac2b7 states -- one clone over ssh and one over https key
/// differently, a fork keys differently from its upstream, a rename on the
/// forge silently orphans the declaration -- and a wrong key that merely
/// matched nothing would leave the corpus quietly not found, which is the one
/// outcome worse than a refusal.
pub fn declarations() -> Result<BTreeMap<String, String>> {
    Ok(corpora_file()?.map(|f| f.corpora).unwrap_or_default())
}

/// The path this reader gave, in place of what `config.yml` declares, for the
/// peer `name` of the corpus `identity` (ADR-da2819aef598), or `None` where
/// there is no override.
///
/// **Silent on a file that cannot be read.** Every verb resolves its corpus
/// through [`declarations`] before it reads a peer, so a file that does not
/// parse has already been refused by the time this is asked.
pub fn peer_override(identity: &str, name: &str) -> Option<String> {
    corpora_file()
        .ok()
        .flatten()?
        .peers
        .remove(&format!("{identity}.{name}"))
}

/// Whether this reader overrides any peer path at all, which is what spares a
/// reader who has overridden nothing the git process an identity costs.
pub fn has_peer_overrides() -> bool {
    corpora_file()
        .ok()
        .flatten()
        .is_some_and(|f| !f.peers.is_empty())
}

/// A key under `peers` as ADR-da2819aef598 defines it: an identity, a dot, and
/// a name a scope entry could spell.
fn is_override_key(key: &str) -> bool {
    key.split_once('.')
        .is_some_and(|(id, name)| is_identity(id) && crate::repo::is_peer_name(name))
}

/// The reader's declarations file, parsed and checked, or `None` where there is
/// none.
fn corpora_file() -> Result<Option<CorporaFile>> {
    let Some(path) = corpora_path() else {
        return Ok(None);
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(CliError::new(
                ExitCode::Environment,
                format!("{}: {e}", path.display()),
            ))
        }
    };
    if let Some(refusal) = newer_than_this_ank(&text, &path, ExitCode::Environment) {
        return Err(refusal);
    }
    let file: CorporaFile = serde_yaml::from_str(&text).map_err(|e| {
        CliError::new(ExitCode::Environment, format!("{}: {e}", path.display())).with_hint(
            "schema: 1, a corpora: map of repository identity to path, \
                 and a peers: map of <identity>.<name> to path",
        )
    })?;
    // Below the range rather than above it, exactly as `parse()` splits them:
    // no binary ever read schema 0, so there is no older tool to name.
    if file.schema != CORPORA_SCHEMA {
        return Err(CliError::new(
            ExitCode::Environment,
            format!(
                "{}: schema {} is not {CORPORA_SCHEMA}",
                path.display(),
                file.schema
            ),
        ));
    }
    for key in file.corpora.keys() {
        if !is_identity(key) {
            return Err(CliError::new(
                ExitCode::Environment,
                format!("{}: '{key}' is not a repository identity", path.display()),
            )
            .with_hint(
                "a key is the root commit, never a path, a remote or a slug: \
                 ank status --json prints it under \"corpus\"",
            ));
        }
    }
    for key in file.peers.keys() {
        if !is_override_key(key) {
            return Err(CliError::new(
                ExitCode::Environment,
                format!(
                    "{}: '{key}' under peers is not <identity>.<name>",
                    path.display()
                ),
            )
            .with_hint("ank config --user peers.<identity>.<name> <path> writes one"));
        }
    }
    Ok(Some(file))
}

// ---------------------------------------------------------------------------
// `ank config` (§4, ADR-e64dfaafd578)
// ---------------------------------------------------------------------------
//
// Text surgery, and never a round-trip through a serializer. `Config` derives
// `Deserialize` and nothing else, and building the other half would be the
// wrong move rather than a shortcut avoided: a parse-mutate-serialize writer
// drops every comment and blank line, returns `verifiers`, `roles` and
// `identities` alphabetised because they are `BTreeMap`s, and -- the reason
// this is not a matter of taste -- writes out every field carrying a serde
// default. An unset key means "follows the tool"; a written one means "pinned
// here". So a round-trip turns absence into assertion, and every repository
// that ever ran one `ank config` silently holds the old value the day a default
// moves.
//
// The writer therefore finds the line, replaces the span the value occupies,
// and leaves every other byte alone.

/// The keys this verb addresses, spelled the way §4's table spells them.
///
/// **The contract's own set, and no longer a literal here**
/// (TASK-b08d090f699c). It was written out below and compared with `config`'s
/// note by a test, which held two renderings in step; the terminal reader is a
/// third consumer and may not link this crate (ADR-8bd76e8d7c4e), so the set
/// moved to `ank-contract` -- where the note is rendered from it too. One
/// table, three surfaces, and the test that compared the first two is now
/// measuring a rendering rather than arbitrating between two memories.
pub const KEYS: &[&str] = ank_contract::verbs::CONFIG_KEYS;

/// A dotted path, resolved against the closed key set.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    /// A scalar at the root of the document.
    Top {
        name: &'static str,
        /// Written without quotes: the field holds a number or a boolean, and
        /// quoting one would store a string the parser refuses.
        verbatim: bool,
        default: Option<String>,
    },
    /// `verifiers.<name>.run`, `.timeout` or `.default`.
    Field {
        verifier: String,
        field: &'static str,
        /// The same rule as the one above, and `default` is why it reaches
        /// here too: `default: "true"` is a string, and the parser wants the
        /// boolean the caller typed.
        verbatim: bool,
        default: Option<String>,
    },
    /// `verifiers.<name>`: legal for `--unset` alone, which is what makes
    /// declaring a verifier reversible.
    Block { verifier: String },
    /// `verifiers.<name>.inputs` (ADR-d8a328e311b9): a list, typed and read
    /// back as one YAML flow sequence and stored with every glob double-quoted,
    /// because a glob opening on `*` written plain is a YAML alias.
    Inputs { verifier: String },
    /// One scalar under one mapping, which is the shape of a declaration: a
    /// peer corpus under `peers` (§7), and a corpus declared for a repository
    /// identity under `corpora` (ADR-96174f1ac2b7). One level shallower than a
    /// verifier, so the whole key is the declaration and `--unset` on it
    /// removes it outright.
    ///
    /// **The mapping is a field and not a literal in the surgery**, because
    /// there are two of them now and the second arrived in a different file.
    /// One shape, one implementation, and the name it is written under is what
    /// the caller supplies.
    ///
    /// `weight` is the third, and the first whose values are numbers: written
    /// verbatim for the reason [`Key::Top`] gives, and resolved to the default
    /// the parser falls back to when absent.
    Under {
        map: &'static str,
        name: String,
        verbatim: bool,
        default: Option<String>,
    },
}

fn unknown_key(path: &str) -> CliError {
    CliError::new(ExitCode::Generic, format!("unknown key '{path}'"))
        .with_hint(format!("keys: {}", KEYS.join(" ")))
}

fn structured(name: &str) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("'{name}' has a structured value, which ank config does not address"),
    )
    .with_hint(format!(
        "edit .ank/config.yml by hand: {name} is a mapping of lists, not a scalar"
    ))
}

/// A `run` written as a block or folded scalar. Refused rather than rewritten:
/// its resolved string depends on line structure a one-line replacement would
/// flatten, and `verify::definition_hash` is taken over the resolved value, so
/// flattening it moves a hash that anchors historical proofs.
fn blocky(name: &str) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("'{name}' is a block or folded scalar, which ank config does not rewrite"),
    )
    .with_hint(
        "edit .ank/config.yml by hand: flattening it would move the verifier's definition hash",
    )
}

fn flow_mapping(name: &str) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("'{name}' is written as a non-empty flow mapping, which ank config does not edit"),
    )
    .with_hint("edit .ank/config.yml by hand, or write it as a block mapping")
}

fn undeclared(verifier: &str) -> CliError {
    CliError::new(
        ExitCode::Prerequisite,
        format!("verifier '{verifier}' is not declared"),
    )
    .with_hint(format!("ank config verifiers.{verifier}.run \"<command>\""))
}

fn whole_block(verifier: &str) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("'verifiers.{verifier}' is a whole verifier, not a value: --unset removes it"),
    )
    .with_hint(format!("ank config verifiers.{verifier}.run"))
}

fn resolve_key(path: &str) -> Result<Key> {
    let segs: Vec<&str> = path.split('.').collect();
    if segs.iter().any(|s| s.is_empty()) {
        return Err(unknown_key(path));
    }
    match segs.as_slice() {
        ["schema"] => Ok(Key::Top {
            name: "schema",
            verbatim: true,
            default: None,
        }),
        // No default: absent, the corpus requires no particular ank
        // (ADR-756e8b0fe416).
        ["requires_ank"] => Ok(Key::Top {
            name: "requires_ank",
            verbatim: false,
            default: None,
        }),
        ["context_budget"] => Ok(Key::Top {
            name: "context_budget",
            verbatim: true,
            default: Some(DEFAULT_CONTEXT_BUDGET.to_string()),
        }),
        ["claim_ttl_max"] => Ok(Key::Top {
            name: "claim_ttl_max",
            verbatim: false,
            default: Some(DEFAULT_CLAIM_TTL_MAX.to_string()),
        }),
        ["claim_ttl_default"] => Ok(Key::Top {
            name: "claim_ttl_default",
            verbatim: false,
            default: Some(DEFAULT_CLAIM_TTL.to_string()),
        }),
        // No default written down: absent, it is the machine's available
        // parallelism, which is not a value the file could print.
        ["verifier_jobs"] => Ok(Key::Top {
            name: "verifier_jobs",
            verbatim: true,
            default: None,
        }),
        // No default: absent, the resolution falls back to
        // refs/remotes/origin/HEAD and fails rather than guessing (§7).
        ["default_branch"] => Ok(Key::Top {
            name: "default_branch",
            verbatim: false,
            default: None,
        }),
        ["peers"] => Err(CliError::new(
            ExitCode::Generic,
            "'peers' is a mapping: address one by name",
        )
        .with_hint("ank config peers.<name> <path>")),
        ["peers", name] => {
            // A name a scope entry could never spell is a declaration nothing
            // can reach, and writing it would be a silent no-op rather than a
            // configuration. Refused here, where the caller can still type
            // another one.
            if !crate::repo::is_peer_name(name) {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!(
                        "peer name '{name}' cannot be named by a scope: \
                         two or more of a-z, A-Z, 0-9, '-' and '_'"
                    ),
                )
                .with_hint("ank config peers.<name> <path>"));
            }
            Ok(Key::Under {
                map: "peers",
                name: (*name).to_string(),
                verbatim: false,
                default: None,
            })
        }
        ["peers", _, ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': a peer is one path, not a block"),
        )
        .with_hint("ank config peers.<name> <path>")),
        ["weight", counter @ ("hot_files" | "plane_bytes")] => Ok(Key::Under {
            map: "weight",
            name: (*counter).to_string(),
            verbatim: true,
            default: Some(
                if *counter == "hot_files" {
                    DEFAULT_WEIGHT_HOT_FILES
                } else {
                    DEFAULT_WEIGHT_PLANE_BYTES
                }
                .to_string(),
            ),
        }),
        ["proof", "reuse_max_age"] => Ok(Key::Under {
            map: "proof",
            name: "reuse_max_age".to_string(),
            verbatim: false,
            default: Some(DEFAULT_PROOF_REUSE_MAX_AGE.to_string()),
        }),
        ["guard", level @ ("corpus" | "edit")] => Ok(Key::Under {
            map: "guard",
            name: (*level).to_string(),
            verbatim: false,
            default: Some(
                if *level == "corpus" {
                    DEFAULT_GUARD_CORPUS
                } else {
                    DEFAULT_GUARD_EDIT
                }
                .to_string(),
            ),
        }),
        ["guard", ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': the guard has two levels"),
        )
        .with_hint("ank config guard.corpus   or   guard.edit")),
        ["proof", ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': proof has one key"),
        )
        .with_hint("ank config proof.reuse_max_age")),
        ["weight", ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': the weight has two counters"),
        )
        .with_hint("ank config weight.hot_files   or   weight.plane_bytes")),
        // ADR-5c6db7a161b8: declaring who may ratify is a trust act, and it
        // has a verb of its own.
        ["signers", ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': who may ratify is written by ank signers, not by ank config"),
        )
        .with_hint("ank help signers")),
        ["roles", ..] => Err(structured("roles")),
        ["identities", ..] => Err(structured("identities")),
        ["verifiers"] => Err(CliError::new(
            ExitCode::Generic,
            "'verifiers' is a mapping: address one by name",
        )
        .with_hint("ank config verifiers.<name>.run \"<command>\"")),
        ["verifiers", name] => Ok(Key::Block {
            verifier: (*name).to_string(),
        }),
        ["verifiers", name, "run"] => Ok(Key::Field {
            verifier: (*name).to_string(),
            field: "run",
            verbatim: false,
            default: None,
        }),
        ["verifiers", name, "timeout"] => Ok(Key::Field {
            verifier: (*name).to_string(),
            field: "timeout",
            verbatim: false,
            default: Some(DEFAULT_VERIFIER_TIMEOUT.to_string()),
        }),
        // The mark of ADR-443590981e41, whose resolved absence is `false`
        // rather than nothing: a verifier nobody marked is one `ank new task`
        // will not seed, which is a value in effect and not a hole. The
        // verifier still has to be declared -- `write_field` refuses this on
        // an undeclared one exactly as it refuses a timeout.
        ["verifiers", name, "default"] => Ok(Key::Field {
            verifier: (*name).to_string(),
            field: "default",
            verbatim: true,
            default: Some("false".to_string()),
        }),
        ["verifiers", name, "inputs"] => Ok(Key::Inputs {
            verifier: (*name).to_string(),
        }),
        // Withdrawn by ADR-7224dea611c1 and refused by name, ahead of the
        // dotted-name arm below, which would blame the verifier's name for a
        // key the parser simply does not know.
        ["verifiers", _, "env", ..] => Err(unknown_key(path)),
        ["verifiers", _, other] => Err(CliError::new(
            ExitCode::Generic,
            format!("unknown verifier field '{other}'"),
        )
        .with_hint(
            "ank config verifiers.<name>.run   or   .timeout   or   .default   or   .inputs",
        )),
        ["verifiers", _, _, _, ..] => Err(CliError::new(
            ExitCode::Generic,
            format!(
                "'{path}': a verifier whose name contains '.' cannot be addressed by a dotted path"
            ),
        )
        .with_hint("edit .ank/config.yml by hand, or rename the verifier")),
        _ => Err(unknown_key(path)),
    }
}

// ---------------------------------------------------------------------------
// Lines, kept exactly as they were read
// ---------------------------------------------------------------------------

/// One physical line and the terminator it carried.
///
/// Each line keeps its own, rather than the file keeping one: §3 forbids ank to
/// *write* CRLF and says nothing about rewriting a CRLF checkout in place, and
/// a writer that normalised terminators would rewrite every line of such a file
/// to change one key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    text: String,
    eol: String,
}

fn split_lines(text: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        match rest.find('\n') {
            Some(i) => {
                let raw = &rest[..i];
                let (body, eol) = match raw.strip_suffix('\r') {
                    Some(b) => (b, "\r\n"),
                    None => (raw, "\n"),
                };
                out.push(Line {
                    text: body.to_string(),
                    eol: eol.to_string(),
                });
                rest = &rest[i + 1..];
            }
            None => {
                if !rest.is_empty() {
                    out.push(Line {
                        text: rest.to_string(),
                        eol: String::new(),
                    });
                }
                return out;
            }
        }
    }
}

fn join_lines(lines: &[Line]) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(&l.text);
        s.push_str(&l.eol);
    }
    s
}

/// The terminator a line appended to this file should carry.
fn dominant_eol(lines: &[Line]) -> String {
    if lines.iter().any(|l| l.eol == "\r\n") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

/// Gives the last line a terminator, so that appending after it does not join
/// the two. Only the last line can be unterminated.
fn terminate_last(lines: &mut [Line], eol: &str) {
    if let Some(last) = lines.last_mut() {
        if last.eol.is_empty() {
            last.eol = eol.to_string();
        }
    }
}

/// Removes whole lines, terminator included.
///
/// Removing the last line of a file that had no trailing newline leaves the
/// previous line's terminator in place, and that is the faithful reading rather
/// than an oversight: that byte was in the file, the caller asked for the key
/// below it, and clearing it would delete a byte nobody named. The absence of a
/// trailing newline is only preserved when the line carrying it survives.
fn remove_lines(lines: &mut Vec<Line>, r: Range<usize>) {
    lines.drain(r);
}

// ---------------------------------------------------------------------------
// Reading the structure: enough YAML to find a line, and no more
// ---------------------------------------------------------------------------

/// The indentation of a line that carries content.
///
/// `None` for a blank line or one holding only a comment. Neither belongs to a
/// block, and reporting them at indent 0 would close every block a comment
/// happens to sit in.
fn content_indent(text: &str) -> Option<usize> {
    let n = text.len() - text.trim_start_matches(' ').len();
    let rest = &text[n..];
    if rest.trim().is_empty() || rest.starts_with('#') {
        return None;
    }
    Some(n)
}

/// A quoted scalar at the head of `s`: its value, and the bytes it occupies.
fn read_quoted(s: &str, quote: char) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = quote.len_utf8();
    while i < bytes.len() {
        let b = bytes[i];
        if quote == '"' && b == b'\\' {
            let n = *bytes.get(i + 1)? as char;
            out.push(match n {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                other => other,
            });
            i += 2;
            continue;
        }
        if quote == '"' && b == b'"' {
            return Some((out, i + 1));
        }
        if quote == '\'' && b == b'\'' {
            if bytes.get(i + 1) == Some(&b'\'') {
                out.push('\'');
                i += 2;
                continue;
            }
            return Some((out, i + 1));
        }
        let ch = s[i..].chars().next()?;
        out.push(ch);
        i += ch.len_utf8();
    }
    None
}

/// The key a line declares, and the byte offset just past its colon.
///
/// Quoted keys are read as keys: `identities` carries `"marie@laptop": human`,
/// and a scan stopping at the first colon would find one inside the quotes.
fn key_of(text: &str) -> Option<(String, usize)> {
    let indent = text.len() - text.trim_start_matches(' ').len();
    let rest = &text[indent..];
    let first = rest.chars().next()?;
    let (name, mut at) = if first == '"' || first == '\'' {
        let (v, len) = read_quoted(rest, first)?;
        (v, indent + len)
    } else {
        let i = rest.find(':')?;
        (rest[..i].trim_end().to_string(), indent + i)
    };
    let bytes = text.as_bytes();
    while bytes.get(at) == Some(&b' ') {
        at += 1;
    }
    if bytes.get(at) != Some(&b':') {
        return None;
    }
    // `key: value` and `key:`, never `key:value` -- YAML requires the space.
    match bytes.get(at + 1) {
        None | Some(b' ') => Some((name, at + 1)),
        _ => None,
    }
}

/// What a value occupies on its line, as a byte range into it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueSpan {
    start: usize,
    end: usize,
    /// The quote the value already carried, so a rewrite keeps its style.
    quote: Option<char>,
    /// `|` or `>`.
    blocky: bool,
    value: String,
}

fn value_span(text: &str, after_colon: usize) -> ValueSpan {
    let bytes = text.as_bytes();
    let mut start = after_colon;
    while bytes.get(start) == Some(&b' ') {
        start += 1;
    }
    let empty = ValueSpan {
        start,
        end: start,
        quote: None,
        blocky: false,
        value: String::new(),
    };
    let rest = &text[start..];
    let Some(first) = rest.chars().next() else {
        return empty;
    };
    if first == '#' {
        return empty;
    }
    if first == '|' || first == '>' {
        return ValueSpan {
            start,
            end: text.len(),
            quote: None,
            blocky: true,
            value: String::new(),
        };
    }
    if first == '"' || first == '\'' {
        if let Some((v, len)) = read_quoted(rest, first) {
            return ValueSpan {
                start,
                end: start + len,
                quote: Some(first),
                blocky: false,
                value: v,
            };
        }
    }
    // A plain scalar ends where the comment after it begins: in YAML a `#`
    // preceded by whitespace opens one, and a `#` that is not part of the
    // value.
    let end_rel = rest.find(" #").unwrap_or(rest.len());
    let plain = rest[..end_rel].trim_end();
    ValueSpan {
        start,
        end: start + plain.len(),
        quote: None,
        blocky: false,
        value: plain.to_string(),
    }
}

/// The block hanging under the key on line `parent`.
///
/// `range` ends one past the block's last line **of content**, which is the
/// distinction insertion needs: a comment between the last verifier and the
/// next top-level key conventionally introduces what follows it, so a new entry
/// goes above the comment and not below it.
struct Block {
    range: Range<usize>,
    /// The indentation the children use; `None` when the block is empty.
    indent: Option<usize>,
}

fn block_under(lines: &[Line], parent: usize, parent_indent: usize) -> Block {
    let mut indent: Option<usize> = None;
    let mut last_content = parent;
    for (i, line) in lines.iter().enumerate().skip(parent + 1) {
        let Some(n) = content_indent(&line.text) else {
            continue;
        };
        match indent {
            None => {
                if n <= parent_indent {
                    break;
                }
                indent = Some(n);
            }
            Some(c) if n < c => break,
            Some(_) => {}
        }
        last_content = i;
    }
    Block {
        range: parent + 1..last_content + 1,
        indent,
    }
}

/// The line declaring `name` at exactly `indent`, within `range`.
fn find_key(lines: &[Line], range: Range<usize>, indent: usize, name: &str) -> Option<usize> {
    range.into_iter().find(|&i| {
        content_indent(&lines[i].text) == Some(indent)
            && key_of(&lines[i].text).is_some_and(|(k, _)| k == name)
    })
}

/// The `verifiers:` line, the block under it, and the indentation of the
/// entries it holds.
fn verifiers_block(lines: &[Line]) -> Option<(usize, Block)> {
    let vi = find_key(lines, 0..lines.len(), 0, "verifiers")?;
    let block = block_under(lines, vi, 0);
    Some((vi, block))
}

/// The `peers:` line and the block under it.
fn named_block(lines: &[Line], map: &str) -> Option<(usize, Block)> {
    let i = find_key(lines, 0..lines.len(), 0, map)?;
    let block = block_under(lines, i, 0);
    Some((i, block))
}

/// The line declaring `name` under `map`.
fn locate_under(lines: &[Line], map: &str, name: &str) -> Option<usize> {
    let (_, block) = named_block(lines, map)?;
    let child = block.indent?;
    find_key(lines, block.range, child, name)
}

/// The line declaring verifier `name`, and the block of its own fields.
fn locate_verifier(lines: &[Line], name: &str) -> Option<(usize, usize, Block)> {
    let (_, block) = verifiers_block(lines)?;
    let child = block.indent?;
    let i = find_key(lines, block.range.clone(), child, name)?;
    Some((i, child, block_under(lines, i, child)))
}

// ---------------------------------------------------------------------------
// Rendering a value back
// ---------------------------------------------------------------------------

/// Plain scalars YAML resolves to something other than a string. A branch
/// called `no` written plain would be stored as a boolean, and the point of the
/// verb is that the value the caller typed is the value stored.
fn resolves_to_non_string(s: &str) -> bool {
    const WORDS: [&str; 10] = [
        "true", "false", "yes", "no", "on", "off", "null", "~", "y", "n",
    ];
    let lower = s.to_ascii_lowercase();
    WORDS.contains(&lower.as_str()) || s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok()
}

fn plain_safe(s: &str) -> bool {
    const INDICATORS: &str = "-?:,[]{}#&*!|>'\"%@`";
    !s.is_empty()
        && s.trim() == s
        && !s.chars().any(char::is_control)
        && !s.starts_with(|c| INDICATORS.contains(c))
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.contains(" #")
        && !resolves_to_non_string(s)
}

fn double_quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The value as it will be written. A string key keeps the quoting style it
/// already had, which is what makes "quoting style survives a write" true of
/// the edited key as well as of its neighbours.
fn render_value(value: &str, verbatim: bool, keep: Option<char>) -> String {
    if verbatim {
        return value.to_string();
    }
    match keep {
        Some('"') => double_quoted(value),
        Some('\'') if !value.contains('\n') => format!("'{}'", value.replace('\'', "''")),
        _ if plain_safe(value) => value.to_string(),
        _ => double_quoted(value),
    }
}

/// Puts `rendered` where `span` was, and nowhere else on the line.
fn splice(line: &mut Line, span: &ValueSpan, rendered: &str) {
    let head = line.text[..span.start].to_string();
    let tail = line.text[span.end..].to_string();
    // `key:` carries no space before an absent value, and one has to appear.
    let sep = if span.start == span.end && !head.ends_with(' ') {
        " "
    } else {
        ""
    };
    line.text = format!("{head}{sep}{rendered}{tail}");
}

// ---------------------------------------------------------------------------
// Reading and writing one key
// ---------------------------------------------------------------------------

/// The value in effect, in the shape both surfaces print.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    /// Carried by the file.
    Set(String),
    /// Absent, and the tool resolves one.
    Default(String),
    /// Absent, with nothing to resolve.
    Unset,
}

impl Value {
    fn display(&self) -> String {
        match self {
            Value::Set(v) => v.clone(),
            Value::Default(v) => format!("{v} (default)"),
            Value::Unset => "(unset)".to_string(),
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Value::Set(_) => "file",
            Value::Default(_) => "default",
            Value::Unset => "unset",
        }
    }

    fn json(&self) -> String {
        match self {
            Value::Set(v) | Value::Default(v) => crate::json::string(v),
            Value::Unset => "null".to_string(),
        }
    }
}

fn from_default(d: &Option<String>) -> Value {
    match d {
        Some(v) => Value::Default(v.clone()),
        None => Value::Unset,
    }
}

fn scalar_at(line: &Line, name: &str) -> Result<Value> {
    let (_, after) = key_of(&line.text).expect("the line was found by its key");
    let span = value_span(&line.text, after);
    if span.blocky {
        return Err(blocky(name));
    }
    if span.start == span.end {
        return Ok(Value::Unset);
    }
    Ok(Value::Set(span.value))
}

/// Read from the text and not from a parsed [`Config`]: this verb runs on a
/// file that does not parse, and a reader that needed the parse would go silent
/// exactly where it is needed (§4).
fn read_key(lines: &[Line], key: &Key) -> Result<Value> {
    match key {
        Key::Block { verifier } => Err(whole_block(verifier)),
        Key::Inputs { verifier } => read_inputs(lines, verifier),
        Key::Top { name, default, .. } => match find_key(lines, 0..lines.len(), 0, name) {
            Some(i) => scalar_at(&lines[i], name),
            None => Ok(from_default(default)),
        },
        // No default for a peer: a peer nobody declared is a peer that does not
        // exist, and there is nothing for the tool to resolve in its place. A
        // weight counter has one, because the parser applies it.
        Key::Under {
            map, name, default, ..
        } => match locate_under(lines, map, name) {
            Some(i) => scalar_at(&lines[i], &format!("{map}.{name}")),
            None => Ok(from_default(default)),
        },
        Key::Field {
            verifier,
            field,
            default,
            ..
        } => {
            // A timeout resolved for a verifier that is not declared would be a
            // value in effect for something that will never run.
            let Some((_, _, inner)) = locate_verifier(lines, verifier) else {
                return Ok(Value::Unset);
            };
            match inner
                .indent
                .and_then(|g| find_key(lines, inner.range.clone(), g, field))
            {
                Some(j) => scalar_at(&lines[j], &format!("verifiers.{verifier}.{field}")),
                None => Ok(from_default(default)),
            }
        }
    }
}

/// What a write reports as the state before and after it.
///
/// Not [`read_key`], because a whole verifier has no value to read and is still
/// the thing `--unset` removes: reporting `declared -> (unset)` is the honest
/// answer, and refusing to observe it would make the removal unreportable.
fn observe(lines: &[Line], key: &Key) -> Result<Value> {
    match key {
        Key::Block { verifier } => Ok(match locate_verifier(lines, verifier) {
            Some(_) => Value::Set("declared".to_string()),
            None => Value::Unset,
        }),
        other => read_key(lines, other),
    }
}

fn write_key(lines: &mut Vec<Line>, key: &Key, value: &str) -> Result<()> {
    match key {
        Key::Block { verifier } => Err(whole_block(verifier)),
        Key::Inputs { verifier } => write_inputs(lines, verifier, value),
        Key::Top { name, verbatim, .. } => match find_key(lines, 0..lines.len(), 0, name) {
            Some(i) => {
                let (_, after) = key_of(&lines[i].text).expect("found by its key");
                let span = value_span(&lines[i].text, after);
                if span.blocky {
                    return Err(blocky(name));
                }
                let rendered = render_value(value, *verbatim, span.quote);
                splice(&mut lines[i], &span, &rendered);
                Ok(())
            }
            None => {
                let eol = dominant_eol(lines);
                terminate_last(lines, &eol);
                let rendered = render_value(value, *verbatim, None);
                lines.push(Line {
                    text: format!("{name}: {rendered}"),
                    eol,
                });
                Ok(())
            }
        },
        Key::Field {
            verifier,
            field,
            verbatim,
            ..
        } => write_field(lines, verifier, field, *verbatim, value),
        Key::Under {
            map,
            name,
            verbatim,
            ..
        } => write_under(lines, map, name, *verbatim, value),
    }
}

/// One scalar under `peers:`, written the way [`write_field`] writes one under a
/// verifier and one level shallower.
///
/// The `{}` promotion is the same byte and the same reason: a mapping written
/// empty cannot receive a child, and the parent of the key being written is the
/// one byte outside the line that a write is allowed to move (§4).
fn write_under(
    lines: &mut Vec<Line>,
    map: &'static str,
    name: &str,
    verbatim: bool,
    value: &str,
) -> Result<()> {
    let eol = dominant_eol(lines);
    let rendered = render_value(value, verbatim, None);

    let Some(pi) = find_key(lines, 0..lines.len(), 0, map) else {
        terminate_last(lines, &eol);
        lines.push(Line {
            text: format!("{map}:"),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("  {name}: {rendered}"),
            eol,
        });
        return Ok(());
    };

    let (_, after) = key_of(&lines[pi].text).expect("found by its key");
    let span = value_span(&lines[pi].text, after);
    if span.blocky {
        return Err(blocky(map));
    }
    if !span.value.is_empty() && span.value != "{}" {
        return Err(flow_mapping(map));
    }

    if span.value == "{}" {
        let head = lines[pi].text[..span.start].trim_end().to_string();
        let tail = lines[pi].text[span.end..].to_string();
        lines[pi].text = format!("{head}{tail}");
        lines.insert(
            pi + 1,
            Line {
                text: format!("  {name}: {rendered}"),
                eol,
            },
        );
        return Ok(());
    }

    let block = block_under(lines, pi, 0);
    let child = block.indent.unwrap_or(2);
    match find_key(lines, block.range.clone(), child, name) {
        Some(i) => {
            let (_, after) = key_of(&lines[i].text).expect("found by its key");
            let span = value_span(&lines[i].text, after);
            if span.blocky {
                return Err(blocky(&format!("{map}.{name}")));
            }
            let rendered = render_value(value, verbatim, span.quote);
            splice(&mut lines[i], &span, &rendered);
            Ok(())
        }
        None => {
            let at = block.range.end;
            if at == lines.len() {
                terminate_last(lines, &eol);
            }
            lines.insert(
                at,
                Line {
                    text: format!("{}{name}: {rendered}", " ".repeat(child)),
                    eol,
                },
            );
            Ok(())
        }
    }
}

fn write_field(
    lines: &mut Vec<Line>,
    verifier: &str,
    field: &str,
    verbatim: bool,
    value: &str,
) -> Result<()> {
    let eol = dominant_eol(lines);
    let rendered = render_value(value, verbatim, None);

    let Some(vi) = find_key(lines, 0..lines.len(), 0, "verifiers") else {
        // Only `run` declares a verifier: a timeout with no command names one
        // that cannot run, and the file would not parse.
        if field != "run" {
            return Err(undeclared(verifier));
        }
        terminate_last(lines, &eol);
        lines.push(Line {
            text: "verifiers:".to_string(),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("  {verifier}:"),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("    run: {rendered}"),
            eol,
        });
        return Ok(());
    };

    let (_, after) = key_of(&lines[vi].text).expect("found by its key");
    let span = value_span(&lines[vi].text, after);
    if span.blocky {
        return Err(blocky("verifiers"));
    }
    if !span.value.is_empty() && span.value != "{}" {
        return Err(flow_mapping("verifiers"));
    }

    // `verifiers: {}` is what `init` writes, and the first verifier declared
    // into it turns the key into a block mapping. That byte is the parent of
    // the key being written, not a byte beside it (§4) -- and without it the
    // file `init` produces could never receive a verifier at all.
    if span.value == "{}" {
        if field != "run" {
            return Err(undeclared(verifier));
        }
        let head = lines[vi].text[..span.start].trim_end().to_string();
        let tail = lines[vi].text[span.end..].to_string();
        lines[vi].text = format!("{head}{tail}");
        lines.insert(
            vi + 1,
            Line {
                text: format!("  {verifier}:"),
                eol: eol.clone(),
            },
        );
        lines.insert(
            vi + 2,
            Line {
                text: format!("    run: {rendered}"),
                eol,
            },
        );
        return Ok(());
    }

    let block = block_under(lines, vi, 0);
    let child = block.indent.unwrap_or(2);
    match find_key(lines, block.range.clone(), child, verifier) {
        Some(i) => {
            let inner = block_under(lines, i, child);
            let grand = inner.indent.unwrap_or(child + 2);
            match find_key(lines, inner.range.clone(), grand, field) {
                Some(j) => {
                    let (_, after) = key_of(&lines[j].text).expect("found by its key");
                    let span = value_span(&lines[j].text, after);
                    if span.blocky {
                        return Err(blocky(&format!("verifiers.{verifier}.{field}")));
                    }
                    let rendered = render_value(value, verbatim, span.quote);
                    splice(&mut lines[j], &span, &rendered);
                    Ok(())
                }
                None => {
                    let at = inner.range.end;
                    if at == lines.len() {
                        terminate_last(lines, &eol);
                    }
                    lines.insert(
                        at,
                        Line {
                            text: format!("{}{field}: {rendered}", " ".repeat(grand)),
                            eol,
                        },
                    );
                    Ok(())
                }
            }
        }
        None => {
            if field != "run" {
                return Err(undeclared(verifier));
            }
            // A new verifier goes at the end of the block, indented the way the
            // ones already there are rather than the way this file would have
            // been written from scratch.
            let grand = first_grandchild_indent(lines, &block, child).unwrap_or(child + 2);
            let at = block.range.end;
            if at == lines.len() {
                terminate_last(lines, &eol);
            }
            lines.insert(
                at,
                Line {
                    text: format!("{}{verifier}:", " ".repeat(child)),
                    eol: eol.clone(),
                },
            );
            lines.insert(
                at + 1,
                Line {
                    text: format!("{}run: {rendered}", " ".repeat(grand)),
                    eol,
                },
            );
            Ok(())
        }
    }
}

/// The line declaring `path` under verifier `verifier`, one mapping per
/// segment, and the indentation it sits at.
fn locate_nested(lines: &[Line], verifier: &str, path: &[String]) -> Option<(usize, usize)> {
    let (mut at, mut indent, mut block) = locate_verifier(lines, verifier)?;
    for seg in path {
        let g = block.indent?;
        at = find_key(lines, block.range.clone(), g, seg)?;
        indent = g;
        block = block_under(lines, at, g);
    }
    Some((at, indent))
}

fn dotted(verifier: &str, path: &[String]) -> String {
    format!("verifiers.{verifier}.{}", path.join("."))
}

/// A scalar `path` levels below a declared verifier, creating the mappings on
/// the way that are not there yet.
///
/// The [`write_field`] rules, one level at a time: a parent written `{}` is
/// promoted to a block, which is the one byte outside the key a write may
/// move, and a parent written as any other flow mapping is refused by name.
fn write_nested(
    lines: &mut Vec<Line>,
    verifier: &str,
    path: &[String],
    verbatim: bool,
    value: &str,
) -> Result<()> {
    let eol = dominant_eol(lines);
    let Some((mut parent, mut parent_indent, _)) = locate_verifier(lines, verifier) else {
        return Err(undeclared(verifier));
    };
    for (k, seg) in path.iter().enumerate() {
        let leaf = k + 1 == path.len();
        let block = block_under(lines, parent, parent_indent);
        let indent = block.indent.unwrap_or(parent_indent + 2);
        let found = block
            .indent
            .and_then(|g| find_key(lines, block.range.clone(), g, seg));
        let Some(j) = found else {
            let mut at = block.range.end;
            if at == lines.len() {
                terminate_last(lines, &eol);
            }
            for (m, s) in path[k..].iter().enumerate() {
                let pad = " ".repeat(indent + 2 * m);
                let text = if k + m + 1 == path.len() {
                    format!("{pad}{s}: {}", render_value(value, verbatim, None))
                } else {
                    format!("{pad}{s}:")
                };
                lines.insert(
                    at,
                    Line {
                        text,
                        eol: eol.clone(),
                    },
                );
                at += 1;
            }
            return Ok(());
        };
        let (_, after) = key_of(&lines[j].text).expect("found by its key");
        let span = value_span(&lines[j].text, after);
        let name = dotted(verifier, &path[..=k]);
        if span.blocky {
            return Err(blocky(&name));
        }
        if leaf {
            let rendered = render_value(value, verbatim, span.quote);
            splice(&mut lines[j], &span, &rendered);
            return Ok(());
        }
        if span.value == "{}" {
            let head = lines[j].text[..span.start].trim_end().to_string();
            let tail = lines[j].text[span.end..].to_string();
            lines[j].text = format!("{head}{tail}");
        } else if !span.value.is_empty() {
            return Err(flow_mapping(&name));
        }
        parent = j;
        parent_indent = indent;
    }
    Ok(())
}

/// Removes the key at `path` with everything under it, then every mapping on
/// the way up that is left with nothing in it: a mapping holding no key is a
/// key nobody wrote.
fn unset_nested(lines: &mut Vec<Line>, verifier: &str, path: &[String]) {
    let Some((j, indent)) = locate_nested(lines, verifier, path) else {
        return;
    };
    let end = block_under(lines, j, indent).range.end.max(j + 1);
    remove_lines(lines, j..end);
    for depth in (1..path.len()).rev() {
        let Some((p, indent)) = locate_nested(lines, verifier, &path[..depth]) else {
            return;
        };
        let (_, after) = key_of(&lines[p].text).expect("found by its key");
        let span = value_span(&lines[p].text, after);
        if block_under(lines, p, indent).indent.is_some() || !span.value.is_empty() {
            return;
        }
        remove_lines(lines, p..p + 1);
    }
}

/// A list of globs as `inputs` is stored: one flow sequence, every item
/// double-quoted.
fn render_inputs(globs: &[String]) -> String {
    let items: Vec<String> = globs.iter().map(|g| double_quoted(g)).collect();
    format!("[{}]", items.join(", "))
}

fn not_a_list(name: &str) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("'{name}' takes a list of globs, written as one YAML flow sequence"),
    )
    .with_hint(format!("ank config {name} '[\"src/**\", \"Cargo.toml\"]'"))
}

/// `inputs` as the file holds it, flow or block sequence, in the form
/// [`render_inputs`] writes.
fn read_inputs(lines: &[Line], verifier: &str) -> Result<Value> {
    let path = ["inputs".to_string()];
    let name = dotted(verifier, &path);
    let Some((j, indent)) = locate_nested(lines, verifier, &path) else {
        return Ok(Value::Unset);
    };
    let (_, after) = key_of(&lines[j].text).expect("found by its key");
    let span = value_span(&lines[j].text, after);
    let block = block_under(lines, j, indent);
    let mut text = format!("inputs: {}\n", &lines[j].text[span.start..]);
    for line in &lines[block.range] {
        text.push_str(&line.text);
        text.push('\n');
    }
    let parsed: BTreeMap<String, Vec<String>> =
        serde_yaml::from_str(&text).map_err(|_| not_a_list(&name))?;
    Ok(Value::Set(render_inputs(&parsed["inputs"])))
}

fn write_inputs(lines: &mut Vec<Line>, verifier: &str, value: &str) -> Result<()> {
    let path = ["inputs".to_string()];
    let globs: Vec<String> =
        serde_yaml::from_str(value).map_err(|_| not_a_list(&dotted(verifier, &path)))?;
    // A block sequence gives way to the flow one: both are the key named, and
    // the items under it are no other key's bytes.
    if let Some((j, indent)) = locate_nested(lines, verifier, &path) {
        let block = block_under(lines, j, indent);
        if block.indent.is_some() {
            remove_lines(lines, block.range);
        }
    }
    write_nested(lines, verifier, &path, true, &render_inputs(&globs))
}

fn first_grandchild_indent(lines: &[Line], block: &Block, child: usize) -> Option<usize> {
    for i in block.range.clone() {
        if content_indent(&lines[i].text) == Some(child) && key_of(&lines[i].text).is_some() {
            return block_under(lines, i, child).indent;
        }
    }
    None
}

fn unset_key(lines: &mut Vec<Line>, key: &Key) -> Result<()> {
    match key {
        Key::Inputs { verifier } => {
            unset_nested(lines, verifier, &["inputs".to_string()]);
            Ok(())
        }
        Key::Top { name, .. } => {
            if let Some(i) = find_key(lines, 0..lines.len(), 0, name) {
                remove_lines(lines, i..i + 1);
            }
            Ok(())
        }
        Key::Field {
            verifier, field, ..
        } => {
            if *field == "run" {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!("verifiers.{verifier}.run is required: a verifier with no command is not one"),
                )
                .with_hint(format!("ank config --unset verifiers.{verifier}")));
            }
            let Some((_, _, inner)) = locate_verifier(lines, verifier) else {
                return Ok(());
            };
            if let Some(j) = inner
                .indent
                .and_then(|g| find_key(lines, inner.range.clone(), g, field))
            {
                remove_lines(lines, j..j + 1);
            }
            Ok(())
        }
        Key::Under { map, name, .. } => {
            let Some((pi, _)) = named_block(lines, map) else {
                return Ok(());
            };
            let Some(i) = locate_under(lines, map, name) else {
                return Ok(());
            };
            remove_lines(lines, i..i + 1);
            // Same counterpart as `verifiers` below: the last peer removed
            // leaves `peers:` with no children, which is a parse error and not
            // an empty map.
            if block_under(lines, pi, 0).indent.is_none() {
                let (_, after) = key_of(&lines[pi].text).expect("found by its key");
                let span = value_span(&lines[pi].text, after);
                splice(&mut lines[pi], &span, "{}");
            }
            Ok(())
        }
        Key::Block { verifier } => {
            let Some((vi, block)) = verifiers_block(lines) else {
                return Ok(());
            };
            let Some(child) = block.indent else {
                return Ok(());
            };
            let Some(i) = find_key(lines, block.range.clone(), child, verifier) else {
                return Ok(());
            };
            let inner = block_under(lines, i, child);
            remove_lines(lines, i..inner.range.end.max(i + 1));

            // The last verifier removed leaves `verifiers:` with no children,
            // which is not an empty map but a parse error -- so the key goes
            // back to the `{}` `init` writes. The counterpart of the promotion
            // above, and forced by the same rule.
            if block_under(lines, vi, 0).indent.is_none() {
                let (_, after) = key_of(&lines[vi].text).expect("found by its key");
                let span = value_span(&lines[vi].text, after);
                splice(&mut lines[vi], &span, "{}");
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// The verb
// ---------------------------------------------------------------------------

/// `ank config <key> [<value>] [--unset]` (§4).
///
/// Runs without the foundation, as `init` and `help` do: `startup` loads
/// `config.yml` for every other verb, so a file that does not parse fails all
/// of them, `check` included. A verb that exists to repair the file and is
/// disabled by exactly the file it repairs is not a verb.
/// Whether a peer value is shaped as a remote URL rather than a path
/// (ADR-96fe1f9d619a): a scheme followed by `://`, or the scp form
/// `user@host:path`.
///
/// **A single letter before the colon is a drive, never a scheme or a host**,
/// for the reason [`crate::repo::is_peer_name`] gives: `C:/src/b` and
/// `C:\src\b` are paths on Windows and stay accepted. The scp form needs an `@`
/// before the first colon and no separator before it, so a relative path that
/// happens to hold both, `./a@b:c`, is still a path.
fn is_url(value: &str) -> bool {
    if let Some((scheme, _)) = value.split_once("://") {
        let mut chars = scheme.chars();
        if scheme.len() >= 2
            && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        {
            return true;
        }
    }
    value
        .split_once(':')
        .is_some_and(|(before, _)| before.contains('@') && !before.contains(['/', '\\']))
}

/// The refusal of a peer value shaped as a URL, naming the override that says
/// "the peer lives elsewhere" without one (ADR-96fe1f9d619a). Taken before the
/// file is read, so a refused value writes nothing on any path.
///
/// `identity` is the declaring corpus's, when it has one: the override is keyed
/// on it, and a hint the caller can paste is worth one `git` call on a path
/// that is already failing.
fn refuse_peer_url(value: &str, identity: Option<&str>, name: &str) -> Result<()> {
    if !is_url(value) {
        return Ok(());
    }
    Err(CliError::new(
        ExitCode::Generic,
        format!(
            "'{value}' is a URL: a peer is a path to a checkout on this disk, \
             and ank never clones or fetches one"
        ),
    )
    .with_hint(format!(
        "clone it where you want it, then ank config --user peers.{}.{name} <path>",
        identity.unwrap_or("<identity>")
    )))
}

/// The keys `ank config --user` knows, and the whole of them.
pub const USER_KEYS: &[&str] = &["schema", "corpora.<identity>", "peers.<identity>.<name>"];

fn unknown_user_key(path: &str) -> CliError {
    CliError::new(ExitCode::Generic, format!("unknown key '{path}'"))
        .with_hint(format!("keys: {}", USER_KEYS.join(" ")))
}

/// The same closed key set discipline as the repository file, over the reader's
/// own (ADR-96174f1ac2b7).
///
/// **Repeated rather than assumed.** The declarations file is outside `.ank/`,
/// so nothing forbids opening it in an editor — and that is exactly why the
/// surgery has to hold here too: the file is small, hand-edited, and the one
/// place a stale entry makes a corpus vanish.
fn resolve_user_key(path: &str) -> Result<Key> {
    let segs: Vec<&str> = path.split('.').collect();
    if segs.iter().any(|s| s.is_empty()) {
        return Err(unknown_user_key(path));
    }
    match segs.as_slice() {
        ["schema"] => Ok(Key::Top {
            name: "schema",
            verbatim: true,
            default: None,
        }),
        ["corpora", identity] => {
            if !is_identity(identity) {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!("'{identity}' is not a repository identity"),
                )
                .with_hint(
                    "a key is the root commit, never a path, a remote or a slug: \
                     ank status --json prints it under \"corpus\"",
                ));
            }
            Ok(Key::Under {
                map: "corpora",
                name: (*identity).to_string(),
                verbatim: false,
                default: None,
            })
        }
        ["corpora", _, ..] => Err(CliError::new(
            ExitCode::Generic,
            format!("'{path}': a declaration is one path, not a block"),
        )
        .with_hint("ank config --user corpora.<identity> <path>")),
        // A peer's path, overridden for this reader (ADR-da2819aef598): the
        // declaring corpus's identity and the name its `config.yml` gives the
        // peer, written as one flat key under `peers`.
        ["peers", identity, name] => {
            if !is_identity(identity) {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!("'{identity}' is not a repository identity"),
                )
                .with_hint(
                    "a key is the root commit of the corpus declaring the peer: \
                     ank status --json prints it under \"corpus\"",
                ));
            }
            if !crate::repo::is_peer_name(name) {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!(
                        "peer name '{name}' cannot be named by a scope: \
                         two or more of a-z, A-Z, 0-9, '-' and '_'"
                    ),
                )
                .with_hint("ank config --user peers.<identity>.<name> <path>"));
            }
            Ok(Key::Under {
                map: "peers",
                name: format!("{identity}.{name}"),
                verbatim: false,
                default: None,
            })
        }
        _ => Err(unknown_user_key(path)),
    }
}

/// What a declarations file holds before anything has been declared.
///
/// **Written only when a write happens**, never by a read: materialising a file
/// to answer a question about it is how a tool starts leaving traces on
/// machines whose owners asked it not to.
fn empty_corpora_yaml() -> String {
    "schema: 1\ncorpora: {}\n".to_string()
}

/// `ank config --user`, over the reader's declarations.
pub fn run_user(inv: &Invocation, out: &mut dyn Write) -> Result<ExitCode> {
    let Some(path) = corpora_path() else {
        return Err(CliError::new(
            ExitCode::Environment,
            "no home directory in the environment, so there is nowhere to declare a corpus",
        )
        .with_hint(if cfg!(windows) {
            "set APPDATA"
        } else {
            "set XDG_CONFIG_HOME or HOME"
        }));
    };
    let unset = inv.has("--unset");
    let Some(raw_key) = inv.positionals.first() else {
        return Err(CliError::new(ExitCode::Generic, "config expects a key")
            .with_hint(format!("keys: {}", USER_KEYS.join(" "))));
    };
    let key = resolve_user_key(raw_key)?;
    let value = inv.positionals.get(1);
    if unset && value.is_some() {
        return Err(CliError::new(ExitCode::Generic, "--unset takes no value")
            .with_hint(format!("ank config --user --unset {raw_key}")));
    }
    if let (
        Key::Under {
            map: "peers", name, ..
        },
        Some(v),
    ) = (&key, value)
    {
        let (identity, peer) = name.split_once('.').unwrap_or(("", name));
        refuse_peer_url(v, Some(identity), peer)?;
    }

    // A file that is not there reads as a file with nothing in it. A reader who
    // has declared nothing is the ordinary case, and refusing them a read would
    // send them to create a file to be told it is empty.
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            return Err(CliError::new(
                ExitCode::Environment,
                format!("{}: {e}", path.display()),
            ))
        }
    };
    let lines = split_lines(&text);

    if !unset && value.is_none() {
        let before = read_key(&lines, &key)?;
        if inv.json() {
            let doc = crate::json::Obj::document()
                .str("key", raw_key)
                .raw("value", &before.json())
                .str("source", before.source())
                .finish();
            let _ = writeln!(out, "{doc}");
        } else if !inv.quiet() {
            let _ = writeln!(out, "{}", before.display());
        }
        return Ok(ExitCode::Ok);
    }

    // **A file from a newer ank is not edited**, and the differential below
    // cannot say so: it refuses a write that *introduces* a parse failure, so a
    // file that already fails both sides of that comparison passes it silently.
    // Measured before the guard existed: this verb wrote its entry into a
    // `corpora.yml` declaring `schema: 2` and exited 0, leaving a file the same
    // binary refuses to read a second later. Taken here rather than inside
    // `parse_corpora` because the read above stays available — a line-addressed
    // read of one declaration is answerable at any version, and refusing it
    // would leave a reader with no way to see what the file says.
    if let Some(refusal) = newer_than_this_ank(&text, &path, ExitCode::Generic) {
        return Err(refusal);
    }

    // The skeleton, and only on the way to a write. `--unset` on an absent file
    // has nothing to remove and writes nothing at all.
    let text = if text.trim().is_empty() && !unset {
        empty_corpora_yaml()
    } else {
        text
    };
    let lines = split_lines(&text);
    let before = observe(&lines, &key)?;
    let mut edited = lines.clone();
    match value {
        Some(v) => write_key(&mut edited, &key, v)?,
        None => unset_key(&mut edited, &key)?,
    }
    let after_text = join_lines(&edited);
    let after = observe(&edited, &key)?;

    // Differential, exactly as the repository file's is: refused when the write
    // *introduces* a parse failure, never when it is performed on a file that
    // already had one.
    if parse_corpora(&after_text, &path).is_err() && parse_corpora(&text, &path).is_ok() {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "refused: the write would leave {} unreadable",
                path.display()
            ),
        )
        .with_hint(format!("ank config --user {raw_key}")));
    }

    let changed = after_text != text;
    if changed {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                CliError::new(ExitCode::Environment, format!("{}: {e}", dir.display()))
            })?;
        }
        std::fs::write(&path, &after_text).map_err(|e| {
            CliError::new(ExitCode::Environment, format!("{}: {e}", path.display()))
        })?;
    }

    if inv.json() {
        let doc = crate::json::Obj::document()
            .str("key", raw_key)
            .raw("previous", &before.json())
            .raw("value", &after.json())
            .bool("changed", changed)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(out, "{} {}", raw_key, after.display());
    }
    Ok(ExitCode::Ok)
}

/// A declaration this binary has decided on and has not yet written.
///
/// **The two halves exist so the refusals can be taken first**
/// (TASK-0dd151b02854). `init --at` writes in two places -- a corpus in the
/// directory the caller named, and a line in the reader's declarations file --
/// and it used to write the first before deciding whether the second was
/// allowed. A refused declaration then arrived after the corpus it was
/// supposed to authorise already existed, leaving a directory nothing pointed
/// at. Every reason this file may not be written is settled by
/// [`plan_corpus`], which touches nothing; [`PendingDeclaration::write`] is
/// the only part that can no longer refuse.
pub struct PendingDeclaration {
    file: std::path::PathBuf,
    /// The whole file as it will stand, not the line being added: the surgery
    /// below is line-based and its result is what gets written.
    text: String,
}

impl PendingDeclaration {
    /// Writes the decided declaration and answers with the file it wrote.
    ///
    /// What is left here is filesystem failure and nothing else. A caller that
    /// has already created the corpus this declares cannot be handed a
    /// refusal by this function, which is the whole point of the split.
    pub fn write(self) -> Result<std::path::PathBuf> {
        if let Some(dir) = self.file.parent() {
            std::fs::create_dir_all(dir).map_err(|e| {
                CliError::new(ExitCode::Environment, format!("{}: {e}", dir.display()))
            })?;
        }
        std::fs::write(&self.file, self.text).map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("{}: {e}", self.file.display()),
            )
        })?;
        Ok(self.file)
    }
}

/// Decides the declaration naming `path` as the corpus of the repository
/// `identity`, taking every refusal and writing nothing.
///
/// **The same surgery `--user` performs**, and deliberately not a serialiser:
/// this file is hand-edited, and a round trip through `serde_yaml` would
/// rewrite somebody's comments and key order to add one line. `init --at` is
/// the gesture that makes a declaration without a text editor and a printed
/// sha; it is not a licence to reformat the file it lands in.
pub fn plan_corpus(identity: &str, path: &str) -> Result<PendingDeclaration> {
    let Some(file) = corpora_path() else {
        return Err(CliError::new(
            ExitCode::Environment,
            "no home directory in the environment, so there is nowhere to declare a corpus",
        )
        .with_hint(if cfg!(windows) {
            "set APPDATA"
        } else {
            "set XDG_CONFIG_HOME or HOME"
        }));
    };
    let text = match std::fs::read_to_string(&file) {
        Ok(text) if !text.trim().is_empty() => text,
        Ok(_) => empty_corpora_yaml(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => empty_corpora_yaml(),
        Err(e) => {
            return Err(CliError::new(
                ExitCode::Environment,
                format!("{}: {e}", file.display()),
            ))
        }
    };
    // The same guard `run_user` takes, and for the same reason: `init --at` is
    // the other gesture that edits this file, and the differential below is
    // blind to a file that was already unreadable.
    if let Some(refusal) = newer_than_this_ank(&text, &file, ExitCode::Generic) {
        return Err(refusal);
    }
    let mut lines = split_lines(&text);
    let key = Key::Under {
        map: "corpora",
        name: identity.to_string(),
        verbatim: false,
        default: None,
    };
    write_key(&mut lines, &key, path)?;
    let after = join_lines(&lines);
    if parse_corpora(&after, &file).is_err() && parse_corpora(&text, &file).is_ok() {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "refused: the write would leave {} unreadable",
                file.display()
            ),
        )
        .with_hint(format!("ank config --user corpora.{identity}")));
    }
    Ok(PendingDeclaration { file, text: after })
}

/// The declarations file parsed, for the differential refusal above.
fn parse_corpora(text: &str, path: &Path) -> Result<()> {
    if let Some(refusal) = newer_than_this_ank(text, path, ExitCode::Generic) {
        return Err(refusal);
    }
    let file: CorporaFile = serde_yaml::from_str(text)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    if file.schema != CORPORA_SCHEMA {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "{}: schema {} is not {CORPORA_SCHEMA}",
                path.display(),
                file.schema
            ),
        ));
    }
    Ok(())
}

pub fn run(inv: &Invocation, repo: &crate::repo::Repo, out: &mut dyn Write) -> Result<ExitCode> {
    let path = repo.config_path();
    let unset = inv.has("--unset");

    let Some(raw_key) = inv.positionals.first() else {
        return Err(CliError::new(ExitCode::Generic, "config expects a key")
            .with_hint(format!("keys: {}", KEYS.join(" "))));
    };
    let key = resolve_key(raw_key)?;
    let value = inv.positionals.get(1);

    if unset && value.is_some() {
        return Err(CliError::new(ExitCode::Generic, "--unset takes no value")
            .with_hint(format!("ank config --unset {raw_key}")));
    }
    if let (
        Key::Under {
            map: "peers", name, ..
        },
        Some(v),
    ) = (&key, value)
    {
        if is_url(v) {
            refuse_peer_url(v, crate::repo::identity(&repo.corpus).as_deref(), name)?;
        }
    }

    let requires = matches!(
        key,
        Key::Top {
            name: "requires_ank",
            ..
        }
    );
    // Refused on its form before the file is touched, in the sentence the
    // reader gives, rather than as a write that would leave it unreadable.
    if let (true, Some(v)) = (requires, value) {
        if crate::update::Version::parse(v).is_none() {
            return Err(not_a_version(v, &path));
        }
    }

    // A guard level outside its closed set, refused the same way and for the
    // same reason, naming the levels the reader may type instead.
    if let (
        Key::Under {
            map: "guard", name, ..
        },
        Some(v),
    ) = (&key, value)
    {
        let known = if name == "corpus" {
            GuardCorpus::parse(v).is_some()
        } else {
            GuardEdit::parse(v).is_some()
        };
        if !known {
            return Err(not_a_guard(&format!("guard.{name}"), v));
        }
    }

    let text = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::new(ExitCode::Generic, format!("{} not found", path.display()))
                .with_hint("ank init")
        } else {
            CliError::new(ExitCode::Generic, format!("{}: {e}", path.display()))
        }
    })?;
    let lines = split_lines(&text);

    if !unset && value.is_none() {
        // `read_key` is what refuses a whole verifier here: reading one as a
        // value is not a thing, and only `--unset` addresses it.
        let before = read_key(&lines, &key)?;
        if inv.json() {
            let doc = crate::json::Obj::document()
                .str("key", raw_key)
                .raw("value", &before.json())
                .str("source", before.source())
                .finish();
            let _ = writeln!(out, "{doc}");
        } else if !inv.quiet() {
            let _ = writeln!(out, "{}", before.display());
        }
        return Ok(ExitCode::Ok);
    }

    let before = observe(&lines, &key)?;
    let mut edited = lines.clone();
    match value {
        Some(v) if requires => write_requires_ank(&mut edited, &key, v)?,
        Some(v) => write_key(&mut edited, &key, v)?,
        None => unset_key(&mut edited, &key)?,
    }
    let after_text = join_lines(&edited);
    let after = observe(&edited, &key)?;

    // Differential, and it has to be (§4): the write is refused when it
    // *introduces* a parse failure, never when it is performed on a file that
    // already had one -- which is the file this verb exists for.
    let mut warning = None;
    if let Err(e) = parse(&after_text, &path) {
        if parse(&text, &path).is_ok() {
            return Err(CliError::new(
                ExitCode::Generic,
                format!(
                    "refused: the write would leave {} unreadable",
                    path.display()
                ),
            )
            .with_hint(format!("{}\n  -> ank config {raw_key}", e.message)));
        }
        warning = Some(e.message);
    }

    let changed = after_text != text;
    if changed {
        std::fs::write(&path, &after_text)
            .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    }

    if inv.json() {
        let doc = crate::json::Obj::document()
            .str("key", raw_key)
            .raw("previous", &before.json())
            .raw("value", &after.json())
            .bool("changed", changed)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        if changed {
            let _ = writeln!(out, "{raw_key} {} -> {}", before.display(), after.display());
        } else {
            let _ = writeln!(out, "{raw_key} {} (unchanged)", after.display());
        }
        if let Some(w) = warning {
            let _ = writeln!(
                out,
                "{} {} still does not parse: {w}",
                inv.style().yellow("warning:"),
                path.display()
            );
        }
    }
    Ok(ExitCode::Ok)
}

/// `requires_ank` written, and the schema it declares with it
/// (ADR-756e8b0fe416): a file at 1 is raised to 2 in the same write, so the key
/// never lands where a binary predating it would call it a typo. A new key goes
/// beneath `schema:`, where `init` writes it, rather than at the end of the
/// file. The schema is never lowered, here or by `--unset`: a file at 2
/// without the key is a file this binary reads.
fn write_requires_ank(lines: &mut Vec<Line>, key: &Key, value: &str) -> Result<()> {
    let schema = Key::Top {
        name: "schema",
        verbatim: true,
        default: None,
    };
    let at = find_key(lines, 0..lines.len(), 0, "schema");
    let below = matches!(
        at.map(|i| scalar_at(&lines[i], "schema")),
        Some(Ok(Value::Set(ref v))) if v.parse::<u32>().is_ok_and(|n| n < REQUIRES_ANK_SCHEMA)
    );
    if below {
        write_key(lines, &schema, &REQUIRES_ANK_SCHEMA.to_string())?;
    }
    match (find_key(lines, 0..lines.len(), 0, "requires_ank"), at) {
        (None, Some(i)) => {
            let eol = dominant_eol(lines);
            terminate_last(lines, &eol);
            let rendered = render_value(value, false, None);
            lines.insert(
                i + 1,
                Line {
                    text: format!("requires_ank: {rendered}"),
                    eol,
                },
            );
            Ok(())
        }
        _ => write_key(lines, key, value),
    }
}

// ---------------------------------------------------------------------------
// `ank signers` (ADR-5c6db7a161b8)
// ---------------------------------------------------------------------------

/// `ank signers <subcommand>`. Every subcommand writes `signers:` through the
/// line-level writer `config` uses (ADR-e64dfaafd578), so what it does not
/// name is left byte for byte as it was.
pub fn run_signers(
    inv: &Invocation,
    repo: &crate::repo::Repo,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let sub = inv.subcommand.as_deref().unwrap_or_default();
    let (takes, positional): (&[&str], bool) = match sub {
        "add" => (&["--key", "--principal", "--note"], false),
        "retire" => (&["--date"], true),
        "remove" => (&[], true),
        "migrate" => (&[], false),
        other => {
            return Err(
                CliError::new(ExitCode::Generic, format!("unknown subcommand '{other}'"))
                    .with_hint("ank help signers"),
            )
        }
    };
    for name in ["--key", "--principal", "--note", "--date"] {
        if inv.has(name) && !takes.contains(&name) {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("'ank signers {sub}' takes no {name}"),
            )
            .with_hint("ank help signers"));
        }
    }
    if !positional {
        if let Some(extra) = inv.positionals.first() {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("extra argument '{extra}': 'ank signers {sub}' takes none"),
            )
            .with_hint("ank help signers"));
        }
    }
    match sub {
        "add" => add_signer(inv, repo, out),
        "retire" | "remove" => {
            let Some(target) = inv.positionals.first() else {
                return Err(CliError::new(
                    ExitCode::Prerequisite,
                    format!("'ank signers {sub}' needs the key it acts on"),
                )
                .with_hint(format!("ank signers {sub} '<keytype> <key>'")));
            };
            edit_signer(inv, repo, sub, target, out)
        }
        _ => migrate_signers(inv, repo, out),
    }
}

/// One `- key:` entry of a principal's list, as lines of the file.
struct KeyItem {
    /// The line carrying the dash.
    start: usize,
    /// One past its last line of content: a comment below it introduces what
    /// follows, so it is not the entry's to take.
    end: usize,
    /// The indentation of its fields, the one after the dash.
    fields: usize,
    /// The `key:` value, unquoted.
    key: Option<String>,
    /// The line declaring `valid_before`, when it has one.
    valid_before: Option<usize>,
}

/// A line of an entry as a mapping line: the dash that opens the entry read
/// as the space it stands in for, so `key_of` and `value_span` find the field
/// at the same byte offsets.
fn as_field(text: &str, dash: Option<usize>) -> String {
    match dash {
        Some(d) => format!("{} {}", &text[..d], &text[d + 1..]),
        None => text.to_string(),
    }
}

fn opens_item(text: &str, indent: usize) -> bool {
    let rest = &text[indent..];
    rest == "-" || rest.starts_with("- ")
}

/// The entries of the sequence under `keys:` on line `ki`, and the indentation
/// of their dashes. YAML lets a sequence sit at its key's own indentation, so
/// `block_under`, which wants deeper lines, cannot find it.
fn key_items(lines: &[Line], ki: usize, kindent: usize) -> (Option<usize>, Vec<KeyItem>) {
    let mut dash: Option<usize> = None;
    let mut items: Vec<KeyItem> = Vec::new();
    for (i, line) in lines.iter().enumerate().skip(ki + 1) {
        let Some(n) = content_indent(&line.text) else {
            continue;
        };
        let d = match dash {
            None if n >= kindent && opens_item(&line.text, n) => {
                dash = Some(n);
                n
            }
            None => break,
            Some(d) => d,
        };
        let first = n == d && opens_item(&line.text, n);
        if first {
            items.push(KeyItem {
                start: i,
                end: i + 1,
                fields: 0,
                key: None,
                valid_before: None,
            });
        } else if n <= d {
            break;
        }
        let Some(item) = items.last_mut() else { break };
        item.end = i + 1;
        let text = as_field(&line.text, first.then_some(d));
        let Some(fi) = content_indent(&text) else {
            continue;
        };
        if first {
            item.fields = fi;
        }
        if fi != item.fields {
            continue;
        }
        if let Some((name, at)) = key_of(&text) {
            match name.as_str() {
                "key" => item.key = Some(value_span(&text, at).value),
                "valid_before" => item.valid_before = Some(i),
                _ => {}
            }
        }
    }
    (dash, items)
}

/// Where a principal's keys are: the principal's line, the end of its block,
/// the `keys:` line and its indentation.
struct PrincipalAt {
    line: usize,
    end: usize,
    keys: Option<(usize, usize)>,
}

fn locate_principal(lines: &[Line], name: &str) -> Option<PrincipalAt> {
    let (_, block) = named_block(lines, "signers")?;
    let child = block.indent?;
    let line = find_key(lines, block.range, child, name)?;
    let own = block_under(lines, line, child);
    let keys = own.indent.and_then(|k| {
        let ki = find_key(lines, own.range.clone(), k, "keys")?;
        Some((ki, k))
    });
    // The block of a principal ends with the last of its keys, which may sit
    // at the principal's own indentation plus nothing: take the further one.
    let end = keys
        .and_then(|(ki, k)| key_items(lines, ki, k).1.last().map(|i| i.end))
        .map_or(own.range.end, |e| e.max(own.range.end));
    Some(PrincipalAt { line, end, keys })
}

/// Whether a declared key is the one `target` names: `<keytype> <key>`, or
/// the key alone.
fn names_key(declared: &str, target: &str) -> bool {
    let d: Vec<&str> = declared.split_whitespace().collect();
    let t: Vec<&str> = target.split_whitespace().collect();
    match (d.as_slice(), t.as_slice()) {
        ([dt, dk], [tt, tk]) => dt.eq_ignore_ascii_case(tt) && dk == tk,
        ([_, dk], [tk]) => dk == tk,
        _ => false,
    }
}

/// `<keytype> <key>` out of the text of a public key, its comment dropped.
fn public_key(text: &str) -> Option<String> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with('#'))?;
    let mut fields = line.split_whitespace();
    let keytype = fields.next()?;
    let key = fields.next()?;
    is_keytype(keytype).then(|| format!("{} {key}", keytype.to_ascii_lowercase()))
}

fn git_config(repo: &crate::repo::Repo, key: &str) -> Option<String> {
    crate::git::output(&repo.worktree, &["config", "--get", key])
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|v| !v.is_empty())
}

/// What `--key` names: a public key file, or `<keytype> <key>` as written.
fn key_from_flag(value: &str) -> Result<String> {
    let path = Path::new(value);
    if path.is_file() {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
        return public_key(&text).ok_or_else(|| {
            CliError::new(
                ExitCode::Generic,
                format!("--key {value}: the file holds no public key '<keytype> <key>'"),
            )
            .with_hint("ank signers add --key <file>.pub")
        });
    }
    public_key(value).ok_or_else(|| {
        CliError::new(
            ExitCode::Generic,
            format!("--key {value}: neither a file nor '<keytype> <key>'"),
        )
        .with_hint("ank signers add --key ~/.ssh/id_ed25519.pub")
    })
}

/// The key git signs with, in the form `signers:` declares it.
fn key_from_git(repo: &crate::repo::Repo) -> Result<String> {
    let missing = |why: String| {
        CliError::new(ExitCode::Prerequisite, why)
            .with_hint("ank signers add --key <file> (or: git config user.signingkey <key>)")
    };
    let Some(value) = git_config(repo, "user.signingkey") else {
        return Err(missing(
            "git names no user.signingkey, and no --key was given".to_string(),
        ));
    };
    let format = git_config(repo, "gpg.format").unwrap_or_else(|| "openpgp".to_string());
    match format.as_str() {
        "ssh" => {
            if let Some(literal) = value.strip_prefix("key::") {
                return public_key(literal).ok_or_else(|| {
                    missing(format!(
                        "user.signingkey '{value}' is not '<keytype> <key>'"
                    ))
                });
            }
            if let Some(k) = public_key(&value) {
                return Ok(k);
            }
            // A path, to the public half or to the private one beside it.
            let path = std::path::PathBuf::from(&value);
            let path = if path.is_relative() {
                repo.worktree.join(path)
            } else {
                path
            };
            let mut public = path.clone().into_os_string();
            public.push(".pub");
            [path, public.into()]
                .iter()
                .find_map(|p| std::fs::read_to_string(p).ok().and_then(|t| public_key(&t)))
                .ok_or_else(|| {
                    missing(format!(
                        "user.signingkey '{value}' names no readable public key"
                    ))
                })
        }
        "openpgp" => {
            let id = value
                .trim_start_matches("0x")
                .trim_end_matches('!')
                .to_ascii_uppercase();
            let hex = id.bytes().all(|b| b.is_ascii_hexdigit());
            if hex && [16, 40, 64].contains(&id.len()) {
                Ok(format!("gpg {id}"))
            } else {
                Err(missing(format!(
                    "user.signingkey '{value}' is not a fingerprint or a long key id"
                )))
            }
        }
        other => Err(missing(format!(
            "gpg.format {other}: signers: declares gpg and ssh keys only"
        ))),
    }
}

/// Inserts lines at `at`, with the file's own terminator.
fn insert_lines(lines: &mut Vec<Line>, at: usize, new: Vec<String>) {
    let eol = dominant_eol(lines);
    if at == lines.len() {
        terminate_last(lines, &eol);
    }
    let new: Vec<Line> = new
        .into_iter()
        .map(|text| Line {
            text,
            eol: eol.clone(),
        })
        .collect();
    lines.splice(at..at, new);
}

fn hand_edit(what: String) -> CliError {
    CliError::new(ExitCode::Generic, format!("refused: {what}"))
        .with_hint("edit signers: in .ank/config.yml by hand, then ank check")
}

/// Reads the result back through the parser every verb uses, and writes it
/// only when it says what the subcommand meant.
fn write_signers(path: &Path, lines: &[Line], holds: impl Fn(&[Signer]) -> bool) -> Result<()> {
    let after = join_lines(lines);
    let read = parse(&after, path).map_err(|e| {
        hand_edit(format!(
            "the change would leave {} unreadable",
            path.display()
        ))
        .with_hint(e.message)
    })?;
    if !holds(&read.signers) {
        return Err(hand_edit(
            "the change would not read back as written".to_string(),
        ));
    }
    std::fs::write(path, after)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))
}

fn report_signer(
    inv: &Invocation,
    out: &mut dyn Write,
    action: &str,
    principal: &str,
    key: &str,
    valid_before: Option<&str>,
) {
    if inv.json() {
        let doc = crate::json::Obj::document()
            .str("action", action)
            .str("principal", principal)
            .str("key", key)
            .opt_str("valid_before", valid_before)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let keytype = key.split_whitespace().next().unwrap_or_default();
        let _ = match valid_before {
            Some(d) => writeln!(
                out,
                "{action} {keytype} key of {principal}, valid before {d}"
            ),
            None => writeln!(out, "{action} {keytype} key of {principal}"),
        };
        let _ = writeln!(out, "  review: git diff .ank");
    }
}

/// `ank signers add`: one key, under its principal, after the last key there;
/// a principal not yet declared after the last one; a block not yet written
/// at the end of the file.
fn add_signer(inv: &Invocation, repo: &crate::repo::Repo, out: &mut dyn Write) -> Result<ExitCode> {
    let key = match inv.value("--key") {
        Some(v) => key_from_flag(v)?,
        None => key_from_git(repo)?,
    };
    let principal = match inv.value("--principal").map(str::to_string) {
        Some(p) => p,
        None => git_config(repo, "user.email").ok_or_else(|| {
            CliError::new(
                ExitCode::Prerequisite,
                "git names no user.email, and no --principal was given",
            )
            .with_hint("ank signers add --principal <email>")
        })?,
    };
    let note: Vec<String> = inv
        .value("--note")
        .map(|n| n.lines().map(str::to_string).collect())
        .unwrap_or_default();

    let path = repo.config_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    let current = parse(&text, &path)?;
    if let Some(s) = current
        .signers
        .iter()
        .find(|s| names_key(&format!("{} {}", s.keytype, s.key), &key))
    {
        return Err(CliError::new(
            ExitCode::Transition,
            format!(
                "refused: the {} key is already declared under {}",
                s.keytype, s.principal
            ),
        )
        .with_hint(format!(
            "ank signers retire '{key}', or ank signers remove '{key}'"
        )));
    }

    let mut lines = split_lines(&text);
    let entry = |dash: usize| {
        let pad = " ".repeat(dash);
        let mut l = vec![format!("{pad}- key: {}", scalar(&key))];
        if !note.is_empty() {
            l.extend(note_lines(&note, &format!("{pad}  ")));
        }
        l
    };
    let declared = current.signers.iter().any(|s| s.principal == principal);
    match named_block(&lines, "signers") {
        None => {
            let block = render_signers(&[Migrated {
                principal: principal.clone(),
                keytype: key
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
                key: key
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string(),
                note: note.clone(),
                valid_before: None,
            }]);
            let at = lines.len();
            insert_lines(&mut lines, at, block);
        }
        Some((si, block)) => {
            if !value_span(&lines[si].text, key_of(&lines[si].text).map_or(0, |k| k.1))
                .value
                .is_empty()
            {
                return Err(hand_edit("signers: is not written as a block".to_string()));
            }
            match locate_principal(&lines, &principal) {
                Some(PrincipalAt {
                    keys: Some((ki, kindent)),
                    ..
                }) => {
                    let (dash, items) = key_items(&lines, ki, kindent);
                    let at = items.last().map_or(ki + 1, |i| i.end);
                    insert_lines(&mut lines, at, entry(dash.unwrap_or(kindent + 2)));
                }
                Some(_) => {
                    return Err(hand_edit(format!(
                        "{principal} under signers: carries no keys: list to add to"
                    )))
                }
                None if declared => {
                    return Err(hand_edit(format!(
                        "{principal} under signers: is not written as a block"
                    )))
                }
                None => {
                    let indent = block.indent.unwrap_or(2);
                    let pad = " ".repeat(indent);
                    let mut new = vec![
                        format!("{pad}{}:", scalar(&principal)),
                        format!("{pad}  keys:"),
                    ];
                    new.extend(entry(indent + 4));
                    insert_lines(&mut lines, block.range.end, new);
                }
            }
        }
    }
    let wanted_note = (!note.is_empty()).then(|| note.join("\n"));
    write_signers(&path, &lines, |read| {
        read.iter().any(|s| {
            s.principal == principal
                && names_key(&format!("{} {}", s.keytype, s.key), &key)
                && s.note == wanted_note
        }) && read.len() == current.signers.len() + 1
    })?;
    report_signer(inv, out, "added", &principal, &key, None);
    Ok(ExitCode::Ok)
}

/// `ank signers retire` and `ank signers remove`, which find one declared key
/// and change only its lines.
fn edit_signer(
    inv: &Invocation,
    repo: &crate::repo::Repo,
    sub: &str,
    target: &str,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let date = match inv.value("--date") {
        Some(d) if date_days(d).is_none() => {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("--date '{d}' is not a YYYY-MM-DD date"),
            )
            .with_hint(format!("ank signers retire '{target}' --date 2026-01-31")))
        }
        Some(d) => d.to_string(),
        None => crate::claim::now_utc()[..10].to_string(),
    };

    let path = repo.config_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    let current = parse(&text, &path)?;
    let matching: Vec<&Signer> = current
        .signers
        .iter()
        .filter(|s| names_key(&format!("{} {}", s.keytype, s.key), target))
        .collect();
    let signer = match matching.as_slice() {
        [one] => (*one).clone(),
        [] => {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("signers: declares no key matching '{target}'"),
            )
            .with_hint("ank review"))
        }
        _ => {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("'{target}' matches {} declared keys", matching.len()),
            )
            .with_hint(format!("ank signers {sub} '<keytype> <key>'")))
        }
    };
    let full = format!("{} {}", signer.keytype, signer.key);

    let mut lines = split_lines(&text);
    let at = locate_principal(&lines, &signer.principal).and_then(|p| {
        let (ki, kindent) = p.keys?;
        let (_, items) = key_items(&lines, ki, kindent);
        let i = items
            .into_iter()
            .position(|i| i.key.as_deref().is_some_and(|k| names_key(k, &full)))?;
        Some((p, ki, kindent, i))
    });
    let Some((p, ki, kindent, index)) = at else {
        return Err(hand_edit(format!(
            "the key of {} is declared in a form this writer does not locate",
            signer.principal
        )));
    };
    let (dash, items) = key_items(&lines, ki, kindent);
    let item = &items[index];

    if sub == "retire" {
        match item.valid_before {
            Some(vi) => {
                let field = as_field(&lines[vi].text, dash.filter(|_| vi == item.start));
                let colon = key_of(&field).map_or(0, |k| k.1);
                let span = value_span(&field, colon);
                splice(&mut lines[vi], &span, &date);
            }
            None => {
                let pad = " ".repeat(item.fields);
                insert_lines(
                    &mut lines,
                    item.end,
                    vec![format!("{pad}valid_before: {date}")],
                );
            }
        }
        write_signers(&path, &lines, |read| {
            read.iter().any(|s| {
                s.principal == signer.principal
                    && s.keytype == signer.keytype
                    && s.key == signer.key
                    && s.note == signer.note
                    && s.valid_before.as_deref() == Some(date.as_str())
            }) && read.len() == current.signers.len()
        })?;
        report_signer(inv, out, "retired", &signer.principal, &full, Some(&date));
        return Ok(ExitCode::Ok);
    }

    // remove: the entry, then the principal it leaves empty, then the block.
    let last_of_principal = items.len() == 1;
    let last_principal = current
        .signers
        .iter()
        .all(|s| s.principal == signer.principal);
    if last_of_principal && last_principal {
        let (si, block) = named_block(&lines, "signers").expect("located above");
        remove_lines(&mut lines, si..block.range.end.max(p.end));
    } else if last_of_principal {
        remove_lines(&mut lines, p.line..p.end);
    } else {
        remove_lines(&mut lines, item.start..item.end);
    }
    write_signers(&path, &lines, |read| {
        read.len() + 1 == current.signers.len()
            && !read
                .iter()
                .any(|s| s.keytype == signer.keytype && s.key == signer.key)
    })?;
    report_signer(inv, out, "removed", &signer.principal, &full, None);
    Ok(ExitCode::Ok)
}

/// One key read out of `.ank/allowed_signers`, before it is written anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Migrated {
    principal: String,
    keytype: String,
    key: String,
    note: Vec<String>,
    valid_before: Option<String>,
}

/// Reads git's allowed-signers layout, `principal [options] keytype key
/// [comment]`, keeping what the layout's readers throw away: the comment lines
/// above a key and the comment after it become that key's note, in order.
///
/// Refuses rather than drops. A line it cannot read, or an option `signers:`
/// has no field for, stops the migration naming the line: the file is deleted
/// at the end, and a key quietly left out of the block is a key that stops
/// ratifying without anybody having decided it.
fn read_allowed_signers(text: &str) -> std::result::Result<Vec<Migrated>, String> {
    let mut keys: Vec<Migrated> = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let t = line.trim();
        if t.is_empty() {
            // A blank line inside a block of comments is a paragraph break in
            // the note; a blank line with nothing pending separates nothing.
            if !pending.is_empty() {
                pending.push(String::new());
            }
            continue;
        }
        if let Some(c) = t.strip_prefix('#') {
            pending.push(c.strip_prefix(' ').unwrap_or(c).trim_end().to_string());
            continue;
        }
        let at = |why: &str| format!(".ank/allowed_signers line {}: {why}: {t}", n + 1);
        let mut fields = t.split_whitespace();
        let principal = fields.next().unwrap_or_default().to_string();
        let mut valid_before = None;
        let keytype = loop {
            let Some(f) = fields.next() else {
                return Err(at("no key type"));
            };
            if is_keytype(f) {
                break f.to_ascii_lowercase();
            }
            match f.strip_prefix("valid-before=") {
                Some(v) => {
                    let v = v.trim_matches('"');
                    let date = (v.len() == 8 && v.bytes().all(|b| b.is_ascii_digit()))
                        .then(|| format!("{}-{}-{}", &v[..4], &v[4..6], &v[6..]))
                        .filter(|d| date_days(d).is_some())
                        .ok_or_else(|| at("valid-before is not a YYYYMMDD date"))?;
                    valid_before = Some(date);
                }
                None => return Err(at(&format!("'{f}' is an option signers: has no field for"))),
            }
        };
        let Some(key) = fields.next() else {
            return Err(at("no key after the key type"));
        };
        while pending.last().is_some_and(String::is_empty) {
            pending.pop();
        }
        let mut note = std::mem::take(&mut pending);
        let trailing: Vec<&str> = fields.collect();
        if !trailing.is_empty() {
            note.push(trailing.join(" "));
        }
        keys.push(Migrated {
            principal,
            keytype,
            key: key.to_string(),
            note,
            valid_before,
        });
    }
    while pending.last().is_some_and(String::is_empty) {
        pending.pop();
    }
    if !pending.is_empty() {
        // Comments below the last key belong to no key; they are kept on the
        // last one rather than lost, after a blank line.
        let Some(last) = keys.last_mut() else {
            return Err(
                ".ank/allowed_signers declares no key: nothing to move, and its \
                 comments would have nowhere to go"
                    .to_string(),
            );
        };
        if !last.note.is_empty() {
            last.note.push(String::new());
        }
        last.note.extend(pending);
    }
    if keys.is_empty() {
        return Err(".ank/allowed_signers declares no key".to_string());
    }
    Ok(keys)
}

/// A scalar as a plain one when YAML reads it back unchanged, double-quoted
/// otherwise.
fn scalar(s: &str) -> String {
    if plain_safe(s) {
        s.to_string()
    } else {
        double_quoted(s)
    }
}

/// A note, at the indentation of its key's fields. Several lines are a
/// literal block, which keeps them readable in review; a line a literal block
/// could not carry as written falls back to one double-quoted scalar.
fn note_lines(note: &[String], indent: &str) -> Vec<String> {
    if note.len() == 1 {
        return vec![format!("{indent}note: {}", scalar(&note[0]))];
    }
    let literal = note.first().is_some_and(|l| !l.is_empty())
        && note
            .iter()
            .all(|l| !l.starts_with([' ', '\t']) && !l.chars().any(char::is_control));
    if !literal {
        return vec![format!("{indent}note: {}", double_quoted(&note.join("\n")))];
    }
    let mut out = vec![format!("{indent}note: |-")];
    for l in note {
        out.push(match l.is_empty() {
            true => String::new(),
            false => format!("{indent}  {l}"),
        });
    }
    out
}

/// The `signers:` block, principals in the order the file first names them.
fn render_signers(keys: &[Migrated]) -> Vec<String> {
    let mut principals: Vec<&str> = Vec::new();
    for k in keys {
        if !principals.contains(&k.principal.as_str()) {
            principals.push(&k.principal);
        }
    }
    let mut out = vec!["signers:".to_string()];
    for p in principals {
        out.push(format!("  {}:", scalar(p)));
        out.push("    keys:".to_string());
        for k in keys.iter().filter(|k| k.principal == p) {
            out.push(format!(
                "      - key: {}",
                scalar(&format!("{} {}", k.keytype, k.key))
            ));
            if !k.note.is_empty() {
                out.extend(note_lines(&k.note, "        "));
            }
            if let Some(d) = &k.valid_before {
                out.push(format!("        valid_before: {d}"));
            }
        }
    }
    out
}

/// `ank signers migrate`: `.ank/allowed_signers` becomes `signers:` in
/// `config.yml`, and the file is deleted.
///
/// The block is appended, so every byte above it is the file as it was; it is
/// read back through the parser every other verb uses and compared key by key
/// before anything is written, and the file is removed only once the block is
/// on disk.
fn migrate_signers(
    inv: &Invocation,
    repo: &crate::repo::Repo,
    out: &mut dyn Write,
) -> Result<ExitCode> {
    let source = repo.ank.join("allowed_signers");
    let file = match std::fs::read_to_string(&source) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::new(
                ExitCode::Prerequisite,
                "no .ank/allowed_signers to migrate: who may ratify is already signers: in .ank/config.yml",
            )
            .with_hint("ank review"))
        }
        Err(e) => {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("{}: {e}", source.display()),
            ))
        }
    };
    let keys = read_allowed_signers(&file)
        .map_err(|why| CliError::new(ExitCode::Generic, format!("refused: {why}")))?;

    let path = repo.config_path();
    let text = std::fs::read_to_string(&path)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    let current = parse(&text, &path)?;
    // What `signers:` already declares is not declared twice. Anything left to
    // add beside a block that exists is refused: appending a second `signers:`
    // would not parse, and merging into a block a human wrote is `ank signers
    // add`'s surgery, not this one's.
    let declared = |k: &Migrated| {
        current
            .signers
            .iter()
            .any(|s| s.principal == k.principal && s.keytype == k.keytype && s.key == k.key)
    };
    let remaining: Vec<Migrated> = keys.iter().filter(|k| !declared(k)).cloned().collect();
    let mut lines = split_lines(&text);
    if !remaining.is_empty() {
        if find_key(&lines, 0..lines.len(), 0, "signers").is_some() {
            return Err(CliError::new(
                ExitCode::Generic,
                format!(
                    "refused: {} key(s) of .ank/allowed_signers are not under the signers: block \
                     .ank/config.yml already has, and migrate only writes an absent one",
                    remaining.len()
                ),
            )
            .with_hint(
                "move them under signers: by hand, then ank signers migrate removes the file",
            ));
        }
        let eol = dominant_eol(&lines);
        terminate_last(&mut lines, &eol);
        for l in render_signers(&remaining) {
            lines.push(Line {
                text: l,
                eol: eol.clone(),
            });
        }
        let after_text = join_lines(&lines);
        // Read back through the parser, and compared field by field: a note
        // YAML would fold or a principal it would retype is caught here,
        // before the file that held it is gone.
        let read = parse(&after_text, &path).map_err(|e| {
            CliError::new(
                ExitCode::Generic,
                format!(
                    "refused: the block would leave {} unreadable",
                    path.display()
                ),
            )
            .with_hint(e.message)
        })?;
        for k in &remaining {
            let note = (!k.note.is_empty()).then(|| k.note.join("\n"));
            let back = read.signers.iter().any(|s| {
                s.principal == k.principal
                    && s.keytype == k.keytype
                    && s.key == k.key
                    && s.note == note
                    && s.valid_before == k.valid_before
            });
            if !back {
                return Err(CliError::new(
                    ExitCode::Generic,
                    format!(
                        "refused: the key of {} would not read back as written",
                        k.principal
                    ),
                ));
            }
        }
        std::fs::write(&path, &after_text)
            .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", path.display())))?;
    }
    std::fs::remove_file(&source)
        .map_err(|e| CliError::new(ExitCode::Generic, format!("{}: {e}", source.display())))?;

    let mut principals: Vec<&str> = remaining.iter().map(|k| k.principal.as_str()).collect();
    principals.sort_unstable();
    principals.dedup();
    if inv.json() {
        let doc = crate::json::Obj::document()
            .num("principals", principals.len())
            .num("keys", remaining.len())
            .str("removed", ".ank/allowed_signers")
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        let _ = writeln!(
            out,
            "migrated {} key(s) of {} principal(s) under signers: in .ank/config.yml, \
             and removed .ank/allowed_signers",
            remaining.len(),
            principals.len()
        );
        let _ = writeln!(out, "  review: git diff .ank");
    }
    Ok(ExitCode::Ok)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> &'static Path {
        Path::new(".ank/config.yml")
    }

    #[test]
    fn durations_in_all_four_units() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(1800));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86_400));
        assert_eq!(parse_duration(" 10m ").unwrap(), Duration::from_secs(600));
    }

    #[test]
    fn invalid_durations_are_named_precisely() {
        assert!(parse_duration("30").unwrap_err().contains("missing unit"));
        assert!(parse_duration("30w").unwrap_err().contains("unknown unit"));
        assert!(parse_duration("h").unwrap_err().contains("digit expected"));
        assert!(parse_duration("").unwrap_err().contains("empty"));
    }

    /// Above the range the diagnosis is the binary's age, not the file's
    /// content (TASK-742cd978a806). This test used to assert *unknown schema
    /// 2*, and the sentence moved on purpose: "unknown" describes the reader's
    /// state and leaves the reader to guess whose fault it is, where "the file
    /// is newer than this ank" says which of the two to change.
    #[test]
    fn a_newer_schema_is_refused_as_newer_and_not_as_unknown() {
        let err = parse("schema: 3\n", p()).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("schema 3"), "{}", err.message);
        assert!(
            err.message.contains("this binary reads up to 2"),
            "{}",
            err.message
        );
        assert!(
            err.message.contains("newer than this ank"),
            "{}",
            err.message
        );
        assert!(err.hint.is_some());
    }

    /// Below the range there is no older tool to name, so the older sentence is
    /// still the true one and keeps its place.
    #[test]
    fn a_schema_below_the_range_is_refused_as_unknown() {
        let err = parse("schema: 0\n", p()).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("unknown schema 0"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    /// The probe reads the version and never becomes a second parser: a file
    /// whose `schema` is in range is still handed to `ConfigFile` whole, so a
    /// key nobody knows is refused by name exactly as before.
    #[test]
    fn the_version_probe_refuses_nothing_a_readable_file_declares() {
        let err = parse("schema: 1\nchannels:\n  release: main\n", p()).unwrap_err();
        assert!(err.message.contains("unknown field"), "{}", err.message);
        assert!(err.message.contains("channels"), "{}", err.message);
        assert!(
            !err.message.contains("newer than this ank"),
            "{}",
            err.message
        );
    }

    #[test]
    fn an_unknown_field_is_refused_rather_than_ignored() {
        let err = parse("schema: 1\nbudget_context: 10\n", p()).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("budget_context"), "{}", err.message);
    }

    #[test]
    fn the_five_fields_of_the_spec_are_read() {
        let cfg = parse(
            "\
schema: 1
context_budget: 4000
claim_ttl_max: 90m
verifiers:
  cargo-test:
    run: cargo test --workspace -q
    timeout: 10m
  fmt-check:
    run: cargo fmt --check
roles:
  agent:
    can: [context, claim]
    cannot: [delete]
  human:
    can: [\"*\"]
identities:
  \"marie@laptop\": human
",
            p(),
        )
        .unwrap();

        assert_eq!(cfg.context_budget, 4000);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(5400));
        assert_eq!(
            cfg.verifier("cargo-test").unwrap().run,
            "cargo test --workspace -q"
        );
        // The timeout default is the one from the spec, not zero.
        assert_eq!(
            cfg.verifier("fmt-check").unwrap().timeout,
            Duration::from_secs(600)
        );
        assert_eq!(cfg.roles["agent"].cannot, vec!["delete".to_string()]);
        assert_eq!(cfg.identities["marie@laptop"], "human");
    }

    /// The repository states its own rhythm, and the tool's value is what an
    /// absent key resolves to (§3, ADR-0bb7ea8991bc).
    ///
    /// The last assertion is the one worth having: `claim_ttl_default` and
    /// [`crate::claim::DEFAULT_TTL`] are thirty minutes written twice, once as
    /// a duration this file can carry and once as a constant the claim record
    /// falls back to, and two spellings of one number is how they start to
    /// disagree. Nothing else pins them together.
    #[test]
    fn claim_ttl_default_is_read_and_falls_back_to_the_tools_value() {
        let cfg = parse("schema: 1\nclaim_ttl_default: 90m\n", p()).unwrap();
        assert_eq!(cfg.claim_ttl_default, Duration::from_secs(5400));

        let cfg = parse("schema: 1\n", p()).unwrap();
        assert_eq!(cfg.claim_ttl_default, Duration::from_secs(30 * 60));

        // A value the file cannot mean is named with its key, like every other
        // duration here.
        let err = parse("schema: 1\nclaim_ttl_default: 30w\n", p()).unwrap_err();
        assert!(err.message.contains("claim_ttl_default"), "{}", err.message);

        assert_eq!(
            parse_duration(DEFAULT_CLAIM_TTL).unwrap(),
            crate::claim::DEFAULT_TTL,
            "the file's default and the record's fallback are the same thirty \
             minutes, and only this assertion says so"
        );
    }

    #[test]
    fn default_branch_is_optional_and_read_when_present() {
        let cfg = parse("schema: 1\ndefault_branch: trunk\n", p()).unwrap();
        assert_eq!(cfg.default_branch.as_deref(), Some("trunk"));

        let cfg = parse("schema: 1\n", p()).unwrap();
        assert_eq!(cfg.default_branch, None, "the key is optional");

        // A blank value is an absence written out, not a branch named "": it
        // must reach the resolution as absent, otherwise the error naming the
        // two missing sources would never fire.
        let cfg = parse("schema: 1\ndefault_branch: \"  \"\n", p()).unwrap();
        assert_eq!(cfg.default_branch, None);
    }

    #[test]
    fn this_repositorys_own_config_loads() {
        // Dogfooding: the config that drives this repository must pass the
        // parser we just wrote, otherwise one of the two is lying.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.ank/config.yml")
            .canonicalize()
            .unwrap();
        let cfg = load(&path).unwrap();
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.default_branch.as_deref(), Some("main"));
        assert!(cfg.verifier("cargo-test").is_some());
        assert!(cfg.verifier("check-repo").is_some());
    }

    #[test]
    fn the_default_yaml_written_by_init_reads_back() {
        let cfg = parse(&default_yaml(), p()).unwrap();
        assert_eq!(cfg.schema, MIN_SCHEMA);
        assert_eq!(cfg.context_budget, DEFAULT_CONTEXT_BUDGET);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);
    }

    // -----------------------------------------------------------------------
    // The writer (§4, ADR-e64dfaafd578)
    // -----------------------------------------------------------------------

    /// Every awkward form the criterion names, in one file: comments on their
    /// own line and after a value, blank lines, verifiers out of alphabetical
    /// order, a quoted `run`, and a verifier carrying no `timeout`.
    ///
    /// One fixture rather than one per property, deliberately: a writer that
    /// survives each awkwardness alone and mangles them together would pass
    /// six narrow tests, and that combination is what a real `config.yml` is.
    const FIXTURE: &str = "\
# The configuration of this repository, reviewed like code.
schema: 1

context_budget: 4000   # tokens, not lines
claim_ttl_max: 2h

verifiers:
  # Out of alphabetical order on purpose: a serializer would sort these.
  fmt-check:
    run: \"cargo fmt --check\"
  cargo-test:
    run: cargo test --workspace -q
    timeout: 30m

roles:
  agent:
    can: [context, claim]
    cannot: [delete]
identities: {}
";

    fn edit(text: &str, key: &str, value: Option<&str>) -> Result<String> {
        let k = resolve_key(key)?;
        let mut lines = split_lines(text);
        match value {
            Some(v) => write_key(&mut lines, &k, v)?,
            None => unset_key(&mut lines, &k)?,
        }
        Ok(join_lines(&lines))
    }

    fn read(text: &str, key: &str) -> Result<Value> {
        read_key(&split_lines(text), &resolve_key(key)?)
    }

    /// `after` with the lines that moved put back as they were.
    ///
    /// This is the assertion the criterion asks for, and it is stronger than
    /// comparing the two texts: it proves that *nothing else* moved, including
    /// the bytes a diff would show as unchanged context.
    fn splice_back(before: &str, after: &str, expected_moves: usize) -> String {
        let b = split_lines(before);
        let mut a = split_lines(after);
        assert_eq!(
            a.len(),
            b.len(),
            "the line count changed, so this was not a value replacement"
        );
        let moved: Vec<usize> = (0..a.len()).filter(|&i| a[i] != b[i]).collect();
        assert_eq!(moved.len(), expected_moves, "lines that moved: {moved:?}");
        for i in moved {
            a[i] = b[i].clone();
        }
        join_lines(&a)
    }

    /// `verify::definition_hash` for every verifier the file declares.
    fn hashes(text: &str) -> BTreeMap<String, String> {
        parse(text, p())
            .unwrap()
            .verifiers
            .iter()
            .map(|(n, v)| (n.clone(), crate::verify::definition_hash(v)))
            .collect()
    }

    /// The surgery moves nothing it was not asked to (ADR-e64dfaafd578):
    /// `inputs` written and removed again gives the file back byte for byte.
    #[test]
    fn inputs_are_written_and_removed_surgically() {
        let text = edit(
            FIXTURE,
            "verifiers.cargo-test.inputs",
            Some("[src/a.rs, '**/*.toml']"),
        )
        .unwrap();
        assert!(
            text.contains("    timeout: 30m\n    inputs: [\"src/a.rs\", \"**/*.toml\"]\n\nroles:"),
            "{text}"
        );
        assert!(parse(&text, p()).is_ok());
        assert_eq!(
            read(&text, "verifiers.cargo-test.inputs").unwrap(),
            Value::Set("[\"src/a.rs\", \"**/*.toml\"]".to_string())
        );
        assert_eq!(
            edit(&text, "verifiers.cargo-test.inputs", None).unwrap(),
            FIXTURE
        );

        // A block sequence is read as the list it is, and replaced by the
        // flow form, its items with it.
        let block = FIXTURE.replace(
            "    timeout: 30m\n",
            "    timeout: 30m\n    inputs:\n      - src/**\n      - Cargo.toml\n",
        );
        assert_eq!(
            read(&block, "verifiers.cargo-test.inputs").unwrap(),
            Value::Set("[\"src/**\", \"Cargo.toml\"]".to_string())
        );
        let flowed = edit(&block, "verifiers.cargo-test.inputs", Some("[x]")).unwrap();
        assert_eq!(
            flowed,
            FIXTURE.replace(
                "    timeout: 30m\n",
                "    timeout: 30m\n    inputs: [\"x\"]\n"
            )
        );

        // Nothing declares a verifier but `run`.
        let err = edit(FIXTURE, "verifiers.nope.inputs", Some("[x]")).unwrap_err();
        assert_eq!(err.code, ExitCode::Prerequisite);
        let err = edit(FIXTURE, "verifiers.cargo-test.inputs", Some("src/**")).unwrap_err();
        assert!(err.message.contains("flow sequence"), "{}", err.message);
    }

    /// ADR-7224dea611c1, refused where the file is parsed: every verb loads
    /// it through here, so a refusal here is one no verb can step around.
    #[test]
    fn inputs_are_validated_and_env_is_unknown_when_the_file_is_parsed() {
        let verifier = |body: &str| format!("schema: 1\nverifiers:\n  t:\n    run: x\n{body}");
        let refused = |body: &str| {
            parse(&verifier(body), p())
                .map(|_| ())
                .expect_err(body)
                .message
        };

        let ok = parse(&verifier("    inputs: [\"src/**\"]\n"), p()).unwrap();
        let t = &ok.verifiers["t"];
        assert_eq!(t.inputs.as_deref(), Some(&["src/**".to_string()][..]));

        // Every key the withdrawn block carried, and the block itself.
        for body in [
            "    env: {}\n",
            "    env:\n      platform: linux/amd64\n",
            "    env:\n      vars:\n        LANG: C\n",
            "    env:\n      network: none\n",
        ] {
            let m = refused(body);
            assert!(
                m.contains("verifiers.t") && m.contains("unknown field `env`"),
                "{body}: {m}"
            );
        }
        let m = refused("    inputs: []\n");
        assert!(m.contains("verifiers.t.inputs"), "{m}");
        let m = refused("    inputs: [\"src/[\"]\n");
        assert!(m.contains("not a glob"), "{m}");
    }

    /// A peer is a key the parser knows, and the strictness beside it is
    /// untouched (§7, ADR-a1de673043b4).
    #[test]
    fn peers_are_read_and_an_unknown_key_is_still_refused() {
        let cfg = parse("schema: 1\npeers:\n  core: ../core\n  web: /srv/web\n", p()).unwrap();
        assert_eq!(cfg.peers.get("core").map(String::as_str), Some("../core"));
        assert_eq!(cfg.peers.get("web").map(String::as_str), Some("/srv/web"));

        // The key next door, misspelled, is still a refusal and not a shrug.
        let err = parse("schema: 1\npeerz:\n  core: ../core\n", p()).unwrap_err();
        assert!(err.message.contains("peerz"), "{}", err.message);

        // No peers is no peers, not an error.
        assert!(parse("schema: 1\n", p()).unwrap().peers.is_empty());
    }

    /// Declaring, redeclaring and removing a peer, each moving the line named
    /// and the parent of it and nothing else (§4).
    #[test]
    fn declaring_a_peer_touches_the_line_and_its_parent_only() {
        // Into a file that never carried the key.
        let after = edit(FIXTURE, "peers.core", Some("../core")).unwrap();
        assert!(
            after.starts_with(FIXTURE),
            "the file was rewritten: {after}"
        );
        assert!(after.ends_with("peers:\n  core: ../core\n"), "{after}");
        assert_eq!(
            read(&after, "peers.core").unwrap(),
            Value::Set("../core".into())
        );

        // Into the block that now exists, and the value replaced in place.
        let two = edit(&after, "peers.web", Some("../web")).unwrap();
        assert_eq!(two.matches("peers:").count(), 1, "{two}");
        let moved = edit(&two, "peers.core", Some("../elsewhere")).unwrap();
        assert_eq!(splice_back(&two, &moved, 1), two);

        // And out again, the last one collapsing the mapping to the empty form
        // `init` writes rather than to a key with no children.
        let one = edit(&moved, "peers.web", None).unwrap();
        assert!(one.contains("core: ../elsewhere"), "{one}");
        let none = edit(&one, "peers.core", None).unwrap();
        assert!(none.contains("peers: {}"), "{none}");
        assert!(parse(&none, p()).is_ok(), "{none}");
        assert_eq!(read(&none, "peers.core").unwrap(), Value::Unset);

        // `peers: {}` promotes to a block mapping, the counterpart of the above.
        let back = edit(&none, "peers.core", Some("../core")).unwrap();
        assert!(!back.contains("peers: {}"), "{back}");
        assert!(parse(&back, p()).is_ok(), "{back}");
    }

    /// A peer nobody could name from a scope is refused where the caller can
    /// still type another name (§7).
    #[test]
    fn a_peer_name_no_scope_could_spell_is_refused_by_name() {
        for name in ["peers.c", "peers.a b", "peers.core!"] {
            let err = resolve_key(name).unwrap_err();
            assert_eq!(err.code, ExitCode::Generic, "{name}");
            assert!(err.message.contains("peer name"), "{name}: {}", err.message);
        }
        // The mapping itself is not a value, and the refusal says what to type.
        let err = resolve_key("peers").unwrap_err();
        assert!(err.hint.is_some_and(|h| h.contains("peers.<name>")));
        assert!(resolve_key("peers.core").is_ok());
    }

    #[test]
    fn a_file_read_and_written_back_unedited_is_byte_identical() {
        // The floor everything else stands on: if the split and the join are
        // not each other's inverse, no assertion below means anything.
        for text in [
            FIXTURE,
            &FIXTURE.replace('\n', "\r\n"),
            "schema: 1",           // no trailing newline
            "\n\n# only comments", // no key at all
            "",
        ] {
            assert_eq!(join_lines(&split_lines(text)), text);
        }
    }

    #[test]
    fn writing_one_key_moves_one_line_and_leaves_every_other_byte() {
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert!(after.contains("claim_ttl_max: 4h"));
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // A value followed by a comment keeps the comment, and the column it
        // sat at: the caller asked for the value, not for the line.
        let after = edit(FIXTURE, "context_budget", Some("9000")).unwrap();
        assert!(
            after.contains("context_budget: 9000   # tokens, not lines"),
            "{after}"
        );
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // Nested, and the two neighbours of the edited line are untouched.
        let after = edit(FIXTURE, "verifiers.cargo-test.timeout", Some("45m")).unwrap();
        assert!(after.contains("    timeout: 45m"), "{after}");
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);
    }

    #[test]
    fn the_quoting_style_of_the_edited_key_survives_the_edit() {
        // `fmt-check` carries a double-quoted run. Rewriting it plain would be
        // a correct YAML value and a gratuitous restyle of a reviewed file.
        let after = edit(
            FIXTURE,
            "verifiers.fmt-check.run",
            Some("cargo fmt -- --check"),
        )
        .unwrap();
        assert!(after.contains("run: \"cargo fmt -- --check\""), "{after}");
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // And an unquoted one stays unquoted where it safely can.
        let after = edit(FIXTURE, "verifiers.cargo-test.run", Some("cargo test -q")).unwrap();
        assert!(after.contains("run: cargo test -q"), "{after}");
    }

    #[test]
    fn a_value_that_would_change_meaning_unquoted_is_quoted() {
        // A branch called `no` is a string, and YAML plain-resolves it to
        // false. The parser would then refuse the file, which is a refusal
        // nobody could act on -- so it is written as what it is.
        let after = edit(FIXTURE, "default_branch", Some("no")).unwrap();
        assert!(after.contains("default_branch: \"no\""), "{after}");
        assert_eq!(parse(&after, p()).unwrap().default_branch.unwrap(), "no");

        // A number stays a number: quoting `context_budget` would store a
        // string the parser refuses.
        let after = edit(FIXTURE, "context_budget", Some("12000")).unwrap();
        assert_eq!(parse(&after, p()).unwrap().context_budget, 12000);

        // A run holding a `#` is not truncated into a comment.
        let after = edit(
            FIXTURE,
            "verifiers.cargo-test.run",
            Some("sh -c 'echo # x'"),
        )
        .unwrap();
        assert_eq!(
            parse(&after, p())
                .unwrap()
                .verifier("cargo-test")
                .unwrap()
                .run,
            "sh -c 'echo # x'"
        );
    }

    #[test]
    fn no_default_is_ever_materialised() {
        // The trap this whole design exists for. A serializer would write
        // `default_branch`, a `timeout` onto `fmt-check`, and `{}` for every
        // empty map -- turning "follows the tool" into "pinned here", so the
        // day a default moves the repository silently holds the old value.
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert!(
            !after.contains("default_branch"),
            "a key the file did not carry was written: {after}"
        );
        let fmt_block = after
            .split("fmt-check:")
            .nth(1)
            .unwrap()
            .split("cargo-test:")
            .next()
            .unwrap();
        assert!(
            !fmt_block.contains("timeout"),
            "fmt-check acquired a timeout it never declared: {fmt_block}"
        );
        // And the parser still resolves the defaults, which is the point: the
        // value is in effect without being written down.
        let cfg = parse(&after, p()).unwrap();
        assert_eq!(cfg.default_branch, None);
        assert_eq!(
            cfg.verifier("fmt-check").unwrap().timeout,
            Duration::from_secs(600)
        );
    }

    #[test]
    fn the_definition_hash_of_every_verifier_the_write_did_not_name_is_unchanged() {
        let before = hashes(FIXTURE);

        // A write elsewhere in the file moves nothing.
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert_eq!(hashes(&after), before);

        // A write to one verifier moves that one and no other.
        let after = edit(FIXTURE, "verifiers.cargo-test.timeout", Some("45m")).unwrap();
        let now = hashes(&after);
        assert_eq!(now["fmt-check"], before["fmt-check"]);
        assert_ne!(now["cargo-test"], before["cargo-test"]);

        // Declaring a new verifier disturbs neither of the two already there.
        let after = edit(FIXTURE, "verifiers.audit.run", Some("cargo audit")).unwrap();
        let now = hashes(&after);
        assert_eq!(now["fmt-check"], before["fmt-check"]);
        assert_eq!(now["cargo-test"], before["cargo-test"]);
    }

    #[test]
    fn quoting_alone_never_moves_a_definition_hash() {
        // The plausible trap, and the wrong one. The hash is taken over the
        // resolved values, so re-quoting a run and respelling a timeout in
        // another unit disturb no historical proof -- which is why those forms
        // are edited rather than refused, and why block scalars are not.
        let quoted = "schema: 1\nverifiers:\n  t:\n    run: \"cargo test\"\n    timeout: 600s\n";
        let plain = "schema: 1\nverifiers:\n  t:\n    run: cargo test\n    timeout: 10m\n";
        assert_eq!(hashes(quoted), hashes(plain));
    }

    #[test]
    fn a_block_or_folded_run_is_refused_by_name_and_never_rewritten() {
        for marker in ["|", ">"] {
            let text =
                format!("schema: 1\nverifiers:\n  ci:\n    run: {marker}\n      cargo test\n");
            let err = edit(&text, "verifiers.ci.run", Some("cargo test -q")).unwrap_err();
            assert_eq!(err.code, ExitCode::Generic);
            assert!(
                err.message.contains("verifiers.ci.run"),
                "the refusal must name the key: {}",
                err.message
            );
            assert!(err.hint.is_some(), "a refusal with no way out");

            // Reading is refused for the same reason: there is no honest
            // one-line rendering of it to print.
            assert!(read(&text, "verifiers.ci.run").is_err());
        }
    }

    #[test]
    fn setting_a_key_that_was_absent_and_unsetting_it_returns_the_file() {
        // Both shapes of insertion: a top-level scalar appended at the end,
        // and a whole verifier block added inside another block.
        for (key, value) in [
            ("default_branch", "main"),
            ("verifiers.audit.run", "cargo audit"),
        ] {
            let with = edit(FIXTURE, key, Some(value)).unwrap();
            assert_ne!(with, FIXTURE, "{key} was not written at all");
            parse(&with, p()).unwrap_or_else(|e| panic!("{key}: {}", e.message));
            let back = edit(&with, unset_target(key), None).unwrap();
            assert_eq!(back, FIXTURE, "{key} did not come back out cleanly");
        }
    }

    /// A verifier is removed as a whole, which is the counterpart of declaring
    /// one by writing its `run`.
    fn unset_target(key: &str) -> &str {
        match key {
            "verifiers.audit.run" => "verifiers.audit",
            other => other,
        }
    }

    #[test]
    fn a_new_verifier_lands_at_the_end_of_the_block_with_the_indentation_it_finds() {
        let after = edit(FIXTURE, "verifiers.audit.run", Some("cargo audit")).unwrap();
        assert!(
            after.contains("  audit:\n    run: cargo audit\n"),
            "{after}"
        );
        // At the end of `verifiers`, not at the end of the file: `roles` and
        // `identities` still follow it, and in that order.
        let at = |needle: &str| after.find(needle).unwrap();
        assert!(at("audit:") > at("cargo-test:"));
        assert!(at("audit:") < at("roles:"));

        // Adding a field to a verifier that has none lands inside its block.
        let after = edit(FIXTURE, "verifiers.fmt-check.timeout", Some("5m")).unwrap();
        assert!(
            after.contains("    run: \"cargo fmt --check\"\n    timeout: 5m\n"),
            "{after}"
        );
        assert_eq!(
            parse(&after, p())
                .unwrap()
                .verifier("fmt-check")
                .unwrap()
                .timeout,
            Duration::from_secs(300)
        );
    }

    #[test]
    fn the_empty_flow_mapping_init_writes_can_receive_a_verifier_and_give_it_back() {
        // Without the promotion, the file `ank init` produces could never
        // receive a verifier at all -- and it is the file every new repository
        // starts from.
        let base = default_yaml();
        let with = edit(&base, "verifiers.cargo-test.run", Some("cargo test")).unwrap();
        let cfg = parse(&with, p()).unwrap();
        assert_eq!(cfg.verifier("cargo-test").unwrap().run, "cargo test");
        assert!(
            with.contains("verifiers:\n  cargo-test:\n    run: cargo test\n"),
            "{with}"
        );
        // `roles` was directly below `verifiers: {}` and is untouched.
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);

        // And the demotion is its exact inverse: `verifiers:` with no children
        // is not an empty map but a parse error, so the key goes back to `{}`.
        let back = edit(&with, "verifiers.cargo-test", None).unwrap();
        assert_eq!(back, base);
        parse(&back, p()).unwrap();
    }

    #[test]
    fn a_crlf_checkout_keeps_its_terminators_on_every_line() {
        // §3 forbids ank to *write* CRLF and says nothing about rewriting one
        // in place. A writer that normalised would rewrite every line of the
        // file to change one key, which is the opposite of what this is for.
        let crlf = FIXTURE.replace('\n', "\r\n");
        let after = edit(&crlf, "claim_ttl_max", Some("4h")).unwrap();
        assert!(!after.contains("\n\n"), "an LF slipped in: {after:?}");
        assert_eq!(after.matches("\r\n").count(), crlf.matches("\r\n").count());
        assert_eq!(splice_back(&crlf, &after, 1), crlf);

        // A key appended to such a file arrives with the file's terminator,
        // not with the one this platform would have used.
        let after = edit(&crlf, "default_branch", Some("main")).unwrap();
        assert!(after.ends_with("default_branch: main\r\n"), "{after:?}");
        assert_eq!(edit(&after, "default_branch", None).unwrap(), crlf);
    }

    #[test]
    fn a_file_with_no_trailing_newline_is_terminated_before_a_key_is_appended() {
        // Otherwise the appended key joins the last line and the file means
        // something else entirely.
        let text = "schema: 1\nclaim_ttl_max: 2h";
        let after = edit(text, "default_branch", Some("main")).unwrap();
        assert_eq!(
            after,
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n"
        );

        // Removing the last line keeps the terminator of the one above it:
        // that byte was in the file, and the caller named the key below it.
        let text = "schema: 1\ndefault_branch: main";
        assert_eq!(edit(text, "default_branch", None).unwrap(), "schema: 1\n");

        // A key removed from the middle leaves the ending as it found it.
        let text = "schema: 1\ndefault_branch: main\nclaim_ttl_max: 2h";
        assert_eq!(
            edit(text, "default_branch", None).unwrap(),
            "schema: 1\nclaim_ttl_max: 2h"
        );
    }

    #[test]
    fn reading_marks_a_resolved_default_and_names_an_absence() {
        assert_eq!(read(FIXTURE, "claim_ttl_max").unwrap().display(), "2h");
        assert_eq!(
            read(FIXTURE, "verifiers.fmt-check.timeout")
                .unwrap()
                .display(),
            "10m (default)",
            "the timeout in effect is the tool's, and the reader has to be told which"
        );
        assert_eq!(
            read(FIXTURE, "default_branch").unwrap().display(),
            "(unset)"
        );
        // A timeout resolved for a verifier that is not declared would be a
        // value in effect for something that will never run.
        assert_eq!(
            read(FIXTURE, "verifiers.nope.timeout").unwrap(),
            Value::Unset
        );
        assert_eq!(
            read(FIXTURE, "verifiers.fmt-check.run").unwrap(),
            Value::Set("cargo fmt --check".to_string()),
            "a quoted value reads unquoted"
        );
    }

    #[test]
    fn the_key_set_is_closed_and_every_refusal_names_what_it_refused() {
        // Unknown: the set it does know, and nothing written.
        let err = edit(FIXTURE, "budget_context", Some("10")).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("budget_context"), "{}", err.message);
        let hint = err.hint.unwrap();
        for key in KEYS {
            assert!(hint.contains(key), "{key} missing from: {hint}");
        }

        // Structured, and known to the parser: refused by name rather than
        // guessed at, which is a different message from "no such key".
        for key in ["roles", "roles.agent.can", "identities"] {
            let err = edit(FIXTURE, key, Some("x")).unwrap_err();
            assert!(err.message.contains("structured"), "{}", err.message);
        }

        // A whole verifier is addressable by --unset alone.
        let err = read(FIXTURE, "verifiers.cargo-test").unwrap_err();
        assert!(err.hint.unwrap().contains("verifiers.cargo-test.run"));
        assert!(edit(FIXTURE, "verifiers.cargo-test", Some("x")).is_err());
        assert!(edit(FIXTURE, "verifiers.cargo-test", None).is_ok());

        // A timeout cannot declare a verifier: it would name one that has no
        // command to run, and the file would not parse.
        let err = edit(FIXTURE, "verifiers.nope.timeout", Some("5m")).unwrap_err();
        assert_eq!(err.code, ExitCode::Prerequisite);
        assert_eq!(
            err.hint.as_deref(),
            Some("ank config verifiers.nope.run \"<command>\"")
        );

        // And `run` cannot be removed on its own, for the same reason.
        let err = edit(FIXTURE, "verifiers.cargo-test.run", None).unwrap_err();
        assert_eq!(
            err.hint.as_deref(),
            Some("ank config --unset verifiers.cargo-test")
        );
    }

    #[test]
    fn unsetting_a_key_the_file_does_not_carry_changes_nothing() {
        assert_eq!(edit(FIXTURE, "default_branch", None).unwrap(), FIXTURE);
        assert_eq!(edit(FIXTURE, "verifiers.nope", None).unwrap(), FIXTURE);
        assert_eq!(
            edit(FIXTURE, "verifiers.fmt-check.timeout", None).unwrap(),
            FIXTURE
        );
    }

    #[test]
    fn a_quoted_key_is_read_as_a_key_and_not_scanned_through() {
        // `identities` carries `"marie@laptop": human`, and a scan stopping at
        // the first colon would find one inside the quotes.
        assert_eq!(key_of("  \"a: b\": value").unwrap().0, "a: b".to_string());
        assert_eq!(key_of("run: cargo test").unwrap().0, "run".to_string());
        // YAML requires the space, so this declares no key.
        assert!(key_of("run:cargo").is_none());
        assert!(key_of("- item").is_none());
        assert!(key_of("verifiers:").is_some());
    }

    /// `weight:` is read whole, a counter left out takes the default `init`
    /// writes, and a third counter is refused by name (ADR-467ce7e9cda1).
    #[test]
    fn the_weight_is_two_counters_with_the_defaults_init_writes() {
        let base = "schema: 1\n";
        assert_eq!(parse(base, p()).unwrap().weight, Weight::default());
        assert_eq!(
            parse(&default_yaml(), p()).unwrap().weight,
            Weight {
                hot_files: 3000,
                plane_bytes: 4_000_000
            }
        );
        let partial = parse(&format!("{base}weight:\n  hot_files: 12\n"), p()).unwrap();
        assert_eq!(partial.weight.hot_files, 12);
        assert_eq!(partial.weight.plane_bytes, 4_000_000);
        let e = parse(&format!("{base}weight:\n  bytes: 12\n"), p()).unwrap_err();
        assert!(e.message.contains("bytes"), "{}", e.message);
    }

    /// Written through `ank config` as numbers, where the block exists and
    /// where it does not; read back with the default when absent.
    #[test]
    fn a_weight_counter_is_written_as_a_number() {
        let after = edit(&default_yaml(), "weight.hot_files", Some("12")).unwrap();
        assert!(after.contains("\n  hot_files: 12\n"), "{after}");
        assert_eq!(parse(&after, p()).unwrap().weight.hot_files, 12);

        let after = edit("schema: 1\n", "weight.plane_bytes", Some("500")).unwrap();
        assert_eq!(after, "schema: 1\nweight:\n  plane_bytes: 500\n");
        assert_eq!(parse(&after, p()).unwrap().weight.plane_bytes, 500);

        assert!(resolve_key("weight.bytes").is_err());
        assert!(resolve_key("weight").is_err());
    }
}

/// The parser held to the table `docs/config-keys.md` is rendered from
/// (ADR-2b62b9a1fe67): every key serde accepts is a row, every row is a key
/// serde accepts, and an absent key resolves to the default the row prints.
#[cfg(test)]
mod reference_table {
    use super::*;
    use ank_core::config::{Absent, KEYS};
    use serde::de::{self, Visitor};
    use std::collections::BTreeSet;

    /// A deserializer that answers nothing and records the field names a
    /// struct asks it for: what serde accepts, read off the parser itself.
    struct FieldNames<'a>(&'a mut Vec<&'static str>);

    impl<'de> de::Deserializer<'de> for FieldNames<'_> {
        type Error = de::value::Error;

        fn deserialize_any<V: Visitor<'de>>(
            self,
            _: V,
        ) -> std::result::Result<V::Value, Self::Error> {
            Err(de::Error::custom("field names only"))
        }

        fn deserialize_struct<V: Visitor<'de>>(
            self,
            _: &'static str,
            fields: &'static [&'static str],
            _: V,
        ) -> std::result::Result<V::Value, Self::Error> {
            self.0.extend(fields);
            Err(de::Error::custom("field names only"))
        }

        serde::forward_to_deserialize_any! {
            bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
            bytes byte_buf option unit unit_struct newtype_struct seq tuple
            tuple_struct map enum identifier ignored_any
        }
    }

    fn fields_of<'de, T: Deserialize<'de>>() -> Vec<&'static str> {
        let mut names = Vec::new();
        let _ = T::deserialize(FieldNames(&mut names));
        names
    }

    #[test]
    fn the_table_names_every_key_the_parser_accepts_and_no_other() {
        let mut accepted = BTreeSet::new();
        for top in fields_of::<ConfigFile>() {
            match top {
                "peers" => {
                    accepted.insert("peers.<name>".to_string());
                }
                "identities" => {
                    accepted.insert("identities.<identity>".to_string());
                }
                "verifiers" => {
                    for f in fields_of::<VerifierFile>() {
                        accepted.insert(format!("verifiers.<name>.{f}"));
                    }
                }
                "roles" => accepted.extend(
                    fields_of::<Role>()
                        .into_iter()
                        .map(|f| format!("roles.<name>.{f}")),
                ),
                "weight" => accepted.extend(
                    fields_of::<Weight>()
                        .into_iter()
                        .map(|f| format!("weight.{f}")),
                ),
                "proof" => accepted.extend(
                    fields_of::<ProofFile>()
                        .into_iter()
                        .map(|f| format!("proof.{f}")),
                ),
                "guard" => accepted.extend(
                    fields_of::<GuardFile>()
                        .into_iter()
                        .map(|f| format!("guard.{f}")),
                ),
                "signers" => {
                    assert_eq!(fields_of::<PrincipalFile>(), vec!["keys"]);
                    accepted.extend(
                        fields_of::<SignerKeyFile>()
                            .into_iter()
                            .map(|f| format!("signers.<principal>.keys[].{f}")),
                    );
                }
                scalar => {
                    accepted.insert(scalar.to_string());
                }
            }
        }
        let table: BTreeSet<String> = KEYS.iter().map(|k| k.path.to_string()).collect();
        assert_eq!(accepted.len(), 23, "{accepted:?}");
        assert_eq!(table, accepted);
        // What `ank config` addresses is a subset of the table, never a key
        // of its own.
        for key in super::KEYS {
            assert!(table.contains(*key), "ank config addresses '{key}'");
        }
    }

    fn row(path: &str) -> Absent {
        KEYS.iter()
            .find(|k| k.path == path)
            .unwrap_or_else(|| panic!("no row '{path}'"))
            .default
    }

    fn dur(a: Absent) -> Duration {
        match a {
            Absent::Text(t) => parse_duration(t).unwrap(),
            other => panic!("{other:?} is not a duration"),
        }
    }

    #[test]
    fn an_absent_key_resolves_to_the_default_the_table_prints() {
        let p = Path::new("config.yml");
        let cfg = parse("schema: 1\nverifiers:\n  t:\n    run: \"true\"\n", p).unwrap();
        assert_eq!(row("schema"), Absent::Required);
        assert!(parse("context_budget: 1\n", p).is_err());
        assert_eq!(
            row("context_budget"),
            Absent::Number(cfg.context_budget as u64)
        );
        assert_eq!(dur(row("claim_ttl_max")), cfg.claim_ttl_max);
        assert_eq!(dur(row("claim_ttl_default")), cfg.claim_ttl_default);
        assert_eq!(row("default_branch"), Absent::None);
        assert_eq!(cfg.default_branch, None);
        assert_eq!(row("peers.<name>"), Absent::None);
        assert!(cfg.peers.is_empty());
        assert_eq!(row("verifiers.<name>.run"), Absent::Required);
        assert!(parse("schema: 1\nverifiers:\n  t:\n    timeout: 1m\n", p).is_err());
        assert_eq!(
            dur(row("verifiers.<name>.timeout")),
            cfg.verifiers["t"].timeout
        );
        assert_eq!(row("verifiers.<name>.default"), Absent::Text("false"));
        assert_eq!(row("verifiers.<name>.inputs"), Absent::None);
        assert_eq!(cfg.verifiers["t"].inputs, None);
        assert!(cfg.default_verifiers.is_empty());
        let roles = parse("schema: 1\nroles:\n  r: {}\n", p).unwrap().roles;
        assert_eq!(row("roles.<name>.can"), Absent::Text("[]"));
        assert_eq!(row("roles.<name>.cannot"), Absent::Text("[]"));
        assert_eq!(
            roles["r"],
            Role {
                can: vec![],
                cannot: vec![]
            }
        );
        assert_eq!(row("identities.<identity>"), Absent::None);
        assert!(cfg.identities.is_empty());
        assert_eq!(
            row("weight.hot_files"),
            Absent::Number(cfg.weight.hot_files)
        );
        assert_eq!(
            row("weight.plane_bytes"),
            Absent::Number(cfg.weight.plane_bytes)
        );
        assert_eq!(dur(row("proof.reuse_max_age")), cfg.reuse_max_age);
        assert_eq!(row("verifier_jobs"), Absent::None);
        assert_eq!(cfg.verifier_jobs, None);
        assert!(cfg.verifier_jobs() >= 1);
        assert_eq!(row("guard.corpus"), Absent::Text("deny"));
        assert_eq!(row("guard.edit"), Absent::Text("warn"));
        assert_eq!(
            cfg.guard,
            Guard {
                corpus: GuardCorpus::Deny,
                edit: GuardEdit::Warn
            }
        );
    }
}
