use crate::error::{Error, Result};
use crate::id::EntityId;
use serde::{Deserialize, Serialize};

/// Format version this crate **writes**, and the newest it reads.
pub const SCHEMA_VERSION: u32 = 2;

/// Oldest format version this crate reads.
///
/// A range rather than a single version, and the asymmetry is the point (§3).
/// Reading older is a promise the format keeps: a corpus is never migrated by a
/// tool that refuses to read it, so every field introduced after version 1 is
/// optional at parse time and its absence means "written before this existed".
/// Reading newer is refused, because the fields this tool does not know about
/// are exactly the ones it would silently drop on the next rewrite.
///
/// Refusing on the *version* is also the only diagnosis that helps. The
/// frontmatter denies unknown fields, so a reader limited to its own version
/// reports a newer file as `unknown field 'author'` while the file plainly
/// declares the schema that explains it — and sends the reader hunting for a
/// typo.
pub const MIN_SCHEMA: u32 = 1;

// ---------------------------------------------------------------------------
// Statuses and transitions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Open,
    InProgress,
    Done,
    Closed,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Done => "done",
            TaskStatus::Closed => "closed",
        }
    }

    /// Legal transitions of the task state machine.
    /// `InProgress -> InProgress` is pickup after TTL expiry.
    /// `InProgress -> Open` is `release`. `-> Closed` is ratified
    /// abandonment (`ank close`, human identity); `Done` and `Closed` are
    /// terminal.
    pub fn transition_allowed(self, to: TaskStatus) -> bool {
        use TaskStatus::*;
        matches!(
            (self, to),
            (Open, InProgress)
                | (InProgress, Done)
                | (InProgress, Open)
                | (InProgress, InProgress)
                | (Open, Closed)
                | (InProgress, Closed)
        )
    }

    pub fn check_transition(self, to: TaskStatus) -> Result<()> {
        if self.transition_allowed(to) {
            Ok(())
        } else {
            Err(Error::IllegalTransition {
                from: self.as_str().to_string(),
                to: to.as_str().to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Superseded,
}

impl AdrStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AdrStatus::Proposed => "proposed",
            AdrStatus::Accepted => "accepted",
            AdrStatus::Superseded => "superseded",
        }
    }

    /// `Accepted -> Superseded` is the only legal write on an accepted ADR,
    /// performed by the `accept` of the ADR that replaces it.
    pub fn transition_allowed(self, to: AdrStatus) -> bool {
        use AdrStatus::*;
        matches!((self, to), (Proposed, Accepted) | (Accepted, Superseded))
    }

    pub fn check_transition(self, to: AdrStatus) -> Result<()> {
        if self.transition_allowed(to) {
            Ok(())
        } else {
            Err(Error::IllegalTransition {
                from: self.as_str().to_string(),
                to: to.as_str().to_string(),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriteriaBy {
    Creator,
    Claimer,
}

impl CriteriaBy {
    pub fn as_str(self) -> &'static str {
        match self {
            CriteriaBy::Creator => "creator",
            CriteriaBy::Claimer => "claimer",
        }
    }
}

// ---------------------------------------------------------------------------
// Proofs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProofType {
    Test,
    Commit,
    HumanReview,
    Assertion,
}

impl ProofType {
    pub fn as_str(self) -> &'static str {
        match self {
            ProofType::Test => "test",
            ProofType::Commit => "commit",
            ProofType::HumanReview => "human-review",
            ProofType::Assertion => "assertion",
        }
    }

    /// The dividing line of the trust hierarchy: an assertion anchors
    /// nothing, and is marked weak by `check`.
    pub fn is_weak(self) -> bool {
        matches!(self, ProofType::Assertion | ProofType::HumanReview)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proof {
    #[serde(rename = "type")]
    pub proof_type: ProofType,
    #[serde(rename = "ref")]
    pub reference: String,
    /// Hash of the scope files' content at execution time (`scope/<hash>`).
    /// Present for local proofs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<String>,
    /// Hash of the frozen done_criteria, recorded by `done`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<String>,
    /// Definition of the verifier that ran: `<name>@<hash>`. Anchors what
    /// actually ran, independently of the current state of config.yml.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
}

// ---------------------------------------------------------------------------
// Entities
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: EntityId,
    pub slug: Option<String>,
    pub title: String,
    /// ISO 8601 timestamp of the act of creation, immutable. Makes task
    /// ordering deterministic without depending on git.
    pub created: String,
    /// The identity that ran `new`, as `$ANK_AGENT` resolves it (§8).
    ///
    /// `None` means the entity predates the field, never that nobody wrote it:
    /// schema 1 had no such thing, and the author of a file that already exists
    /// cannot be recovered — `git log` would say who committed it, and
    /// ADR-b8884edcebe3 forbids porcelain. The signals that need it skip those
    /// entities and `check` says so once for the corpus.
    pub author: Option<String>,
    pub status: TaskStatus,
    pub scope: Vec<String>,
    pub blocked_by: Vec<EntityId>,
    pub done_criteria: Option<String>,
    pub criteria_by: Option<CriteriaBy>,
    pub verify: Vec<String>,
    pub proof: Vec<Proof>,
    pub schema: u32,
    pub version: u64,
    /// Markdown body, verbatim, `## Log` section included.
    pub body: String,
}

impl Task {
    /// `blocked` is derived, never entered: blocked if and only if at least
    /// one `blocked_by` is not `done`. The resolver is supplied by the caller
    /// (the index, in practice). An unknown reference is an error, not a
    /// silent unblocking.
    pub fn active_blockers<'a, F>(&'a self, status_of: F) -> Result<Vec<&'a EntityId>>
    where
        F: Fn(&EntityId) -> Option<TaskStatus>,
    {
        let mut active = Vec::new();
        for b in &self.blocked_by {
            match status_of(b) {
                None => return Err(Error::UnknownReference(b.to_string())),
                // Only `done` unblocks. `closed` does not: the work was not
                // carried out, and `check` reports the case.
                Some(TaskStatus::Done) => {}
                Some(_) => active.push(b),
            }
        }
        Ok(active)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Adr {
    pub id: EntityId,
    pub slug: Option<String>,
    pub title: String,
    pub created: String,
    /// The identity that ran `new`. `None` means the entity predates the field.
    pub author: Option<String>,
    pub status: AdrStatus,
    pub scope: Vec<String>,
    pub constraint: String,
    pub see: Option<String>,
    pub supersedes: Option<EntityId>,
    /// Signed ratification commit (set by `accept`).
    pub ratified: Option<String>,
    pub schema: u32,
    pub version: u64,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Task(Task),
    Adr(Adr),
}

impl Entity {
    pub fn id(&self) -> &EntityId {
        match self {
            Entity::Task(t) => &t.id,
            Entity::Adr(a) => &a.id,
        }
    }

    pub fn scope(&self) -> &[String] {
        match self {
            Entity::Task(t) => &t.scope,
            Entity::Adr(a) => &a.scope,
        }
    }
}
