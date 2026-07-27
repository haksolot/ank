//! Squelette du CLI. Les verbes arrivent tache par tache — voir .ankor/tasks/.
fn main() {
    eprintln!("ankor {} — pre-v1, voir .ankor/tasks/ pour l'avancement", env!("CARGO_PKG_VERSION"));
    std::process::exit(1);
}
