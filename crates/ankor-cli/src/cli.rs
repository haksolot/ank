//! Analyse d'arguments et dispatch (§4, §12).
//!
//! L'analyse est faite a la main, sans bibliotheque. La raison n'est pas
//! l'economie d'une dependance mais le controle au caractere pres de deux
//! surfaces lues par des agents : les erreurs auto-correctives, qu'un
//! analyseur generique remplacerait par ses propres messages, et l'aide,
//! dont le cout est paye a chaque appel qui la declenche. La surface etant
//! figee a douze verbes (ADR-2f8a61c04b7d), ce cout ne croit pas.
//!
//! Les cas limites de l'analyse sont l'endroit ou le fait main se plante, et
//! ils ressemblent a des bugs metier une fois en production : ils sont donc
//! tous testes, un test par cas.

use std::collections::BTreeMap;
use std::fmt;

// ---------------------------------------------------------------------------
// Erreur portant son code de sortie
// ---------------------------------------------------------------------------

/// Erreur du CLI. Le code vient de la table du §4 ; `hint` porte la commande
/// exacte a executer ensuite. Jamais d'aide generique : un aller-retour
/// d'erreur bien concu coute moins cher que trois tentatives a l'aveugle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub code: i32,
    pub message: String,
    pub hint: Option<String>,
}

impl CliError {
    pub fn new(code: i32, message: impl Into<String>) -> CliError {
        CliError {
            code,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> CliError {
        self.hint = Some(hint.into());
        self
    }

    /// Rendu terse, type `git status`, sur la sortie d'erreur.
    pub fn render(&self) -> String {
        match &self.hint {
            Some(h) => format!("error[{}]: {}\n  -> {}", self.code, self.message, h),
            None => format!("error[{}]: {}", self.code, self.message),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<crate::store::StoreError> for CliError {
    fn from(e: crate::store::StoreError) -> CliError {
        let code = e.code();
        let hint = e.hint();
        let mut err = CliError::new(code, e.to_string());
        err.hint = hint;
        err
    }
}

pub type Result<T> = std::result::Result<T, CliError>;

// ---------------------------------------------------------------------------
// Description de la surface
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlagSpec {
    pub name: &'static str,
    pub takes_value: bool,
    pub repeatable: bool,
}

const fn flag(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: false,
    }
}

const fn switch(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: false,
        repeatable: false,
    }
}

const fn multi(name: &'static str) -> FlagSpec {
    FlagSpec {
        name,
        takes_value: true,
        repeatable: true,
    }
}

/// Flags globaux, volontairement limites a trois (§4). `--json` est
/// disponible sur toutes les commandes sans exception : la scriptabilite
/// integrale est un invariant, pas une option — d'ou leur ajout mecanique a
/// la surface de chaque commande plutot qu'une declaration par commande,
/// qui laisserait la possibilite d'en oublier une.
pub const GLOBAL_FLAGS: &[FlagSpec] = &[switch("--json"), switch("--quiet"), flag("--repo")];

#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    pub name: &'static str,
    /// Sous-commandes obligatoires, comme `new task` / `new adr`.
    pub subcommands: &'static [&'static str],
    pub max_positionals: usize,
    pub positional_help: &'static str,
    pub flags: &'static [FlagSpec],
    /// Tache qui porte l'implementation, tant qu'elle n'existe pas.
    pub owner_task: Option<&'static str>,
}

/// Les douze verbes du §4, plus `init` (§9). L'ordre est celui de la spec :
/// boucle agent, hors-boucle agent, surface humaine.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "context",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[flag("--limit")],
        owner_task: Some("TASK-d4e5f6a7b8c9"),
    },
    CommandSpec {
        name: "claim",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--criteria"), flag("--ttl")],
        owner_task: Some("TASK-c3d4e5f6a7b8"),
    },
    CommandSpec {
        name: "log",
        subcommands: &[],
        max_positionals: 2,
        positional_help: "[<id>] <message>",
        flags: &[],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "done",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--proof")],
        owner_task: Some("TASK-e5f6a7b8c9d0"),
    },
    CommandSpec {
        name: "release",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<id>]",
        flags: &[flag("--reason")],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "new",
        subcommands: &["task", "adr"],
        max_positionals: 0,
        positional_help: "",
        flags: &[
            flag("--title"),
            multi("--scope"),
            flag("--criteria"),
            multi("--blocked-by"),
            flag("--constraint"),
        ],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "find",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<query>",
        flags: &[flag("--type"), flag("--status"), flag("--scope")],
        owner_task: Some("TASK-f6a7b8c9d0e1"),
    },
    CommandSpec {
        name: "review",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "accept",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "close",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[flag("--reason")],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "check",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "show",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "<id>",
        flags: &[],
        owner_task: Some("TASK-a7b8c9d0e1f2"),
    },
    CommandSpec {
        name: "init",
        subcommands: &[],
        max_positionals: 1,
        positional_help: "[<path>]",
        flags: &[],
        owner_task: None,
    },
];

pub fn spec_of(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|c| c.name == name)
}

fn known_flags(spec: &CommandSpec) -> Vec<&'static str> {
    let mut v: Vec<&'static str> = spec.flags.iter().map(|f| f.name).collect();
    v.extend(GLOBAL_FLAGS.iter().map(|f| f.name));
    v.sort_unstable();
    v
}

fn find_flag(spec: &CommandSpec, name: &str) -> Option<FlagSpec> {
    spec.flags
        .iter()
        .chain(GLOBAL_FLAGS.iter())
        .find(|f| f.name == name)
        .copied()
}

// ---------------------------------------------------------------------------
// Invocation analysee
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub command: &'static str,
    pub subcommand: Option<String>,
    pub positionals: Vec<String>,
    /// Valeurs par flag. Un commutateur porte une liste vide.
    pub flags: BTreeMap<String, Vec<String>>,
}

impl Invocation {
    pub fn has(&self, flag: &str) -> bool {
        self.flags.contains_key(flag)
    }

    pub fn value(&self, flag: &str) -> Option<&str> {
        self.flags.get(flag)?.last().map(|s| s.as_str())
    }

    pub fn values(&self, flag: &str) -> &[String] {
        self.flags.get(flag).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn json(&self) -> bool {
        self.has("--json")
    }

    pub fn quiet(&self) -> bool {
        self.has("--quiet")
    }

    pub fn repo(&self) -> Option<&str> {
        self.value("--repo")
    }
}

// ---------------------------------------------------------------------------
// Analyse
// ---------------------------------------------------------------------------

pub fn parse(argv: &[String]) -> Result<Invocation> {
    let Some(first) = argv.first() else {
        return Err(CliError::new(1, "aucune commande").with_hint("ankor context"));
    };

    let spec = spec_of(first).ok_or_else(|| {
        let noms: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        CliError::new(1, format!("commande inconnue '{first}'"))
            .with_hint(format!("ankor <{}>", noms.join("|")))
    })?;

    let mut rest = &argv[1..];
    let mut subcommand = None;
    if !spec.subcommands.is_empty() {
        let sub = rest.first().ok_or_else(|| {
            CliError::new(1, format!("'{}' attend une sous-commande", spec.name)).with_hint(
                format!("ankor {} <{}>", spec.name, spec.subcommands.join("|")),
            )
        })?;
        if !spec.subcommands.contains(&sub.as_str()) {
            return Err(CliError::new(
                1,
                format!("sous-commande inconnue '{sub}' pour '{}'", spec.name),
            )
            .with_hint(format!(
                "ankor {} <{}>",
                spec.name,
                spec.subcommands.join("|")
            )));
        }
        subcommand = Some(sub.clone());
        rest = &rest[1..];
    }

    let mut positionals: Vec<String> = Vec::new();
    let mut flags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut i = 0usize;
    let mut terminated = false;

    while i < rest.len() {
        let arg = &rest[i];

        if !terminated && arg == "--" {
            // Tout ce qui suit est positionnel, verbatim : c'est la seule
            // facon d'ecrire un message qui commence par un tiret.
            terminated = true;
            i += 1;
            continue;
        }

        if !terminated && arg.starts_with("--") {
            let (name, inline) = match arg.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (arg.clone(), None),
            };

            let Some(fs) = find_flag(spec, &name) else {
                return Err(CliError::new(
                    1,
                    format!("flag inconnu '{name}' pour '{}'", spec.name),
                )
                .with_hint(format!("flags valides : {}", known_flags(spec).join(" "))));
            };

            if !fs.takes_value {
                if inline.is_some() {
                    return Err(CliError::new(1, format!("'{name}' ne prend pas de valeur"))
                        .with_hint(format!("ankor {} {name}", spec.name)));
                }
                flags.entry(name).or_default();
                i += 1;
                continue;
            }

            let value = match inline {
                Some(v) => {
                    i += 1;
                    v
                }
                None => {
                    let v = rest.get(i + 1).ok_or_else(|| {
                        CliError::new(1, format!("'{name}' attend une valeur"))
                            .with_hint(format!("ankor {} {name} <valeur>", spec.name))
                    })?;
                    i += 2;
                    v.clone()
                }
            };

            let slot = flags.entry(name.clone()).or_default();
            if !fs.repeatable {
                slot.clear();
            }
            slot.push(value);
            continue;
        }

        positionals.push(arg.clone());
        i += 1;
    }

    if positionals.len() > spec.max_positionals {
        let surnumeraire = &positionals[spec.max_positionals];
        return Err(CliError::new(
            1,
            format!(
                "argument en trop '{surnumeraire}' : '{}' en accepte {}",
                spec.name, spec.max_positionals
            ),
        )
        .with_hint(usage(spec)));
    }

    Ok(Invocation {
        command: spec.name,
        subcommand,
        positionals,
        flags,
    })
}

pub fn usage(spec: &CommandSpec) -> String {
    let mut s = format!("ankor {}", spec.name);
    if !spec.subcommands.is_empty() {
        s.push_str(&format!(" <{}>", spec.subcommands.join("|")));
    }
    if !spec.positional_help.is_empty() {
        s.push(' ');
        s.push_str(spec.positional_help);
    }
    s
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

fn not_implemented(spec: &CommandSpec) -> CliError {
    let task = spec.owner_task.unwrap_or("TASK-inconnue");
    CliError::new(1, format!("'{}' n'est pas encore implementee", spec.name))
        .with_hint(format!("voir .ankor/tasks/{task}.md"))
}

/// Point d'entree. Rend le code de sortie ; n'appelle jamais `exit` lui-meme,
/// pour rester testable.
pub fn run(argv: &[String], cwd: &std::path::Path, out: &mut dyn std::io::Write) -> i32 {
    match dispatch(argv, cwd, out) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.render());
            err.code
        }
    }
}

/// Socle commun a tous les verbes sauf `init`, qui par definition precede
/// l'existence du repo.
///
/// L'ordre compte et n'est pas arbitraire : on resout le repo avant de
/// verifier git, parce qu'un `--repo` errone doit etre nomme comme tel et
/// non deguise en probleme d'environnement. Cette resolution a lieu **avant**
/// le rejet d'un verbe non implemente : sans cela `--repo` ne serait
/// exerce par aucun chemin reel tant qu'aucun verbe n'existe, et le socle
/// serait teste sans jamais etre atteint.
struct Startup {
    #[allow(dead_code)]
    repo: crate::repo::Repo,
    #[allow(dead_code)]
    config: crate::config::Config,
    #[allow(dead_code)]
    identity: String,
}

fn startup(inv: &Invocation, cwd: &std::path::Path) -> Result<Startup> {
    let repo = crate::repo::resolve(inv.repo(), cwd)?;
    // git est une dependance dure, sa version est verifiee au demarrage
    // (ADR-92b9cda9f6a9).
    crate::git::ensure_usable(&repo.root)?;
    let config = crate::config::load(&repo.config_path())?;
    let identity = crate::identity::resolve();
    Ok(Startup {
        repo,
        config,
        identity,
    })
}

fn dispatch(argv: &[String], cwd: &std::path::Path, out: &mut dyn std::io::Write) -> Result<i32> {
    let inv = parse(argv)?;
    let spec = spec_of(inv.command).expect("spec resolue a l'analyse");

    if inv.command == "init" {
        return crate::init::run(&inv, cwd, out);
    }

    let _startup = startup(&inv, cwd)?;
    Err(not_implemented(spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    fn ok(s: &[&str]) -> Invocation {
        parse(&argv(s)).unwrap_or_else(|e| panic!("{s:?} devait passer : {}", e.render()))
    }

    #[test]
    fn repo_dans_ses_deux_formes() {
        let a = ok(&["check", "--repo=/chemin/vers/projet"]);
        let b = ok(&["check", "--repo", "/chemin/vers/projet"]);
        assert_eq!(a.repo(), Some("/chemin/vers/projet"));
        assert_eq!(b.repo(), Some("/chemin/vers/projet"));
        assert_eq!(a.flags, b.flags);
    }

    #[test]
    fn terminateur_transmet_le_message_intact() {
        // Le cas qui motive le terminateur : un message qui commence par
        // deux tirets et serait sinon lu comme un flag.
        let inv = ok(&["log", "--", "-- message"]);
        assert_eq!(inv.positionals, vec!["-- message".to_string()]);
        assert!(inv.flags.is_empty());

        // Apres le terminateur, plus rien n'est un flag.
        let inv = ok(&["log", "--", "--json"]);
        assert_eq!(inv.positionals, vec!["--json".to_string()]);
        assert!(!inv.json(), "--json apres -- est du texte, pas un flag");

        // Avant le terminateur, les flags restent des flags.
        let inv = ok(&["log", "--json", "--", "--repo"]);
        assert!(inv.json());
        assert_eq!(inv.positionals, vec!["--repo".to_string()]);
    }

    #[test]
    fn flag_inconnu_nomme_les_flags_valides() {
        let err = parse(&argv(&["claim", "--tll", "30m"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("--tll"), "{}", err.message);
        let hint = err.hint.unwrap();
        for attendu in ["--criteria", "--ttl", "--json", "--quiet", "--repo"] {
            assert!(hint.contains(attendu), "{attendu} absent de : {hint}");
        }
    }

    #[test]
    fn valeur_manquante_apres_un_flag_qui_en_attend_une() {
        let err = parse(&argv(&["claim", "8f3a", "--ttl"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("--ttl"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ankor claim --ttl <valeur>"));

        // Le flag global n'est pas un cas particulier.
        let err = parse(&argv(&["check", "--repo"])).unwrap_err();
        assert!(err.message.contains("--repo"), "{}", err.message);
    }

    #[test]
    fn json_accepte_sur_les_douze_commandes_sans_exception() {
        // Invariant de la spec : la scriptabilite integrale n'est pas une
        // option. Le test parcourt la table, donc un verbe ajoute sans
        // --json le ferait echouer.
        for spec in COMMANDS {
            let mut a = vec![spec.name.to_string()];
            if let Some(sub) = spec.subcommands.first() {
                a.push(sub.to_string());
            }
            a.push("--json".to_string());
            let inv = parse(&a)
                .unwrap_or_else(|e| panic!("--json refuse sur {} : {}", spec.name, e.render()));
            assert!(inv.json(), "--json non retenu sur {}", spec.name);
        }
        assert_eq!(COMMANDS.len(), 13, "douze verbes du §4, plus init du §9");
    }

    #[test]
    fn argument_positionnel_surnumeraire_refuse_jamais_ignore() {
        let err = parse(&argv(&["show", "8f3a", "51c2"])).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("51c2"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ankor show <id>"));

        // `log` en accepte deux : [<id>] <message>.
        assert_eq!(ok(&["log", "8f3a", "message"]).positionals.len(), 2);
        assert!(parse(&argv(&["log", "8f3a", "message", "en trop"])).is_err());
    }

    #[test]
    fn commande_inconnue_liste_les_commandes() {
        let err = parse(&argv(&["claimm"])).unwrap_err();
        assert!(err.message.contains("claimm"), "{}", err.message);
        let hint = err.hint.unwrap();
        assert!(hint.contains("claim") && hint.contains("context"), "{hint}");
    }

    #[test]
    fn new_exige_sa_sous_commande() {
        let err = parse(&argv(&["new"])).unwrap_err();
        assert_eq!(err.hint.as_deref(), Some("ankor new <task|adr>"));

        let err = parse(&argv(&["new", "epic"])).unwrap_err();
        assert!(err.message.contains("epic"), "{}", err.message);

        let inv = ok(&["new", "task", "--title", "T", "--scope", "src/**"]);
        assert_eq!(inv.subcommand.as_deref(), Some("task"));
    }

    #[test]
    fn scope_est_repetable_les_autres_non() {
        let inv = ok(&[
            "new", "task", "--scope", "src/**", "--scope", "tests/**", "--title", "A", "--title",
            "B",
        ]);
        assert_eq!(inv.values("--scope"), ["src/**", "tests/**"]);
        // Un flag non repetable garde la derniere valeur, sans accumuler.
        assert_eq!(inv.values("--title"), ["B"]);
    }

    #[test]
    fn commutateur_refuse_une_valeur_collee() {
        let err = parse(&argv(&["check", "--json=oui"])).unwrap_err();
        assert!(
            err.message.contains("ne prend pas de valeur"),
            "{}",
            err.message
        );
    }

    #[test]
    fn les_verbes_non_implementes_nomment_leur_tache() {
        for spec in COMMANDS.iter().filter(|c| c.owner_task.is_some()) {
            let err = not_implemented(spec);
            assert_eq!(err.code, 1);
            let hint = err.hint.unwrap();
            assert!(hint.contains("TASK-"), "{} : {hint}", spec.name);
        }
    }

    #[test]
    fn le_socle_est_resolu_avant_le_rejet_du_verbe_non_implemente() {
        // Sans cet ordre, --repo ne serait exerce par aucun chemin reel
        // tant qu'aucun verbe n'existe : le socle serait teste unitairement
        // sans jamais etre atteint par le binaire.
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", "/chemin/qui/n/existe/pas"]),
            std::path::Path::new("."),
            &mut out,
        );
        assert_eq!(code, 1);

        // Un --repo valide traverse le socle et atteint le rejet du verbe.
        let racine = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", racine.to_str().unwrap()]),
            std::path::Path::new("."),
            &mut out,
        );
        assert_eq!(code, 1, "le verbe check n'est pas encore implemente");
    }

    #[test]
    fn init_ne_passe_pas_par_le_socle() {
        // init precede l'existence du repo : exiger un .ankor/ prealable
        // rendrait la commande inutilisable pour ce qu'elle sert a faire.
        let inv = ok(&["init"]);
        assert_eq!(inv.command, "init");
        assert!(spec_of("init").unwrap().owner_task.is_none());
    }

    #[test]
    fn le_rendu_d_erreur_suit_la_forme_de_la_spec() {
        let err = CliError::new(7, "TASK-51c2 n'a pas de done_criteria")
            .with_hint("ankor claim 51c2 --criteria \"<critere verifiable>\"");
        assert_eq!(
            err.render(),
            "error[7]: TASK-51c2 n'a pas de done_criteria\n  -> ankor claim 51c2 --criteria \"<critere verifiable>\""
        );
        assert_eq!(
            CliError::new(1, "sans suite").render(),
            "error[1]: sans suite"
        );
    }
}
