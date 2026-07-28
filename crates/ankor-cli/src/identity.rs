//! Identite de l'appelant (§8).
//!
//! `$ANKOR_AGENT` est pose par l'agent lui-meme. Ce n'est donc pas une
//! preuve, et le modele ne fait pas semblant du contraire : le repli est
//! `<user>@<hostname>`, le role par defaut est `agent` — le moindre
//! privilege — et la seule autorite reelle vient de la signature.

use std::process::Command;

pub const ENV_AGENT: &str = "ANKOR_AGENT";

/// Identite courante. Toujours resolue, jamais absente : une identite
/// inconnue recoit le role `agent`, ce qui est le comportement voulu.
pub fn resolve() -> String {
    match std::env::var(ENV_AGENT) {
        Ok(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => format!("{}@{}", user(), hostname()),
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
    // Dernier recours : le binaire. Absent sur certaines images minimales,
    // d'ou le repli final plutot qu'une erreur — l'identite n'est pas une
    // preuve, elle ne doit jamais bloquer.
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

/// Nom court : le FQDN d'une machine de CI est long, bruyant dans un log de
/// tache, et sans valeur discriminante supplementaire.
fn short_host(h: &str) -> String {
    h.split('.').next().unwrap_or(h).to_ascii_lowercase()
}

/// Role declare pour une identite. Toute identite absente de la table est
/// un `agent` : le moindre privilege par defaut (§8).
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
    fn le_fqdn_est_reduit_a_son_nom_court() {
        assert_eq!(short_host("runner-4.ci.example.com"), "runner-4");
        assert_eq!(short_host("LAPTOP-9F2"), "laptop-9f2");
        assert_eq!(short_host("localhost"), "localhost");
    }

    #[test]
    fn le_repli_a_la_forme_user_arobase_host() {
        // On ne peut pas retirer ANKOR_AGENT de facon fiable en parallele
        // d'autres tests ; on teste la forme du repli, pas l'environnement.
        let repli = format!("{}@{}", user(), hostname());
        assert!(repli.contains('@'), "{repli}");
        let (u, h) = repli.split_once('@').unwrap();
        assert!(!u.is_empty(), "utilisateur vide");
        assert!(!h.is_empty(), "hote vide");
        assert!(!h.contains('.'), "l'hote doit etre court : {h}");
    }

    #[test]
    fn identite_inconnue_recoit_le_role_agent() {
        let mut ids = BTreeMap::new();
        ids.insert("marie@laptop".to_string(), "human".to_string());
        assert_eq!(role_of("marie@laptop", &ids), "human");
        assert_eq!(role_of("codex@host-9", &ids), "agent");
        assert_eq!(role_of("", &ids), "agent");
    }
}
