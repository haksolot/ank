use thiserror::Error;

/// Erreurs structurees du parseur. Le CLI les traduira en messages
/// auto-correctifs ; le crate ne formate jamais de conseil, il nomme la cause.
#[derive(Debug, Error)]
pub enum Error {
    #[error("frontmatter absent : le fichier doit commencer par '---'")]
    MissingFrontmatter,

    #[error("YAML invalide : {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("identifiant invalide : {0}")]
    InvalidId(String),

    #[error("le champ 'type' ({field_type}) ne correspond pas au prefixe de l'id ({id})")]
    TypeMismatch { id: String, field_type: String },

    #[error("scope vide : une entite sans scope est invisible, refuse a la creation")]
    EmptyScope,

    #[error("glob invalide dans scope : {0}")]
    InvalidGlob(String),

    #[error("schema {found} inconnu (supporte : {supported}) ; migrer ou mettre a jour l'outil")]
    UnknownSchema { found: u32, supported: u32 },

    #[error("criteria_by present sans done_criteria")]
    CriteriaByWithoutCriteria,

    #[error("prefixe ambigu '{prefix}' : {candidates:?}")]
    AmbiguousPrefix {
        prefix: String,
        candidates: Vec<String>,
    },

    #[error("entite introuvable : {0}")]
    NotFound(String),

    #[error("prefixe trop court '{0}' (minimum 4 caracteres)")]
    PrefixTooShort(String),

    #[error("reference inconnue dans blocked_by : {0}")]
    UnknownReference(String),

    #[error("transition illegale : {from} -> {to}")]
    IllegalTransition { from: String, to: String },
}

pub type Result<T> = std::result::Result<T, Error>;
