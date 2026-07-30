//! Loading `.ank/config.yml` (§4, §8).
//!
//! The file is controlled by the repository, and therefore reviewed like any
//! code change — that is what makes it acceptable for `done` to run its
//! verifiers. An unknown `schema` is cleanly refused rather than misread:
//! that is the counterpart of "the format is the specification".

use crate::cli::CliError;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

pub const SUPPORTED_SCHEMA: u32 = 1;
pub const DEFAULT_CONTEXT_BUDGET: usize = 8000;
pub const DEFAULT_CLAIM_TTL_MAX: &str = "2h";
pub const DEFAULT_VERIFIER_TIMEOUT: &str = "10m";

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    schema: u32,
    #[serde(default = "default_budget")]
    context_budget: usize,
    #[serde(default = "default_ttl_max")]
    claim_ttl_max: String,
    #[serde(default)]
    verifiers: BTreeMap<String, VerifierFile>,
    #[serde(default)]
    roles: BTreeMap<String, Role>,
    #[serde(default)]
    identities: BTreeMap<String, String>,
}

fn default_budget() -> usize {
    DEFAULT_CONTEXT_BUDGET
}

fn default_ttl_max() -> String {
    DEFAULT_CLAIM_TTL_MAX.to_string()
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub schema: u32,
    pub context_budget: usize,
    pub claim_ttl_max: Duration,
    pub verifiers: BTreeMap<String, Verifier>,
    pub roles: BTreeMap<String, Role>,
    pub identities: BTreeMap<String, String>,
}

impl Config {
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
    let raw: ConfigFile = serde_yaml::from_str(text)
        .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;

    if raw.schema != SUPPORTED_SCHEMA {
        return Err(CliError::new(
            1,
            format!(
                "{}: unknown schema {} (supported: {SUPPORTED_SCHEMA})",
                path.display(),
                raw.schema
            ),
        )
        .with_hint("update ank, or fix the schema field"));
    }

    let dur = |v: &str, field: &str| -> Result<Duration> {
        parse_duration(v).map_err(|e| CliError::new(1, format!("{}: {field}: {e}", path.display())))
    };

    let claim_ttl_max = dur(&raw.claim_ttl_max, "claim_ttl_max")?;
    let mut verifiers = BTreeMap::new();
    for (name, v) in raw.verifiers {
        let timeout = dur(&v.timeout, &format!("verifiers.{name}.timeout"))?;
        verifiers.insert(
            name,
            Verifier {
                run: v.run,
                timeout,
            },
        );
    }

    Ok(Config {
        schema: raw.schema,
        context_budget: raw.context_budget,
        claim_ttl_max,
        verifiers,
        roles: raw.roles,
        identities: raw.identities,
    })
}

pub fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::new(1, format!("{} not found", path.display())).with_hint("ank init")
        } else {
            CliError::new(1, format!("{}: {e}", path.display()))
        }
    })?;
    parse(&text, path)
}

/// Content written by `ank init`. The canonical form of the file, aligned with
/// the one this repository uses to dogfood itself.
pub fn default_yaml() -> String {
    "\
schema: 1
context_budget: 8000
claim_ttl_max: 2h
verifiers: {}
roles:
  agent:
    can: [context, find, claim, log, done, new:task, new:adr:proposed]
    cannot: [adr:accept, adr:edit-constraint, task:close, delete]
  human:
    can: [\"*\"]
identities: {}
"
    .to_string()
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

    #[test]
    fn an_unknown_schema_is_refused_with_the_next_step() {
        let err = parse("schema: 2\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("unknown schema 2"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    #[test]
    fn an_unknown_field_is_refused_rather_than_ignored() {
        let err = parse("schema: 1\nbudget_context: 10\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
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
        assert!(cfg.verifier("cargo-test").is_some());
        assert!(cfg.verifier("check-repo").is_some());
    }

    #[test]
    fn the_default_yaml_written_by_init_reads_back() {
        let cfg = parse(&default_yaml(), p()).unwrap();
        assert_eq!(cfg.schema, SUPPORTED_SCHEMA);
        assert_eq!(cfg.context_budget, DEFAULT_CONTEXT_BUDGET);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);
    }
}
