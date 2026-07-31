use thiserror::Error;

/// Structured parser errors. The CLI turns them into self-correcting
/// messages; this crate never formats advice, it names the cause.
#[derive(Debug, Error)]
pub enum Error {
    #[error("missing frontmatter: the file must start with '---'")]
    MissingFrontmatter,

    /// The one diagnostic in this crate that carries a command, and it does so
    /// because the cause *is* a git configuration: nothing about the file's
    /// content can be corrected, and a reader told only "CRLF line endings"
    /// would go and edit the file that git is about to convert back on the
    /// next checkout. Naming the cause here means naming git.
    ///
    /// Never returned by [`crate::parse_entity`], which reads CRLF and
    /// normalises it (§3). This is what `check` reports for a file that parses
    /// but is not in canonical form.
    #[error("CRLF line endings: the format is LF only; git is converting on checkout, fix with 'git config core.autocrlf input'")]
    CrlfLineEndings,

    #[error("invalid YAML: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("invalid identifier: {0}")]
    InvalidId(String),

    #[error("the 'type' field ({field_type}) does not match the id prefix ({id})")]
    TypeMismatch { id: String, field_type: String },

    #[error("empty scope: an entity without a scope is invisible, refused at creation")]
    EmptyScope,

    #[error("invalid glob in scope: {0}")]
    InvalidGlob(String),

    #[error("unknown schema {found} (supported: {supported}); migrate or update the tool")]
    UnknownSchema { found: u32, supported: u32 },

    #[error("criteria_by present without done_criteria")]
    CriteriaByWithoutCriteria,

    #[error("ambiguous prefix '{prefix}': {candidates:?}")]
    AmbiguousPrefix {
        prefix: String,
        candidates: Vec<String>,
    },

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("prefix too short '{0}' (minimum 4 characters)")]
    PrefixTooShort(String),

    #[error("unknown reference in blocked_by: {0}")]
    UnknownReference(String),

    #[error("illegal transition: {from} -> {to}")]
    IllegalTransition { from: String, to: String },
}

pub type Result<T> = std::result::Result<T, Error>;
