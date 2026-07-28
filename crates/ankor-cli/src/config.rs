//! Chargement de `.ankor/config.yml` (§4, §8).
//!
//! Le fichier est controle par le repo, donc revu comme n'importe quel
//! changement de code — c'est ce qui rend acceptable que `done` execute ses
//! verificateurs. Un `schema` inconnu est refuse proprement plutot que lu de
//! travers : c'est la contrepartie de « le format est la spec ».

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

/// Durees de la forme `<n><unite>`, unites `s`, `m`, `h`, `d`. Volontairement
/// etroit : `claim_ttl_max: 2h` doit se lire sans documentation, et une
/// grammaire plus riche n'apporterait que des formes a ne pas ecrire.
pub fn parse_duration(text: &str) -> std::result::Result<Duration, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("duree vide".to_string());
    }
    let (digits, unit) = t.split_at(
        t.find(|c: char| !c.is_ascii_digit())
            .unwrap_or_else(|| t.len()),
    );
    if digits.is_empty() {
        return Err(format!("duree '{text}' : chiffre attendu avant l'unite"));
    }
    let n: u64 = digits
        .parse()
        .map_err(|_| format!("duree '{text}' : nombre illisible"))?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        "" => return Err(format!("duree '{text}' : unite manquante (s, m, h, d)")),
        other => {
            return Err(format!(
                "duree '{text}' : unite '{other}' inconnue (s, m, h, d)"
            ))
        }
    };
    Ok(Duration::from_secs(secs))
}

pub fn parse(text: &str, path: &Path) -> Result<Config> {
    let raw: ConfigFile = serde_yaml::from_str(text)
        .map_err(|e| CliError::new(1, format!("{} : {e}", path.display())))?;

    if raw.schema != SUPPORTED_SCHEMA {
        return Err(CliError::new(
            1,
            format!(
                "{} : schema {} inconnu (supporte : {SUPPORTED_SCHEMA})",
                path.display(),
                raw.schema
            ),
        )
        .with_hint("mettre a jour ankor, ou corriger le champ schema"));
    }

    let dur = |v: &str, champ: &str| -> Result<Duration> {
        parse_duration(v)
            .map_err(|e| CliError::new(1, format!("{} : {champ} : {e}", path.display())))
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
            CliError::new(1, format!("{} introuvable", path.display())).with_hint("ankor init")
        } else {
            CliError::new(1, format!("{} : {e}", path.display()))
        }
    })?;
    parse(&text, path)
}

/// Contenu ecrit par `ankor init`. Forme canonique du fichier, alignee sur
/// celle que ce repo utilise pour se dogfooder.
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
        Path::new(".ankor/config.yml")
    }

    #[test]
    fn durees_dans_les_quatre_unites() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(1800));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86_400));
        assert_eq!(parse_duration(" 10m ").unwrap(), Duration::from_secs(600));
    }

    #[test]
    fn durees_invalides_nommees_precisement() {
        assert!(parse_duration("30")
            .unwrap_err()
            .contains("unite manquante"));
        assert!(parse_duration("30w").unwrap_err().contains("inconnue"));
        assert!(parse_duration("h").unwrap_err().contains("chiffre attendu"));
        assert!(parse_duration("").unwrap_err().contains("vide"));
    }

    #[test]
    fn schema_inconnu_refuse_avec_la_suite_a_donner() {
        let err = parse("schema: 2\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("schema 2 inconnu"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    #[test]
    fn champ_inconnu_refuse_plutot_qu_ignore() {
        let err = parse("schema: 1\nbudget_context: 10\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("budget_context"), "{}", err.message);
    }

    #[test]
    fn les_cinq_champs_de_la_spec_sont_lus() {
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
        // Le defaut de timeout est celui de la spec, pas zero.
        assert_eq!(
            cfg.verifier("fmt-check").unwrap().timeout,
            Duration::from_secs(600)
        );
        assert_eq!(cfg.roles["agent"].cannot, vec!["delete".to_string()]);
        assert_eq!(cfg.identities["marie@laptop"], "human");
    }

    #[test]
    fn le_config_de_ce_repo_se_charge() {
        // Dogfooding : la config qui pilote ce repo doit passer le parseur
        // qu'on vient d'ecrire, sinon l'un des deux ment.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.ankor/config.yml")
            .canonicalize()
            .unwrap();
        let cfg = load(&path).unwrap();
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert!(cfg.verifier("cargo-test").is_some());
        assert!(cfg.verifier("check-repo").is_some());
    }

    #[test]
    fn le_yaml_par_defaut_de_init_se_relit() {
        let cfg = parse(&default_yaml(), p()).unwrap();
        assert_eq!(cfg.schema, SUPPORTED_SCHEMA);
        assert_eq!(cfg.context_budget, DEFAULT_CONTEXT_BUDGET);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);
    }
}
