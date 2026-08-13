//! The caller's identity (§8).
//!
//! `$ANK_AGENT` is set by the agent itself. It is therefore not proof, and the
//! model does not pretend otherwise: the fallback is `<user>@<hostname>`, the
//! default role is `agent` — least privilege — and the only real authority
//! comes from the signature.

use std::process::Command;

pub const ENV_AGENT: &str = "ANK_AGENT";

/// Where the identity in effect came from.
///
/// The two are not interchangeable and only one of them is a trap: a value the
/// caller declared names a session, while the fallback names a machine, so two
/// sessions in one checkout that both forgot the variable are one identity as
/// far as every claim, log entry and signal is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// Read from `$ANK_AGENT`, which the caller set.
    Env,
    /// Nothing was set, so the tool composed `<user>@<hostname>`.
    Fallback,
}

impl Source {
    /// The word `--json` prints, on the terms `config` already set for
    /// `source`: a short token a script matches on, never a sentence.
    pub fn word(self) -> &'static str {
        match self {
            Source::Env => "env",
            Source::Fallback => "fallback",
        }
    }

    /// What the human surface says beside the value. `config` renders a
    /// resolved default as `8000 (default)` and the file's own value bare;
    /// here both sides are named, because the reader who needs the line is the
    /// one who does not yet know which of the two they are looking at.
    pub fn display(self) -> String {
        match self {
            Source::Env => format!("({ENV_AGENT})"),
            Source::Fallback => format!("(fallback, {ENV_AGENT} unset)"),
        }
    }
}

/// The current identity and where it came from. Always resolved, never absent:
/// an unknown identity gets the `agent` role, which is the intended behaviour.
pub fn resolved() -> (String, Source) {
    match std::env::var(ENV_AGENT) {
        Ok(v) if !v.trim().is_empty() => (v.trim().to_string(), Source::Env),
        _ => (format!("{}@{}", user(), hostname()), Source::Fallback),
    }
}

fn first_env(keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Ok(v) = std::env::var(k) {
            let v = v.trim().to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

pub fn user() -> String {
    first_env(&["USERNAME", "USER", "LOGNAME"]).unwrap_or_else(|| "unknown".to_string())
}

pub fn hostname() -> String {
    if let Some(h) = first_env(&["COMPUTERNAME", "HOSTNAME"]) {
        return short_host(&h);
    }
    // Last resort: the binary. Absent from some minimal images, hence the
    // final fallback rather than an error — identity is not proof, so it must
    // never block.
    if let Ok(out) = Command::new("hostname").output() {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return short_host(&s);
            }
        }
    }
    "localhost".to_string()
}

/// Short name: a CI machine's FQDN is long, noisy in a task log, and carries
/// no extra discriminating value.
fn short_host(h: &str) -> String {
    h.split('.').next().unwrap_or(h).to_ascii_lowercase()
}

/// The declared role for an identity. Any identity absent from the table is an
/// `agent`: least privilege by default (§8).
pub fn role_of(identity: &str, identities: &std::collections::BTreeMap<String, String>) -> String {
    identities
        .get(identity)
        .cloned()
        .unwrap_or_else(|| "agent".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn an_fqdn_is_reduced_to_its_short_name() {
        assert_eq!(short_host("runner-4.ci.example.com"), "runner-4");
        assert_eq!(short_host("LAPTOP-9F2"), "laptop-9f2");
        assert_eq!(short_host("localhost"), "localhost");
    }

    #[test]
    fn the_fallback_has_the_user_at_host_shape() {
        // ANK_AGENT cannot be removed reliably while other tests run in
        // parallel; we test the shape of the fallback, not the environment.
        let fallback = format!("{}@{}", user(), hostname());
        assert!(fallback.contains('@'), "{fallback}");
        let (u, h) = fallback.split_once('@').unwrap();
        assert!(!u.is_empty(), "empty user");
        assert!(!h.is_empty(), "empty host");
        assert!(!h.contains('.'), "the host must be short: {h}");
    }

    #[test]
    fn an_unknown_identity_gets_the_agent_role() {
        let mut ids = BTreeMap::new();
        ids.insert("marie@laptop".to_string(), "human".to_string());
        assert_eq!(role_of("marie@laptop", &ids), "human");
        assert_eq!(role_of("codex@host-9", &ids), "agent");
        assert_eq!(role_of("", &ids), "agent");
    }
}
