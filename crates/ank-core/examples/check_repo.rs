//! Embryo of `ank check`, used to dogfood the format while the CLI does not
//! exist yet: parses all of `.ank/`, and requires an identical round-trip and
//! resolvable `blocked_by` references.
//! Usage: `cargo run --example check_repo [path/.ank]`

use ank_core::*;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let root = std::env::args().nth(1).unwrap_or_else(|| ".ank".into());
    let root = PathBuf::from(root);
    let mut errors = 0usize;
    let mut entities: Vec<(PathBuf, Entity)> = Vec::new();

    for sub in ["tasks", "adr"] {
        let dir = root.join(sub);
        let Ok(rd) = fs::read_dir(&dir) else {
            eprintln!("error: directory not found {}", dir.display());
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
                    eprintln!("error: {}: {err}", path.display());
                    errors += 1;
                }
                Ok(entity) => {
                    let out = serialize_entity(&entity);
                    if out != input {
                        eprintln!(
                            "error: {}: non-canonical form (round-trip differs)",
                            path.display()
                        );
                        errors += 1;
                    }
                    // The file name must carry the canonical id.
                    let expected = format!("{}.md", entity.id());
                    if path.file_name().unwrap().to_str() != Some(expected.as_str()) {
                        eprintln!("error: {}: expected name {expected}", path.display());
                        errors += 1;
                    }
                    entities.push((path, entity));
                }
            }
        }
    }

    // blocked_by references and derived blocking.
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
                    eprintln!("error: {}: {err}", path.display());
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
        eprintln!(
            "check_repo: {errors} error(s) across {} entities",
            entities.len()
        );
        exit(8);
    }
    println!("check_repo: ok — {tasks} tasks ({ready} ready), {adrs} adr, round-trip identical");
}
