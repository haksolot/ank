use crate::error::{Error, Result};
use crate::registry;
use sha2::{Digest, Sha256};
use std::fmt;

pub const ID_HEX_LEN: usize = 12;
pub const MIN_PREFIX_LEN: usize = 4;

/// A kind, as an index into the registry rather than as a second declaration
/// of it. The name and the prefix are read from [`crate::registry::KINDS`] and
/// are written down nowhere else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EntityKind {
    Task,
    Adr,
    Spec,
    Log,
}

impl EntityKind {
    /// The enum's declaration order **is** the registry's, and that is the one
    /// coupling between them. It is asserted rather than trusted, in
    /// `registry_and_enum_agree` below: a row added to the table without a
    /// variant, or the reverse, fails the suite instead of resolving to the
    /// wrong kind in silence.
    fn spec(self) -> &'static registry::KindSpec {
        &registry::KINDS[self as usize]
    }

    pub fn prefix(self) -> &'static str {
        self.spec().prefix
    }

    pub fn as_str(self) -> &'static str {
        self.spec().name
    }

    /// The kind whose `type` is this string, or `None` if the registry declares
    /// none. The caller turns that `None` into [`Error::UnknownKind`], naming
    /// the kind rather than the first field it happens to carry.
    pub fn from_type_name(name: &str) -> Option<Self> {
        Self::all().find(|k| k.as_str() == name)
    }

    fn all() -> impl Iterator<Item = EntityKind> {
        [
            EntityKind::Task,
            EntityKind::Adr,
            EntityKind::Spec,
            EntityKind::Log,
        ]
        .into_iter()
    }
}

/// Canonical identifier: a prefix the registry declares, then 12 hex characters
/// — `TASK-`, `ADR-`, `SPEC-`, `LOG-` today, and the prefix is read from
/// [`crate::registry::KINDS`] rather than written here.
/// Immutable, derived from the act of creation, never from the content.
///
/// **There is no short form here, and that absence is the decision.** §3 makes
/// the displayed length a function of the corpus — the shortest prefix naming
/// exactly one entity in it — and an identifier does not know the corpus it is
/// about to be printed beside. A `short()` on this type could only ever return
/// a constant, and a constant is precisely the value [`resolve_prefix`] below
/// refuses once two ids share it (TASK-c1f01f301d63). Callers that print ask
/// the corpus instead, through `ank_cli::context::shorts_of`.
/// `Ord` is the tiebreaker of the entry order (§3) and nothing more. It sorts by
/// kind and then by hex, which is arbitrary — an id is a hash of the act of
/// creation and carries no order — and that is exactly what it is for: the last
/// resort under `created` and `seq`, where two entries are genuinely concurrent
/// and any stable answer is the right one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId {
    kind: EntityKind,
    hex: String, // 12 characters, lowercase
}

impl EntityId {
    pub fn parse(s: &str) -> Result<Self> {
        let Some((kind, rest)) =
            EntityKind::all().find_map(|k| s.strip_prefix(k.prefix()).map(|rest| (k, rest)))
        else {
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
    let (kind_filter, hex_prefix) = EntityKind::all()
        .find_map(|k| prefix.strip_prefix(k.prefix()).map(|r| (Some(k), r)))
        .unwrap_or((None, prefix));

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

#[cfg(test)]
mod tests {
    use super::*;

    /// The one coupling between the enum and the table, asserted rather than
    /// trusted. A row added without a variant would make `spec()` index past
    /// the end; a variant added without a row would resolve to the wrong kind
    /// in silence, which is the worse of the two.
    #[test]
    fn registry_and_enum_agree() {
        assert_eq!(
            EntityKind::all().count(),
            registry::KINDS.len(),
            "every kind in the registry needs a variant, and the reverse"
        );
        for (i, kind) in EntityKind::all().enumerate() {
            assert_eq!(kind.as_str(), registry::KINDS[i].name);
            assert_eq!(kind.prefix(), registry::KINDS[i].prefix);
            assert_eq!(EntityKind::from_type_name(kind.as_str()), Some(kind));
        }
        assert_eq!(EntityKind::from_type_name("epic"), None);
    }

    /// The prefix is read from the registry, so an id resolves to a kind
    /// without any of the three former copies of that fact being consulted.
    #[test]
    fn a_prefix_resolves_through_the_registry() {
        for kind in EntityKind::all() {
            let id = format!("{}0123456789ab", kind.prefix());
            assert_eq!(EntityId::parse(&id).unwrap().kind(), kind);
        }
        assert!(matches!(
            EntityId::parse("EPIC-0123456789ab"),
            Err(Error::InvalidId(_))
        ));
    }
}
