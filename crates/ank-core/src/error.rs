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

    /// A kind the registry does not declare (§3). Distinct from
    /// [`Error::TypeMismatch`], which is a known kind contradicting its own id,
    /// and from [`Error::InvalidId`], which is a malformed identifier: the
    /// three answer different questions and a reader sent to the wrong one
    /// loses an hour. `priorty:` inside a `task` is a typo; `type: epic` is a
    /// document this tool does not know how to read, and saying so means
    /// naming the kind rather than the first field it happens to carry.
    #[error("unknown kind: {kind}")]
    UnknownKind { kind: String },

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

    /// A line of a log **file** that the grammar does not accept (§3).
    ///
    /// Strict where the legacy `## Log` section is tolerant, and the asymmetry
    /// is the decision: a markdown body may hold anything, so a line under the
    /// heading that is not an entry is skipped and reported by `check`. A file
    /// whose entire content is the log leaves a stray line nothing else it
    /// could be. The diagnostic names which line, because the file grows.
    #[error("malformed log line {line}: expected '- <timestamp> <identity> — <message>'")]
    MalformedLogLine { line: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
