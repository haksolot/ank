//! Gel par hash (§3 de la spec).
//!
//! Le CLI n'est pas un gardien : n'importe quel outil peut reecrire un
//! fichier. Un champ gele est donc ancre par un hash dans un artefact que
//! l'editeur ne controle pas (l'enregistrement de claim, le commit de
//! ratification), et compare ici. La normalisation rend le hash insensible
//! au bruit d'edition (espaces de fin de ligne, saut final) sans jamais
//! tolerer un changement de sens.

use sha2::{Digest, Sha256};

/// Normalisation : fins de ligne CRLF -> LF, espaces de fin de ligne
/// supprimes, sauts de ligne finaux supprimes.
pub fn normalize(text: &str) -> String {
    let unified = text.replace("\r\n", "\n");
    let mut lines: Vec<&str> = unified.lines().map(|l| l.trim_end()).collect();
    while lines.last() == Some(&"") {
        lines.pop();
    }
    lines.join("\n")
}

/// Hash complet (hex, 64 caracteres) du texte normalise.
pub fn freeze_hash(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(normalize(text).as_bytes());
    hex::encode(h.finalize())
}

/// Forme courte pour affichage et enregistrement de claim (12 hex,
/// coherent avec la longueur des identifiants).
pub fn freeze_hash_short(text: &str) -> String {
    freeze_hash(text)[..12].to_string()
}

/// Verification d'un champ gele contre son hash d'ancrage. Accepte la
/// forme courte ou la forme longue.
pub fn verify_frozen(text: &str, anchor: &str) -> bool {
    let full = freeze_hash(text);
    full == anchor || full.starts_with(anchor) && anchor.len() >= 12
}
