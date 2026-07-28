//! Store d'entites : lecture, ecriture atomique, compare-and-swap sur `version`.
//!
//! Couche fichiers sous l'index SQLite, lequel est un cache jetable et jamais
//! source de verite (§6). Ne depend ni de la config ni du dispatch : un
//! [`Store`] se construit avec un chemin `.ankor/` et rien d'autre.
//!
//! Deux garanties distinctes et complementaires, qu'il ne faut pas confondre :
//!
//! - **write-then-rename** — le fichier definitif n'est jamais observe dans un
//!   etat partiel, parce qu'il n'est jamais ecrit en place ;
//! - **verrou sur le cycle lecture-comparaison-ecriture** — c'est lui, et non
//!   le renommage, qui rend le compare-and-swap sur `version` effectif. Le
//!   renommage seul ne compare rien : deux ecrivains liraient la meme version
//!   de base et le second ecraserait le premier sans que rien ne le signale.

use ankor_core::{parse_entity, resolve_prefix, serialize_entity, Entity, EntityId, EntityKind};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Attente maximale du verrou d'entite. Au-dela, le verrou est repute
/// abandonne par un processus mort : on echoue en nommant le fichier a
/// supprimer, plutot que d'attendre indefiniment sans rien dire.
const LOCK_TIMEOUT: Duration = Duration::from_secs(10);

// ---------------------------------------------------------------------------
// Erreurs
// ---------------------------------------------------------------------------

/// Erreurs du store. Chacune porte son code de sortie (§4) et, quand une
/// suite existe, la commande exacte a executer — jamais d'aide generique.
/// Le rendu `error[<code>]: ...` appartient a la couche CLI ; ici on nomme
/// la cause et on fournit la sortie.
#[derive(Debug)]
pub enum StoreError {
    NotFound(String),
    AmbiguousPrefix {
        prefix: String,
        candidates: Vec<String>,
    },
    PrefixTooShort(String),
    VersionConflict {
        id: String,
        expected: u64,
        found: u64,
    },
    FilenameMismatch {
        path: PathBuf,
        expected: PathBuf,
    },
    Parse {
        path: PathBuf,
        source: ankor_core::Error,
    },
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    LockTimeout {
        lock: PathBuf,
    },
}

impl StoreError {
    /// Code de sortie de la table du §4.
    pub fn code(&self) -> i32 {
        match self {
            StoreError::NotFound(_)
            | StoreError::AmbiguousPrefix { .. }
            | StoreError::PrefixTooShort(_) => 2,
            StoreError::VersionConflict { .. } => 3,
            StoreError::FilenameMismatch { .. }
            | StoreError::Parse { .. }
            | StoreError::Io { .. }
            | StoreError::LockTimeout { .. } => 1,
        }
    }

    /// Commande exacte a executer ensuite, quand il y en a une.
    pub fn hint(&self) -> Option<String> {
        match self {
            StoreError::NotFound(p) => Some(format!("ankor find {p}")),
            StoreError::AmbiguousPrefix { candidates, .. } => {
                candidates.first().map(|c| format!("ankor show {c}"))
            }
            StoreError::PrefixTooShort(p) => Some(format!("ankor find {p}")),
            // Code 3 signifie litteralement : quelqu'un a bouge, relis.
            StoreError::VersionConflict { .. } => Some("ankor context".to_string()),
            StoreError::FilenameMismatch { path, expected } => {
                Some(format!("git mv {} {}", path.display(), expected.display()))
            }
            StoreError::LockTimeout { lock } => Some(format!("rm {}", lock.display())),
            StoreError::Parse { .. } | StoreError::Io { .. } => None,
        }
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::NotFound(p) => write!(f, "entite introuvable : {p}"),
            StoreError::AmbiguousPrefix { prefix, candidates } => {
                write!(f, "prefixe ambigu '{prefix}' : {}", candidates.join(", "))
            }
            StoreError::PrefixTooShort(p) => {
                write!(f, "prefixe trop court '{p}' (minimum 4 caracteres)")
            }
            StoreError::VersionConflict {
                id,
                expected,
                found,
            } => write!(
                f,
                "{id} a ete modifiee : version {found} sur disque, {expected} attendue"
            ),
            StoreError::FilenameMismatch { path, expected } => write!(
                f,
                "{} ne porte pas l'id de l'entite qu'il contient (attendu {})",
                path.display(),
                expected.display()
            ),
            StoreError::Parse { path, source } => write!(f, "{} : {source}", path.display()),
            StoreError::Io { path, source } => write!(f, "{} : {source}", path.display()),
            StoreError::LockTimeout { lock } => write!(
                f,
                "verrou {} toujours tenu apres {}s, processus probablement mort",
                lock.display(),
                LOCK_TIMEOUT.as_secs()
            ),
        }
    }
}

impl std::error::Error for StoreError {}

pub type Result<T> = std::result::Result<T, StoreError>;

// ---------------------------------------------------------------------------
// Acces au champ version, commun aux deux types d'entite
// ---------------------------------------------------------------------------

pub fn version_of(entity: &Entity) -> u64 {
    match entity {
        Entity::Task(t) => t.version,
        Entity::Adr(a) => a.version,
    }
}

fn set_version(entity: &mut Entity, v: u64) {
    match entity {
        Entity::Task(t) => t.version = v,
        Entity::Adr(a) => a.version = v,
    }
}

// ---------------------------------------------------------------------------
// Verrou d'entite
// ---------------------------------------------------------------------------

/// Verrou exclusif porte par la creation atomique d'un fichier : `create_new`
/// echoue si la cible existe, ce que le noyau garantit entre threads comme
/// entre processus. Libere par `Drop`, y compris en cas de panique.
struct Lock {
    path: PathBuf,
}

impl Lock {
    fn acquire(target: &Path) -> Result<Lock> {
        let path = lock_path(target);
        let deadline = Instant::now() + LOCK_TIMEOUT;
        // Dernier refus observe, pour distinguer au bout du compte une
        // contention d'un vrai probleme de droits.
        let mut last: Option<std::io::Error> = None;
        loop {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Lock { path }),
                // `AlreadyExists` est la contention nominale. `PermissionDenied`
                // l'est aussi sur Windows, et c'est contre-intuitif : entre le
                // `remove_file` du `Drop` et la disparition effective, le
                // fichier est en etat delete-pending et son ouverture rend
                // ERROR_ACCESS_DENIED, pas ERROR_FILE_EXISTS. Traiter ce cas
                // comme une erreur fatale ferait echouer un verrou en cours de
                // liberation — c'est-a-dire precisement le cas nominal sous
                // concurrence.
                Err(e)
                    if e.kind() == ErrorKind::AlreadyExists
                        || e.kind() == ErrorKind::PermissionDenied =>
                {
                    if Instant::now() >= deadline {
                        return match last {
                            // Dix secondes de refus de droits ne sont pas une
                            // contention : on rend l'erreur systeme telle quelle.
                            Some(source) if source.kind() == ErrorKind::PermissionDenied => {
                                Err(StoreError::Io { path, source })
                            }
                            _ => Err(StoreError::LockTimeout { lock: path }),
                        };
                    }
                    last = Some(e);
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(source) => return Err(StoreError::Io { path, source }),
            }
        }
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// `.<nom>.lock` a cote de la cible : commence par un point et ne finit pas
/// en `.md`, donc jamais confondu avec une entite par [`Store::list_ids`].
fn lock_path(target: &Path) -> PathBuf {
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target.with_file_name(format!(".{name}.lock"))
}

// ---------------------------------------------------------------------------
// Ecriture atomique
// ---------------------------------------------------------------------------

/// Nom temporaire unique. L'unicite n'est pas redondante avec le verrou : un
/// temporaire residuel laisse par un processus tue ne doit pas faire echouer
/// l'ecriture suivante.
fn tmp_path(target: &Path) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    target.with_file_name(format!(
        ".{name}.tmp.{}.{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ))
}

fn write_atomic(target: &Path, contents: &str) -> Result<()> {
    let tmp = tmp_path(target);
    let io = |path: &Path, source: std::io::Error| StoreError::Io {
        path: path.to_path_buf(),
        source,
    };
    {
        let mut f = File::create(&tmp).map_err(|e| io(&tmp, e))?;
        f.write_all(contents.as_bytes()).map_err(|e| io(&tmp, e))?;
        // Le contenu doit avoir atteint le disque avant que le nom definitif
        // ne le designe : sans cela, un crash laisse un fichier au bon nom et
        // au contenu vide, exactement ce que le renommage doit exclure.
        f.sync_all().map_err(|e| io(&tmp, e))?;
    }
    match fs::rename(&tmp, target) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(io(target, e))
        }
    }
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

/// Une entite chargee, avec le chemin d'ou elle vient.
#[derive(Debug, Clone)]
pub struct Loaded {
    pub entity: Entity,
    pub path: PathBuf,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    /// `root` est le repertoire `.ankor/`. Rien d'autre n'est requis : ni
    /// config, ni index, ni git.
    pub fn new(root: impl Into<PathBuf>) -> Store {
        Store { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn subdir(kind: EntityKind) -> &'static str {
        match kind {
            EntityKind::Task => "tasks",
            EntityKind::Adr => "adr",
        }
    }

    /// Chemin canonique d'une entite. Le nom de fichier porte toujours l'id.
    pub fn path_of(&self, id: &EntityId) -> PathBuf {
        self.root
            .join(Self::subdir(id.kind()))
            .join(format!("{id}.md"))
    }

    /// Identifiants presents sur disque. Un fichier dont le nom n'est pas
    /// `<ID>.md` n'est pas une entite et est ignore ici — temporaires,
    /// verrous et notes libres traversent donc le listage sans le polluer.
    /// C'est `check` qui signale un `.md` egare, pas le store.
    pub fn list_ids(&self) -> Result<Vec<EntityId>> {
        let mut ids = Vec::new();
        for kind in [EntityKind::Task, EntityKind::Adr] {
            let dir = self.root.join(Self::subdir(kind));
            let rd = match fs::read_dir(&dir) {
                Ok(rd) => rd,
                Err(e) if e.kind() == ErrorKind::NotFound => continue,
                Err(source) => return Err(StoreError::Io { path: dir, source }),
            };
            for entry in rd {
                let entry = entry.map_err(|source| StoreError::Io {
                    path: dir.clone(),
                    source,
                })?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("md") {
                    continue;
                }
                let stem = match path.file_stem().and_then(|s| s.to_str()) {
                    Some(s) => s,
                    None => continue,
                };
                if let Ok(id) = EntityId::parse(stem) {
                    if id.kind() == kind {
                        ids.push(id);
                    }
                }
            }
        }
        ids.sort_by_key(|id| id.to_string());
        Ok(ids)
    }

    /// Resolution d'un prefixe. L'ambiguite est une erreur qui liste ses
    /// candidats : l'outil ne devine jamais (§3).
    pub fn resolve(&self, prefix: &str) -> Result<EntityId> {
        let ids = self.list_ids()?;
        match resolve_prefix(prefix, ids.iter()) {
            Ok(id) => Ok(id.clone()),
            Err(ankor_core::Error::AmbiguousPrefix { prefix, candidates }) => {
                Err(StoreError::AmbiguousPrefix { prefix, candidates })
            }
            Err(ankor_core::Error::PrefixTooShort(p)) => Err(StoreError::PrefixTooShort(p)),
            Err(_) => Err(StoreError::NotFound(prefix.to_string())),
        }
    }

    /// Charge le fichier d'un chemin donne, en exigeant que son nom porte
    /// l'id de l'entite qu'il contient. Sans cette verification, un fichier
    /// renomme a la main deviendrait une entite fantome : listee sous un id,
    /// chargee sous un autre.
    pub fn load_path(&self, path: &Path) -> Result<Loaded> {
        let text = fs::read_to_string(path).map_err(|source| {
            if source.kind() == ErrorKind::NotFound {
                StoreError::NotFound(path.display().to_string())
            } else {
                StoreError::Io {
                    path: path.to_path_buf(),
                    source,
                }
            }
        })?;
        let entity = parse_entity(&text).map_err(|source| StoreError::Parse {
            path: path.to_path_buf(),
            source,
        })?;
        let expected = self.path_of(entity.id());
        let same_name = path.file_name() == expected.file_name();
        if !same_name {
            return Err(StoreError::FilenameMismatch {
                path: path.to_path_buf(),
                expected,
            });
        }
        Ok(Loaded {
            entity,
            path: path.to_path_buf(),
        })
    }

    pub fn load(&self, id: &EntityId) -> Result<Loaded> {
        let path = self.path_of(id);
        match self.load_path(&path) {
            Err(StoreError::NotFound(_)) => Err(StoreError::NotFound(id.to_string())),
            other => other,
        }
    }

    pub fn load_prefix(&self, prefix: &str) -> Result<Loaded> {
        let id = self.resolve(prefix)?;
        self.load(&id)
    }

    /// Cree une entite qui n'existe pas encore. Echoue si le fichier est deja
    /// la : `new` ne doit jamais ecraser, un id en collision est un bug qu'on
    /// veut voir.
    pub fn create(&self, entity: &Entity) -> Result<()> {
        let path = self.path_of(entity.id());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|source| StoreError::Io {
                path: parent.to_path_buf(),
                source,
            })?;
        }
        let _lock = Lock::acquire(&path)?;
        if path.exists() {
            return Err(StoreError::Io {
                path: path.clone(),
                source: std::io::Error::new(ErrorKind::AlreadyExists, "l'entite existe deja"),
            });
        }
        write_atomic(&path, &serialize_entity(entity))
    }

    /// Ecriture avec compare-and-swap sur `version`.
    ///
    /// `base_version` est la version telle qu'elle a ete lue par l'appelant.
    /// Si le disque a bouge entretemps, rien n'est ecrit et le code 3 dit a
    /// la boucle agentique de relire. En cas de succes, la version ecrite est
    /// exactement `base_version + 1` — c'est le store qui l'incremente, pour
    /// que l'appelant ne puisse pas oublier de le faire.
    pub fn write(&self, entity: &Entity, base_version: u64) -> Result<u64> {
        let path = self.path_of(entity.id());
        // Le verrou couvre la lecture, la comparaison et l'ecriture. Le
        // relacher entre la lecture et l'ecriture reintroduirait exactement
        // la course que le compare-and-swap existe pour fermer.
        let _lock = Lock::acquire(&path)?;
        let current = self.load_path(&path)?;
        let found = version_of(&current.entity);
        if found != base_version {
            return Err(StoreError::VersionConflict {
                id: entity.id().to_string(),
                expected: base_version,
                found,
            });
        }
        let next_version = base_version + 1;
        let mut next = entity.clone();
        set_version(&mut next, next_version);
        write_atomic(&path, &serialize_entity(&next))?;
        Ok(next_version)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ankor_core::{CriteriaBy, Task, TaskStatus};
    use std::sync::Arc;

    /// Repertoire `.ankor/` jetable. Pas de dependance externe : le besoin
    /// est trop mince pour justifier une caisse de plus.
    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> TempRoot {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ankor-store-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join("tasks")).unwrap();
            fs::create_dir_all(p.join("adr")).unwrap();
            TempRoot(p)
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn task(hex: &str, title: &str) -> Entity {
        Entity::Task(Task {
            id: EntityId::parse(&format!("TASK-{hex}")).unwrap(),
            slug: Some("exemple".into()),
            title: title.into(),
            created: "2026-07-28T00:00:00Z".into(),
            status: TaskStatus::Open,
            scope: vec!["src/**".into()],
            blocked_by: vec![],
            done_criteria: Some("Un critere verifiable.\n".into()),
            criteria_by: Some(CriteriaBy::Creator),
            verify: vec![],
            proof: vec![],
            schema: 1,
            version: 1,
            body: "\nCorps libre.\n".into(),
        })
    }

    fn seeded() -> (TempRoot, Store, Entity) {
        let root = TempRoot::new();
        let store = Store::new(&root.0);
        let e = task("000000000001", "Tache exemple");
        store.create(&e).unwrap();
        (root, store, e)
    }

    #[test]
    fn charge_par_id_complet_et_par_prefixe() {
        let (_root, store, e) = seeded();
        assert_eq!(store.load(e.id()).unwrap().entity, e);
        assert_eq!(store.load_prefix("0000").unwrap().entity, e);
        assert_eq!(store.load_prefix("TASK-0000").unwrap().entity, e);
    }

    #[test]
    fn introuvable_et_prefixe_ambigu_sortent_en_2() {
        let (_root, store, _) = seeded();
        store.create(&task("00000000ffff", "Autre")).unwrap();

        let err = store.load_prefix("abcd").unwrap_err();
        assert_eq!(err.code(), 2, "{err}");
        assert!(matches!(err, StoreError::NotFound(_)));

        // "0000" matche les deux entites : l'outil ne devine pas.
        let err = store.load_prefix("0000").unwrap_err();
        assert_eq!(err.code(), 2, "{err}");
        match &err {
            StoreError::AmbiguousPrefix { candidates, .. } => {
                assert_eq!(candidates.len(), 2, "les candidats sont listes : {err}");
            }
            other => panic!("attendu AmbiguousPrefix, obtenu {other:?}"),
        }
        assert!(err.hint().unwrap().starts_with("ankor show TASK-"));
    }

    #[test]
    fn version_perimee_refusee_en_3_et_fichier_inchange() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        let avant = fs::read(&path).unwrap();

        // Une premiere ecriture porte le disque en version 2.
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        let apres_gagnant = fs::read(&path).unwrap();

        // Le retardataire tient toujours la version 1 pour base.
        let err = store.write(&e, 1).unwrap_err();
        assert_eq!(err.code(), 3, "{err}");
        assert_eq!(err.hint().as_deref(), Some("ankor context"));
        assert_eq!(
            fs::read(&path).unwrap(),
            apres_gagnant,
            "un refus ne doit rien ecrire"
        );
        assert_ne!(avant, apres_gagnant);
    }

    #[test]
    fn ecriture_acceptee_incremente_version_de_un() {
        let (_root, store, e) = seeded();
        assert_eq!(version_of(&e), 1);
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        assert_eq!(version_of(&store.load(e.id()).unwrap().entity), 2);
        assert_eq!(store.write(&e, 2).unwrap(), 3);
        assert_eq!(version_of(&store.load(e.id()).unwrap().entity), 3);
    }

    #[test]
    fn relecture_apres_ecriture_identique_octet_pour_octet() {
        let (_root, store, e) = seeded();
        store.write(&e, 1).unwrap();
        let relu = store.load(e.id()).unwrap().entity;
        let sur_disque = fs::read_to_string(store.path_of(e.id())).unwrap();
        assert_eq!(serialize_entity(&relu), sur_disque);
        assert!(!sur_disque.contains('\r'), "le store ecrit du LF");
    }

    #[test]
    fn temporaire_residuel_ni_lu_ni_masquant() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        let residu = tmp_path(&path);
        fs::write(&residu, "contenu partiel, pas une entite").unwrap();

        assert_eq!(store.list_ids().unwrap(), vec![e.id().clone()]);
        assert_eq!(store.load(e.id()).unwrap().entity, e);
        assert_eq!(store.write(&e, 1).unwrap(), 2);
        assert!(
            residu.exists(),
            "le store ne nettoie pas le residu d'autrui"
        );
    }

    #[test]
    fn nom_de_fichier_ne_portant_pas_l_id_refuse() {
        let (root, store, e) = seeded();
        let egare = root.0.join("tasks").join("TASK-0000000000ff.md");
        fs::copy(store.path_of(e.id()), &egare).unwrap();

        let err = store.load_path(&egare).unwrap_err();
        match &err {
            StoreError::FilenameMismatch { .. } => {}
            other => panic!("attendu FilenameMismatch, obtenu {other:?}"),
        }
        assert!(err.hint().unwrap().starts_with("git mv "), "{err}");

        // L'entite fantome est listee sous le nom du fichier, et son
        // chargement echoue franchement plutot que de rendre l'autre.
        let err = store.load_prefix("0000000000ff").unwrap_err();
        assert!(matches!(err, StoreError::FilenameMismatch { .. }));
    }

    #[test]
    fn ecrivains_concurrents_un_seul_gagnant() {
        let (_root, store, e) = seeded();
        let store = Arc::new(store);
        let n = 16;

        let mut handles = Vec::new();
        for i in 0..n {
            let store = Arc::clone(&store);
            // Chaque thread ecrit un titre distinct : un fichier final
            // mixte serait donc visible, pas seulement une version fausse.
            let mine = task("000000000001", &format!("Ecrivain {i}"));
            handles.push(std::thread::spawn(move || store.write(&mine, 1)));
        }
        let resultats: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        let gagnants = resultats.iter().filter(|r| r.is_ok()).count();
        assert_eq!(gagnants, 1, "exactement un gagnant : {resultats:?}");
        for r in resultats.iter().filter(|r| r.is_err()) {
            let err = r.as_ref().unwrap_err();
            assert_eq!(err.code(), 3, "les perdants sortent en 3 : {err}");
        }

        // Le fichier final est une entite valide, jamais tronquee ni mixte.
        let final_ = store.load(e.id()).unwrap().entity;
        assert_eq!(version_of(&final_), 2);
        let sur_disque = fs::read_to_string(store.path_of(e.id())).unwrap();
        assert_eq!(serialize_entity(&final_), sur_disque);
        match &final_ {
            Entity::Task(t) => assert!(
                t.title.starts_with("Ecrivain "),
                "titre mixte : {:?}",
                t.title
            ),
            _ => panic!("attendu une tache"),
        }
    }

    #[test]
    fn le_verrou_est_libere_apres_usage() {
        let (_root, store, e) = seeded();
        let path = store.path_of(e.id());
        store.write(&e, 1).unwrap();
        assert!(
            !lock_path(&path).exists(),
            "un verrou survivant bloquerait toute ecriture ulterieure"
        );
    }
}
