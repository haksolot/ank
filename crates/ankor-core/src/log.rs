//! Section `## Log` du fichier de tache (§3 de la spec).
//!
//! Append-only, une ligne horodatee par entree :
//! `- 2026-07-26T14:02Z claude-code@host-3 — message`
//! Appendre en fin de fichier produit un diff git d'une ligne, ce qui est
//! la propriete que le format existe pour garantir (§12).

pub const LOG_HEADER: &str = "## Log";
const SEPARATOR: &str = " — ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// Horodatage ISO 8601, conserve tel quel (le crate ne reformate pas).
    pub timestamp: String,
    /// Identite `agent@host`.
    pub who: String,
    pub message: String,
}

impl LogEntry {
    pub fn format_line(&self) -> String {
        format!(
            "- {} {}{}{}",
            self.timestamp, self.who, SEPARATOR, self.message
        )
    }

    pub fn parse_line(line: &str) -> Option<LogEntry> {
        let rest = line.strip_prefix("- ")?;
        let (timestamp, rest) = rest.split_once(' ')?;
        let (who, message) = rest.split_once(SEPARATOR)?;
        Some(LogEntry {
            timestamp: timestamp.to_string(),
            who: who.to_string(),
            message: message.to_string(),
        })
    }
}

/// Extrait les entrees de la section `## Log` d'un corps de tache.
/// Les lignes non conformes sous la section sont ignorees silencieusement
/// a la lecture — c'est `check` qui les signale, pas le parseur.
pub fn parse_log(body: &str) -> Vec<LogEntry> {
    let mut in_log = false;
    let mut entries = Vec::new();
    for line in body.lines() {
        if line.trim_end() == LOG_HEADER {
            in_log = true;
            continue;
        }
        if in_log && line.starts_with("## ") {
            break;
        }
        if in_log {
            if let Some(e) = LogEntry::parse_line(line) {
                entries.push(e);
            }
        }
    }
    entries
}

/// Appende une entree au corps, en creant la section si necessaire.
/// Invariant : le resultat differe de l'entree d'exactement une ligne
/// (plus l'en-tete de section la premiere fois).
pub fn append_log(body: &str, entry: &LogEntry) -> String {
    let mut out = body.to_string();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    let has_header = out.lines().any(|l| l.trim_end() == LOG_HEADER);
    if !has_header {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(LOG_HEADER);
        out.push('\n');
    }
    out.push_str(&entry.format_line());
    out.push('\n');
    out
}
