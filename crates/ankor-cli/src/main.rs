//! Squelette du CLI. Les verbes arrivent tache par tache — voir .ankor/tasks/.

// Le module est declare ici parce que Rust l'exige pour le compiler ; son
// cablage au dispatch appartient a TASK-c8637488773c, qui reecrira ce
// fichier. `dead_code` tombera avec le premier verbe qui l'appelle.
#[allow(dead_code)]
mod store;

fn main() {
    eprintln!(
        "ankor {} — pre-v1, voir .ankor/tasks/ pour l'avancement",
        env!("CARGO_PKG_VERSION")
    );
    std::process::exit(1);
}
