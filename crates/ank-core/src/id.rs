use crate::error::{Error, Result};
use sha2::{Digest, Sha256};
use std::fmt;

pub const ID_HEX_LEN: usize = 12;
pub const MIN_PREFIX_LEN: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Task,
    Adr,
}

impl EntityKind {
    pub fn prefix(self) -> &'static str {
        match self {
            EntityKind::Task => "TASK-",
            EntityKind::Adr => "ADR-",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            EntityKind::Task => "task",
            EntityKind::Adr => "adr",
        }
    }
}

/// Canonical identifier: `TASK-<12 hex>` or `ADR-<12 hex>`.
/// Immutable, derived from the act of creation, never from the content.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityId {
    kind: EntityKind,
    hex: String, // 12 characters, lowercase
}

impl EntityId {
    pub fn parse(s: &str) -> Result<Self> {
        let (kind, rest) = if let Some(r) = s.strip_prefix("TASK-") {
            (EntityKind::Task, r)
        } else if let Some(r) = s.strip_prefix("ADR-") {
            (EntityKind::Adr, r)
        } else {
            return Err(Error::InvalidId(s.to_string()));
        };
        if rest.len() != ID_HEX_LEN || !rest.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(Error::InvalidId(s.to_string()));
        }
        Ok(EntityId {
            kind,
            hex: rest.to_ascii_lowercase(),
        })
    }

    /// Hash of the act of creation: timestamp, identity, title, entropy.
    /// The caller supplies the entropy (this crate depends on no RNG).
    pub fn generate(
        kind: EntityKind,
        timestamp: &str,
        identity: &str,
        title: &str,
        entropy: &[u8],
    ) -> Self {
        let mut h = Sha256::new();
        h.update(timestamp.as_bytes());
        h.update([0]);
        h.update(identity.as_bytes());
        h.update([0]);
        h.update(title.as_bytes());
        h.update([0]);
        h.update(entropy);
        let digest = h.finalize();
        EntityId {
            kind,
            hex: hex::encode(&digest[..ID_HEX_LEN / 2]),
        }
    }

    pub fn kind(&self) -> EntityKind {
        self.kind
    }

    pub fn hex(&self) -> &str {
        &self.hex
    }

    pub fn short(&self) -> String {
        format!("{}{}", self.kind.prefix(), &self.hex[..4])
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.kind.prefix(), self.hex)
    }
}

/// Prefix resolution. Ambiguity is an error: the tool never guesses.
pub fn resolve_prefix<'a, I>(prefix: &str, ids: I) -> Result<&'a EntityId>
where
    I: IntoIterator<Item = &'a EntityId>,
{
    // The type prefix is optional on input ("TASK-8f3a" or "8f3a").
    let (kind_filter, hex_prefix) = if let Some(r) = prefix.strip_prefix("TASK-") {
        (Some(EntityKind::Task), r)
    } else if let Some(r) = prefix.strip_prefix("ADR-") {
        (Some(EntityKind::Adr), r)
    } else {
        (None, prefix)
    };

    let hex_prefix = hex_prefix.to_ascii_lowercase();
    if hex_prefix.len() < MIN_PREFIX_LEN {
        return Err(Error::PrefixTooShort(prefix.to_string()));
    }
    if !hex_prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::InvalidId(prefix.to_string()));
    }

    let matches: Vec<&EntityId> = ids
        .into_iter()
        .filter(|id| kind_filter.map_or(true, |k| id.kind() == k))
        .filter(|id| id.hex().starts_with(&hex_prefix))
        .collect();

    match matches.len() {
        0 => Err(Error::NotFound(prefix.to_string())),
        1 => Ok(matches[0]),
        _ => Err(Error::AmbiguousPrefix {
            prefix: prefix.to_string(),
            candidates: matches.iter().map(|id| id.to_string()).collect(),
        }),
    }
}
