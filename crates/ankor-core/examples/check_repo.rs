//! Embryon de `ankor check`, utilise pour dogfooder le format pendant que
//! le CLI n'existe pas : parse tout `.ankor/`, exige le round-trip a
//! l'identique et la resolution des references `blocked_by`.
//! Usage : `cargo run --example check_repo [chemin/.ankor]`

use ankor_core::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".ankor".into());
    let root = PathBuf::from(root);
    let mut errors = 0usize;
    let mut entities: Vec<(PathBuf, Entity)> = Vec::new();

    for sub in ["tasks", "adr"] {
        let dir = root.join(sub);
        let Ok(rd) = fs::read_dir(&dir) else {
            eprintln!("error: dossier introuvable {}", dir.display());
            exit(1);
        };
        for e in rd {
            let path = e.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let input = fs::read_to_string(&path).unwrap();
            match parse_entity(&input) {
                Err(err) => {
                    eprintln!("error: {} : {err}", path.display());
                    errors += 1;
                }
                Ok(entity) => {
                    let out = serialize_entity(&entity);
                    if out != input {
                        eprintln!("error: {} : forme non canonique (round-trip different)", path.display());
                        errors += 1;
                    }
                    // Le nom de fichier doit porter l'id canonique.
                    let expected = format!("{}.md", entity.id());
                    if path.file_name().unwrap().to_str() != Some(expected.as_str()) {
                        eprintln!("error: {} : nom attendu {expected}", path.display());
                        errors += 1;
                    }
                    entities.push((path, entity));
                }
            }
        }
    }

    // References blocked_by et blocage derive.
    let statuses: HashMap<String, TaskStatus> = entities
        .iter()
        .filter_map(|(_, e)| match e {
            Entity::Task(t) => Some((t.id.to_string(), t.status)),
            _ => None,
        })
        .collect();
    let mut ready = 0usize;
    for (path, e) in &entities {
        if let Entity::Task(t) = e {
            match t.active_blockers(|id| statuses.get(&id.to_string()).copied()) {
                Err(err) => {
                    eprintln!("error: {} : {err}", path.display());
                    errors += 1;
                }
                Ok(active) => {
                    if t.status == TaskStatus::Open && active.is_empty() {
                        ready += 1;
                    }
                }
            }
        }
    }

    let tasks = statuses.len();
    let adrs = entities.len() - tasks;
    if errors > 0 {
        eprintln!("check_repo: {errors} erreur(s) sur {} entites", entities.len());
        exit(8);
    }
    println!("check_repo: ok — {tasks} taches ({ready} pretes), {adrs} adr, round-trip identique");
}
