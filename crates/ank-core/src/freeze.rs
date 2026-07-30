//! Hash freeze (§3 of the specification).
//!
//! The CLI is not a gatekeeper: any tool can rewrite a file. A frozen field
//! is therefore anchored by a hash in an artifact the editor does not control
//! (the claim record, the ratification commit), and compared here.
//! Normalisation makes the hash insensitive to editing noise (trailing
//! whitespace, trailing newline) without ever tolerating a change of meaning.

use sha2::{Digest, Sha256};

/// Normalisation: CRLF line endings become LF, trailing whitespace is
/// stripped, trailing blank lines are removed.
pub fn normalize(text: &str) -> String {
    let unified = text.replace("\r\n", "\n");
    let mut lines: Vec<&str> = unified.lines().map(|l| l.trim_end()).collect();
    while lines.last() == Some(&"") {
        lines.pop();
    }
    lines.join("\n")
}

/// Full hash (hex, 64 characters) of the normalised text.
pub fn freeze_hash(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(normalize(text).as_bytes());
    hex::encode(h.finalize())
}

/// Short form, for display and for the claim record (12 hex characters,
/// consistent with the length of identifiers).
pub fn freeze_hash_short(text: &str) -> String {
    freeze_hash(text)[..12].to_string()
}

/// Check a frozen field against its anchoring hash. Accepts the short form
/// or the long form.
pub fn verify_frozen(text: &str, anchor: &str) -> bool {
    let full = freeze_hash(text);
    full == anchor || full.starts_with(anchor) && anchor.len() >= 12
}
