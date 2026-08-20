//! What reading this repository through a Rust git library would cost, against
//! the same reads through the git binary (TASK-a8054da67947).
//!
//! **This decides nothing.** ADR-9307e5d214a7 is accepted and says the plumbing
//! goes through the git binary and never through a library; the ADR that would
//! supersede it is written against the number this prints, not before it. So
//! this crate is excluded from the workspace, ships in nothing, and no verb
//! reaches it.
//!
//! Run it from the repository it should measure:
//!
//!     cargo run --release --manifest-path spike/Cargo.toml -- <repo>
//!
//! **The open is timed apart from the reads**, because a library that opens a
//! repository in 300 ms has already given back most of what it saved, and an
//! average taken over one call would hide exactly that.
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// Read from the manifest so the record names what was measured.
const GIX_VERSION: &str = "0.86";

fn main() {
    let root = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string());
    let root = Path::new(&root);
    println!("repository: {}", root.display());
    // The library under measurement, not this crate: printing the spike's own
    // version said `0.0.0` and recorded nothing about what was measured.
    println!("gix: {}", GIX_VERSION);
    println!("platform: {}", std::env::consts::OS);

    // --- what the binary costs to start at all -----------------------------
    let t = Instant::now();
    for _ in 0..10 {
        let _ = Command::new("git").arg("--version").output();
    }
    println!("binary  spawn floor      {:>7.1} ms  (git --version, mean of 10)",
             t.elapsed().as_secs_f64() * 100.0);

    // --- opening -----------------------------------------------------------
    let t = Instant::now();
    let repo = match gix::open(root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gix could not open {}: {e}", root.display());
            std::process::exit(1);
        }
    };
    println!("library open             {:>7.1} ms", t.elapsed().as_secs_f64() * 1000.0);

    let t = Instant::now();
    let _ = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(root)
        .output();
    println!("binary  open             {:>7.1} ms  (rev-parse --show-toplevel)",
             t.elapsed().as_secs_f64() * 1000.0);

    // --- reading every entity blob at HEAD ---------------------------------
    let paths = entity_paths(root);
    println!("entities                 {:>7}", paths.len());

    let t = Instant::now();
    let mut bytes = 0usize;
    let tree = repo.head_commit().ok().and_then(|c| c.tree().ok());
    if let Some(tree) = tree {
        for p in &paths {
            if let Ok(Some(entry)) = tree.clone().peel_to_entry_by_path(Path::new(p)) {
                if let Ok(obj) = entry.object() {
                    bytes += obj.data.len();
                }
            }
        }
    }
    println!("library read blobs       {:>7.1} ms  ({bytes} bytes)",
             t.elapsed().as_secs_f64() * 1000.0);

    let t = Instant::now();
    let input: String = paths
        .iter()
        .map(|p| format!("HEAD:{p}\n"))
        .collect();
    let out = batch(root, &input);
    println!("binary  read blobs       {:>7.1} ms  ({out} bytes)",
             t.elapsed().as_secs_f64() * 1000.0);

    // --- reading the coordination refs -------------------------------------
    let t = Instant::now();
    let mut refs = 0usize;
    if let Ok(platform) = repo.references() {
        if let Ok(iter) = platform.prefixed("refs/ank/") {
            for r in iter.flatten() {
                let _ = r.id().object().map(|o| o.data.len());
                refs += 1;
            }
        }
    }
    println!("library read refs        {:>7.1} ms  ({refs} refs)",
             t.elapsed().as_secs_f64() * 1000.0);

    let t = Instant::now();
    let listed = Command::new("git")
        .args(["for-each-ref", "--format=%(objectname)", "refs/ank/"])
        .current_dir(root)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    let n = listed.lines().count();
    let _ = batch(root, &listed);
    println!("binary  read refs        {:>7.1} ms  ({n} refs)",
             t.elapsed().as_secs_f64() * 1000.0);
}

/// The entity paths, read from the working tree because the point is which
/// objects are asked for, not how the list is obtained.
fn entity_paths(root: &Path) -> Vec<String> {
    let dir = root.join(".ank").join("entities");
    let mut out: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md").then(|| format!(".ank/entities/{name}"))
        })
        .collect();
    out.sort();
    out
}

/// One `cat-file --batch`, which is what the tool does today.
fn batch(root: &Path, input: &str) -> usize {
    use std::io::Write as _;
    let Ok(mut child) = Command::new("git")
        .args(["cat-file", "--batch"])
        .current_dir(root)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        return 0;
    };
    if let Some(mut sink) = child.stdin.take() {
        let _ = sink.write_all(input.as_bytes());
    }
    child
        .wait_with_output()
        .map(|o| o.stdout.len())
        .unwrap_or(0)
}
