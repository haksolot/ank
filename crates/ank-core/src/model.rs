use crate::error::{Error, Result};
use crate::id::EntityId;
use serde::{Deserialize, Serialize};

/// The values [`Log::records`] is known to take.
///
/// A vocabulary and not an enum, deliberately: the field parses as a free
/// string so that an entry written by a newer build is read rather than
/// refused, and this list is what `check` compares against to say that a word
/// is unknown to *this* build (ADR-3877fef1d662).
pub const RECORDS_KINDS: &[&str] = &[RECORDS_EDIT];

/// The word an entry carries when it records a change of content made outside
/// a status transition (ADR-16813b3bcf37).
///
/// Named once and written from one place, so the vocabulary above and the verb
/// that writes into it cannot drift apart by a typo.
pub const RECORDS_EDIT: &str = "edit";

/// Format version this crate **writes**, and the newest it reads.
///
/// 3 carried two changes: the log leaving the entity body, and [`Verified`]
/// with the typed-actor convention it names (§3). The flat layout of the same
/// revision carried no bump — it moves files rather than fields, and a reader
/// that finds the file finds every field it already knew.
///
/// The log is what made that bump necessary, and it is the case this constant
/// exists for: a reader that does not know the log has left the body shows an
/// empty history for a task that has one, silently.
///
/// 4 carries one change, [`Log::records`], and it is the same case again. An
/// entry marked as machinery is kept out of the work trace a reader reads
/// (ADR-16813b3bcf37); a reader that does not know the field drops it on the
/// next rewrite, and the entry silently rejoins the trace it was written to
/// stay out of. Silently is the word that decides it, here as at 3.
///
/// **The cost is paid by binaries already distributed**, and it is worth
/// stating where the number lives. Every entity a verb writes carries this
/// value, so a corpus edited by a build at 4 becomes progressively unreadable
/// to a build at 3, one entity at a time, with `schema_ahead` warning once and
/// every listing then answering as if those entities were not there. That is
/// the designed behaviour of this constant and not a side effect, but it is a
/// real price and the field that triggers it is used by one kind.
pub const SCHEMA_VERSION: u32 = 4;

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

/// A spec's lifecycle is an ADR's, and it is the same type rather than a copy
/// of it (§3). A spec is `proposed` while it is a draft, promoted by `accept`,
/// and revised by supersession — the three values and the two legal
/// transitions of [`AdrStatus`], stated once. Declaring a second enum with the
/// same variants would be the per-layer restatement ADR-c9f9d0d6f05d exists to
/// remove, and the two would eventually disagree.
pub type SpecStatus = AdrStatus;

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

/// The route by which a proof arrived (§4, ADR-b6b69053a47b).
///
/// The type says what the reference points at; this says who put it there, and
/// the second is what the trust rests on. A run reference is the strongest
/// thing in the hierarchy when a pipeline wrote it and the weakest when
/// somebody typed it — before this existed the file said the same thing in both
/// cases, so `--proof test:<anything>` was recorded as strong, unchecked, and
/// silenced the one finding designed to catch a completion nothing external
/// anchors.
///
/// **A closed set, and the absent field is not a member of it.** `None` means
/// the entry was written before the distinction existed and is read exactly as
/// it was read then; a spelling outside the set is a parse error, because a
/// writer inventing a fourth route would be inventing one the trust hierarchy
/// has no rule for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProofVia {
    /// Ank ran a verifier `config.yml` declares, and this is its own statement.
    Verifier,
    /// The entry reached the task on `refs/ank/proof/<id>`, written by whoever
    /// held the pipeline (§7). A third-party statement.
    Attested,
    /// A caller passed it to `done --proof` or `attest --proof`. Recorded as
    /// given, and never refused: the CLI is not a gatekeeper.
    Submitted,
}

impl ProofVia {
    pub fn as_str(self) -> &'static str {
        match self {
            ProofVia::Verifier => "verifier",
            ProofVia::Attested => "attested",
            ProofVia::Submitted => "submitted",
        }
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
    /// How the entry arrived. `None` means it predates the field (§3).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<ProofVia>,
}

impl Proof {
    /// Whether this entry anchors the completion in something outside the
    /// agent's reach — the question `check` asks before reporting `done with no
    /// test proof` (§4, ADR-b6b69053a47b).
    ///
    /// **Read on the route and never on the type alone.** That is the whole
    /// change: a `test` reference somebody typed at a keyboard used to answer
    /// yes here, which made the finding silenceable by the very act it exists
    /// to catch.
    ///
    /// **`None` answers yes**, and that is a decision rather than an oversight.
    /// An entry written before the field cannot say which route it took, and a
    /// rule that read the silence as `submitted` would redden every completion
    /// recorded before it landed — twenty-one of them in this repository's own
    /// corpus, ten typed by hand and eleven attested by the pipeline, with
    /// nothing in the files to tell them apart. A rule that fires on a corpus
    /// that did nothing is a rule everybody turns off.
    pub fn anchors_externally(&self) -> bool {
        self.proof_type == ProofType::Test && self.via != Some(ProofVia::Submitted)
    }
}

// ---------------------------------------------------------------------------
// Readings
// ---------------------------------------------------------------------------

/// A reading: somebody read this entity and stands behind it (§3).
///
/// `proof` anchors that something *ran*; this anchors that somebody *read*, and
/// a corpus written by agents needs the second more than a corpus written by
/// people did. Optional on every kind and required by no verb — a trust field
/// that were required would be filled in to make the tool stop complaining, at
/// which point it records nothing.
///
/// `by` follows the actor convention (`human:<id>`, `<producer>/<version>`,
/// `process:<id>`) and a value that does not is a `check` finding, **never a
/// parse error**: the corpus is not migrated by a rule it predates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verified {
    /// A typed actor.
    pub by: String,
    /// ISO 8601 instant, in UTC.
    pub at: String,
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
    /// ADR-9307e5d214a7 forbids porcelain. The signals that need it skip those
    /// entities and `check` says so once for the corpus.
    pub author: Option<String>,
    pub status: TaskStatus,
    pub scope: Vec<String>,
    pub blocked_by: Vec<EntityId>,
    pub done_criteria: Option<String>,
    pub criteria_by: Option<CriteriaBy>,
    pub verify: Vec<String>,
    pub proof: Vec<Proof>,
    /// Readings, optional and empty by default (§3).
    pub verified: Vec<Verified>,
    pub schema: u32,
    pub version: u64,
    /// Markdown body, verbatim.
    ///
    /// Since schema 3 the log is not in here: it is a file of its own, at
    /// `.ank/log/<ID>.md`. A schema 1 or schema 2 body may still carry a
    /// `## Log` section, which is read where it is and never written there.
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
    /// Readings, optional and empty by default (§3).
    pub verified: Vec<Verified>,
    pub schema: u32,
    pub version: u64,
    pub body: String,
}

/// A specification: one document, whole (§3).
///
/// **It declares no `constraint`, and that absence is what justifies the
/// kind.** A spec describes, an ADR binds; an entity that did both would be an
/// ADR with an unbounded constraint, which restates the ceiling problem rather
/// than solving it. Nothing in the refusal machinery reads a spec, and a rule
/// that must bind is still an ADR.
///
/// What the kind buys is that the document moves the way every other decision
/// does: a status, a supersession, a ratification anchor and a version. The
/// anchor differs from an ADR's in one respect only — `ratified` holds the hash
/// of the **body** and `scope`, because there is no narrower field carrying the
/// authority, so revising an accepted specification is a supersession.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub id: EntityId,
    pub slug: Option<String>,
    pub title: String,
    pub created: String,
    /// The identity that ran `new`. `None` means the entity predates the field.
    pub author: Option<String>,
    pub status: SpecStatus,
    pub scope: Vec<String>,
    /// The documents and decisions this one rests on (§3, ADR-c88f99e1c16e).
    ///
    /// A declared field and never a sentence: a specification cut into documents
    /// drifts unless the coherence between them is verified, and verifying it
    /// means the citation has to sit somewhere a parser can reach. What a
    /// reference resolving to nothing, to a draft or to a superseded document
    /// costs is `check`'s business — the kind that may be named included. Here
    /// it is a list of identifiers, on the reading `about` already gets: a
    /// corpus is not made unreadable by one citation.
    pub references: Vec<EntityId>,
    pub supersedes: Option<EntityId>,
    /// Hash of the document and scope at acceptance (set by `accept`).
    pub ratified: Option<String>,
    /// Readings, optional and empty by default (§3).
    pub verified: Vec<Verified>,
    pub schema: u32,
    pub version: u64,
    /// The document itself.
    pub body: String,
}

/// A log entry, as an entity (§3, ADR-25f977377fa0).
///
/// Distinct from [`crate::log::LogEntry`], which is the *rendering*: a dash,
/// the timestamp, the identity, an em dash, the message. That grammar is what a
/// reader sees and no longer what is stored — here the instant is `created`,
/// the identity is `author`, the message is `title`, and the entity the entry
/// is about is [`Log::about`].
///
/// **Written once and never modified.** A correction is a new entry naming the
/// one it corrects, which is why the kind carries no `status`: an entry has
/// nothing to transition to, and a field with one legal value records nothing.
/// `version` stays, and earns its place by what it falsifies — an entry above 1
/// has been rewritten, and the format says it should not have been.
#[derive(Debug, Clone, PartialEq)]
pub struct Log {
    pub id: EntityId,
    pub slug: Option<String>,
    /// The message. No field is added for it: every entity already carries the
    /// field a lister prints, and a second one is the same sentence twice.
    pub title: String,
    pub created: String,
    /// The identity that wrote the entry. `None` means the entity predates the
    /// field, as on every other kind.
    pub author: Option<String>,
    /// The subject's scope, written when the entry is written. An entry appears
    /// wherever what it is about appears, and it records the scope as it stood
    /// rather than tracking it.
    pub scope: Vec<String>,
    /// The entity the entry is about, of any kind. This is what the previous
    /// shape computed from the id instead: an address became a query, and in
    /// exchange an entry is indexed and reachable like anything else.
    pub about: EntityId,
    /// The rank of this entry among the entries about [`Log::about`], from 0.
    ///
    /// **A timestamp is not an order.** `created` has one-second resolution and
    /// writing an entry costs a few hundred milliseconds, so several entries
    /// inside one second is the ordinary case; with no other key the order
    /// falls to the identifier, which is a hash of the act of creation and
    /// carries none. An append-only file gave insertion order for free and a
    /// set of files does not, so the rank is a field (§3).
    ///
    /// **Not a clock and not a lock.** Two writers who cannot see each other
    /// produce the same value, which is the honest answer rather than a defect:
    /// they were concurrent. `created` separates them when their instants
    /// differ and the identifier settles what is left.
    pub seq: u64,
    /// What this entry records, when it records something other than work.
    ///
    /// **Absent means a work entry**, which is what `ank log` means to a reader
    /// and what every entry written before this field existed is. A value names
    /// machinery: `edit`, written by the verbs that change an entity's content
    /// outside a status transition (ADR-16813b3bcf37).
    ///
    /// **A free string at parse time, a vocabulary at `check` time**, on the
    /// terms ADR-3877fef1d662 sets for a typed actor: a value the tool does not
    /// know is a finding and never a parse error, because a corpus written by a
    /// newer build must stay readable rather than become unparseable.
    pub records: Option<String>,
    /// Readings, optional and empty by default (§3).
    pub verified: Vec<Verified>,
    pub schema: u32,
    pub version: u64,
    /// Anything the line cannot hold.
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entity {
    Task(Task),
    Adr(Adr),
    Spec(Spec),
    Log(Log),
}

impl Entity {
    pub fn id(&self) -> &EntityId {
        match self {
            Entity::Task(t) => &t.id,
            Entity::Adr(a) => &a.id,
            Entity::Spec(s) => &s.id,
            Entity::Log(l) => &l.id,
        }
    }

    pub fn scope(&self) -> &[String] {
        match self {
            Entity::Task(t) => &t.scope,
            Entity::Adr(a) => &a.scope,
            Entity::Spec(s) => &s.scope,
            Entity::Log(l) => &l.scope,
        }
    }

    /// The format version the file declares. Every kind carries one — it is in
    /// the common base of §3 — so a caller asking the question asks it of an
    /// entity rather than of each kind in turn.
    pub fn schema(&self) -> u32 {
        match self {
            Entity::Task(t) => t.schema,
            Entity::Adr(a) => a.schema,
            Entity::Spec(s) => s.schema,
            Entity::Log(l) => l.schema,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Entity::Task(t) => &t.title,
            Entity::Adr(a) => &a.title,
            Entity::Spec(s) => &s.title,
            Entity::Log(l) => &l.title,
        }
    }

    pub fn created(&self) -> &str {
        match self {
            Entity::Task(t) => &t.created,
            Entity::Adr(a) => &a.created,
            Entity::Spec(s) => &s.created,
            Entity::Log(l) => &l.created,
        }
    }

    pub fn slug(&self) -> Option<&str> {
        match self {
            Entity::Task(t) => t.slug.as_deref(),
            Entity::Adr(a) => a.slug.as_deref(),
            Entity::Spec(s) => s.slug.as_deref(),
            Entity::Log(l) => l.slug.as_deref(),
        }
    }

    /// `None` means the entity predates the field, on every kind alike.
    pub fn author(&self) -> Option<&str> {
        match self {
            Entity::Task(t) => t.author.as_deref(),
            Entity::Adr(a) => a.author.as_deref(),
            Entity::Spec(s) => s.author.as_deref(),
            Entity::Log(l) => l.author.as_deref(),
        }
    }

    pub fn verified(&self) -> &[Verified] {
        match self {
            Entity::Task(t) => &t.verified,
            Entity::Adr(a) => &a.verified,
            Entity::Spec(s) => &s.verified,
            Entity::Log(l) => &l.verified,
        }
    }

    pub fn version(&self) -> u64 {
        match self {
            Entity::Task(t) => t.version,
            Entity::Adr(a) => a.version,
            Entity::Spec(s) => s.version,
            Entity::Log(l) => l.version,
        }
    }

    /// The compare-and-swap counter of §7, written by the store on every write.
    pub fn set_version(&mut self, v: u64) {
        match self {
            Entity::Task(t) => t.version = v,
            Entity::Adr(a) => a.version = v,
            Entity::Spec(s) => s.version = v,
            Entity::Log(l) => l.version = v,
        }
    }

    /// The body, for a caller that rewrites it. Reading it is
    /// [`crate::registry::Fields::body`], which every kind already answers.
    pub fn body_mut(&mut self) -> &mut String {
        match self {
            Entity::Task(t) => &mut t.body,
            Entity::Adr(a) => &mut a.body,
            Entity::Spec(s) => &mut s.body,
            Entity::Log(l) => &mut l.body,
        }
    }
}
