//! Binaire `ankor`.
//!
//! Volontairement mince : il resout le repertoire courant, delegue a
//! [`cli::run`] et propage le code de sortie. Tout ce qui merite un test vit
//! dans les modules, ou il est testable sans lancer de processus.

// Le socle expose deliberement plus que ce que le dispatch consomme
// aujourd'hui : identite, config et store sont ecrits et testes ici, mais
// leurs appelants sont les verbes, qui arrivent tache par tache. L'allow
// tombe avec le dernier d'entre eux ; le garder a la racine plutot que
// disperse en annotations le rend visible et retirable en une ligne.
#![allow(dead_code)]

mod cli;
mod config;
mod git;
mod identity;
mod init;
mod repo;
mod store;

// Modules de verbes, vers lesquels le dispatch route. Chacun est rempli par
// sa propre tache — voir .ankor/tasks/. Ils existent des maintenant pour que
// la table des commandes de cli.rs soit complete et testee, et pour qu'aucune
// tache de verbe n'ait a toucher au dispatch.
mod claim;
mod commands;
mod context;
mod done;
mod human;
mod index;
mod verify;

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let mut out = std::io::stdout();
    std::process::exit(cli::run(&argv, &cwd, &mut out));
}
