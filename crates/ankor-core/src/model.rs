use crate::error::{Error, Result};
use crate::id::EntityId;
use serde::{Deserialize, Serialize};

/// Version de format supportee par ce crate. Un fichier d'un autre schema
/// est refuse proprement (jamais de rupture silencieuse).
pub const SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// Statuts et transitions
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

    /// Transitions legales de la machine a etats des taches.
    /// `InProgress -> InProgress` est la reprise apres expiration de TTL.
    /// `InProgress -> Open` est `release`. `-> Closed` est l'abandon
    /// ratifie (`ankor close`, identite humaine) ; `Done` et `Closed`
    /// sont terminaux.
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

    /// `Accepted -> Superseded` est la seule ecriture legale sur un ADR
    /// accepte, effectuee par l'`accept` de l'ADR remplacant.
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
// Preuves
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

    /// La ligne de partage de la hierarchie de confiance :
    /// une assertion n'ancre rien, elle est marquee faible par `check`.
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
    /// Hash du contenu des fichiers du scope au moment de l'execution
    /// (`scope/<hash>`). Present pour les preuves locales.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tree: Option<String>,
    /// Hash du done_criteria gele, inscrit par `done`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub criteria: Option<String>,
    /// Definition du verificateur execute : `<nom>@<hash>`. Ancre ce qui a
    /// reellement tourne, independamment de l'etat courant de config.yml.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verifier: Option<String>,
}

// ---------------------------------------------------------------------------
// Entites
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: EntityId,
    pub slug: Option<String>,
    pub title: String,
    /// Horodatage ISO 8601 de l'acte de creation, immuable. Rend l'ordre
    /// des taches deterministe sans dependre de git.
    pub created: String,
    pub status: TaskStatus,
    pub scope: Vec<String>,
    pub blocked_by: Vec<EntityId>,
    pub done_criteria: Option<String>,
    pub criteria_by: Option<CriteriaBy>,
    pub verify: Vec<String>,
    pub proof: Vec<Proof>,
    pub schema: u32,
    pub version: u64,
    /// Corps markdown, verbatim, section `## Log` incluse.
    pub body: String,
}

impl Task {
    /// `blocked` est derive, jamais saisi : bloquee ssi au moins un
    /// `blocked_by` n'est pas `done`. Le resolveur est fourni par l'appelant
    /// (l'index, en pratique). Une reference inconnue est une erreur, pas un
    /// deblocage silencieux.
    pub fn active_blockers<'a, F>(&'a self, status_of: F) -> Result<Vec<&'a EntityId>>
    where
        F: Fn(&EntityId) -> Option<TaskStatus>,
    {
        let mut active = Vec::new();
        for b in &self.blocked_by {
            match status_of(b) {
                None => return Err(Error::UnknownReference(b.to_string())),
                // Seul `done` debloque. `closed` ne debloque pas : le
                // travail n'a pas ete fait, `check` remonte le cas.
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
    pub status: AdrStatus,
    pub scope: Vec<String>,
    pub constraint: String,
    pub see: Option<String>,
    pub supersedes: Option<EntityId>,
    /// Commit signe de ratification (pose par `accept`).
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
