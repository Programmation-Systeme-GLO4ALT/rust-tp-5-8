// ============================================================
// TD GL4 — Exercice 4 : Système de logs persistants avec rotation
// Séance 8 : Système de fichiers & I/O
// Programmation Système avec Rust — ENSPD GL4 2025-2026
// ============================================================
//
// Parties :
//   4.1 — Lecture et écriture avec std::fs et std::io
//   4.2 — Rotation de fichiers
//   4.3 — Gestion des chemins et portabilité
// ============================================================

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// Types
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum NiveauLog { DEBUG, INFO, WARN, ERROR }

impl std::fmt::Display for NiveauLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NiveauLog::DEBUG => write!(f, "DEBUG"),
            NiveauLog::INFO  => write!(f, "INFO"),
            NiveauLog::WARN  => write!(f, "WARN"),
            NiveauLog::ERROR => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntreeLog {
    pub timestamp: u64,
    pub niveau:    NiveauLog,
    pub module:    String,
    pub message:   String,
}

/// Configuration de la rotation des fichiers de logs
#[derive(Debug, Clone)]
pub struct RotationConfig {
    pub repertoire:    PathBuf,  // Dossier de stockage
    pub prefixe:       String,   // Ex: "app_server"
    pub taille_max_ko: u64,      // Taille max en Ko
    pub max_fichiers:  usize,    // Nombre max d'archives
}

// ============================================================
// PARTIE 4.1 — Lecture et écriture
// ============================================================

/// Q1 — Écriture d'une entrée de log dans un fichier (mode append).
/// Utilise BufWriter pour grouper les écritures et réduire les appels système.
///
/// Format : [timestamp] NIVEAU module: message\n
pub fn ecrire_log(chemin: &Path, entree: &EntreeLog) -> Result<(), io::Error> {
    let fichier = OpenOptions::new()
        .write(true)
        .create(true)    // crée le fichier s'il n'existe pas
        .append(true)    // ajoute à la fin (ne tronque pas)
        .open(chemin)?;

    // BufWriter regroupe les écritures pour réduire les syscalls
    let mut ecrivain = BufWriter::new(fichier);

    writeln!(
        ecrivain,
        "[{}] {} {}: {}",
        entree.timestamp, entree.niveau, entree.module, entree.message
    )?;

    // flush() force l'écriture du buffer vers le disque
    ecrivain.flush()?;
    Ok(())
}

/// Q2 — Lit les N dernières lignes d'un fichier sans charger tout en mémoire.
/// Utilise un VecDeque de taille bornée (fenêtre glissante sur les lignes).
/// VecDeque permet push_back + pop_front en O(1), gardant exactement n lignes.
pub fn lire_derniers_logs(chemin: &Path, n: usize) -> Result<Vec<String>, io::Error> {
    let fichier = fs::File::open(chemin)?;
    let lecteur = io::BufReader::new(fichier);

    // VecDeque de taille bornée : quand elle est pleine, on retire la plus ancienne
    let mut fenetre: VecDeque<String> = VecDeque::with_capacity(n + 1);

    for ligne in lecteur.lines() {
        let l = ligne?;
        fenetre.push_back(l);
        if fenetre.len() > n {
            fenetre.pop_front(); // retire la plus ancienne
        }
    }

    Ok(fenetre.into_iter().collect())
}

// ============================================================
// PARTIE 4.2 — Rotation de fichiers
// ============================================================

/// Q3 — Vérifie si le fichier courant dépasse la taille max.
/// Si oui, le renomme en archive horodatée et supprime les plus anciennes.
pub fn verifier_rotation(config: &RotationConfig) -> Result<(), io::Error> {
    let chemin_courant = chemin_log_courant(config);

    // Vérifier si le fichier courant existe
    if !chemin_courant.exists() {
        return Ok(()); // rien à faire
    }

    // Récupérer la taille du fichier
    let metadata = fs::metadata(&chemin_courant)?;
    let taille_ko = metadata.len() / 1024;

    if taille_ko < config.taille_max_ko {
        return Ok(()); // pas encore atteint la limite
    }

    // --- Renommer le fichier courant en archive horodatée ---
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let nom_archive = format!("{}_{}.log", config.prefixe, timestamp);
    let chemin_archive = config.repertoire.join(&nom_archive);

    fs::rename(&chemin_courant, &chemin_archive)?;
    println!("  [ROTATION] {} → {}", chemin_courant.display(), nom_archive);

    // --- Supprimer les archives trop anciennes ---
    supprimer_archives_exces(config)?;

    Ok(())
}

/// Q4 — Liste et trie les archives par date de modification (plus récent en premier).
pub fn lister_archives_triees(config: &RotationConfig) -> Result<Vec<PathBuf>, io::Error> {
    let pattern = format!("{}_", config.prefixe);

    // Lire le répertoire
    let mut archives: Vec<(PathBuf, SystemTime)> = fs::read_dir(&config.repertoire)?
        .filter_map(|entree| entree.ok())
        .filter(|entree| {
            let nom = entree.file_name();
            let nom_str = nom.to_string_lossy();
            // Garder uniquement les fichiers correspondant au pattern <prefixe>_*.log
            nom_str.starts_with(&pattern) && nom_str.ends_with(".log")
        })
        .filter_map(|entree| {
            // Récupérer la date de modification
            let chemin = entree.path();
            let modif = fs::metadata(&chemin)
                .and_then(|m| m.modified())
                .ok()?;
            Some((chemin, modif))
        })
        .collect();

    // Trier par date de modification décroissante (plus récent en premier)
    archives.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(archives.into_iter().map(|(p, _)| p).collect())
}

/// Supprime les archives en excès si leur nombre dépasse max_fichiers.
fn supprimer_archives_exces(config: &RotationConfig) -> Result<(), io::Error> {
    let archives = lister_archives_triees(config)?;

    // Supprimer les archives au-delà de max_fichiers
    for archive_a_supprimer in archives.iter().skip(config.max_fichiers) {
        fs::remove_file(archive_a_supprimer)?;
        println!("  [SUPPRESSION] {}", archive_a_supprimer.display());
    }
    Ok(())
}

// ============================================================
// PARTIE 4.3 — Gestion des chemins et portabilité
// ============================================================

/// Q5 — Path vs PathBuf :
///
///   Path    : type non-owning, similaire à &str.
///             C'est une "vue" sur un chemin, utilisé en paramètre quand on
///             veut juste lire le chemin sans en prendre possession.
///             → Utilisé dans les paramètres de fonctions : fn f(p: &Path)
///
///   PathBuf : type owning, similaire à String.
///             Possède les données du chemin sur le heap, peut être modifié.
///             Utilisé quand on construit ou retourne un chemin.
///             → Utilisé en retour de fonctions : fn g() -> PathBuf
///
///   Analogie :
///     &str   ↔  &Path    (emprunt, lecture seule)
///     String ↔  PathBuf  (propriétaire, modifiable)
///
///   En paramètre : préférer &Path (accepte aussi &PathBuf via Deref coercion).
///   En retour     : PathBuf obligatoire (on ne peut pas retourner &Path qui
///                   pointerait sur des données locales).

/// Q6 — Retourne le chemin du fichier de log courant.
/// Utilise Path::join pour la portabilité Linux/Windows (pas de "/" en dur).
pub fn chemin_log_courant(config: &RotationConfig) -> PathBuf {
    // Path::join utilise le séparateur natif : '/' sur Linux, '\' sur Windows
    config.repertoire.join(format!("{}.log", config.prefixe))
}

/// Q7 (Bonus) — Memory-mapped files (memmap2) :
///
/// Un fichier memory-mappé (mmap) est mappé directement dans l'espace d'adressage
/// du processus. On peut accéder à ses octets comme s'ils étaient en mémoire,
/// sans copier les données via read().
///
/// Quand préférer mmap à BufReader :
///   → Lecture aléatoire de très grands fichiers (>100 Mo) : mmap évite de lire
///     séquentiellement depuis le début.
///   → Recherche de patterns dans un fichier de logs volumineux.
///   → Plusieurs processus accèdent au même fichier (partage de mémoire).
///
/// Précautions de sécurité avec mmap :
///   → Le fichier peut être modifié par un autre processus pendant la lecture
///     (TOCTOU — Time of Check to Time of Use). Toujours valider les données lues.
///   → Utiliser des blocs unsafe en Rust car mmap est fondamentalement unsafe
///     (les invariants mémoire ne sont pas garantis si le fichier change).
///   → Éviter de mapper des fichiers de taille inconnue sans limites (DoS).
///   → Sur Linux, gérer les signaux SIGBUS (fichier tronqué après le mapping).

// ============================================================
// Programme principal
// ============================================================
fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  TD GL4 — Ex4 : Logs Persistants avec Rotation      ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    let tmp_dir = PathBuf::from("/tmp/logs_demo");
    fs::create_dir_all(&tmp_dir).unwrap();

    let config = RotationConfig {
        repertoire:    tmp_dir.clone(),
        prefixe:       "app".to_string(),
        taille_max_ko: 1,      // 1 Ko pour démo (petit seuil)
        max_fichiers:  3,
    };

    let chemin = chemin_log_courant(&config);
    println!("[ Chemin courant ] {}", chemin.display());

    // --- Écriture de logs ---
    println!("\n[ Écriture de 5 logs ]");
    let entrees = vec![
        EntreeLog { timestamp: 1000, niveau: NiveauLog::INFO,  module: "main".into(), message: "Démarrage serveur".into() },
        EntreeLog { timestamp: 1001, niveau: NiveauLog::DEBUG, module: "db".into(),   message: "Connexion établie".into() },
        EntreeLog { timestamp: 1002, niveau: NiveauLog::WARN,  module: "auth".into(), message: "Tentative échouée".into() },
        EntreeLog { timestamp: 1003, niveau: NiveauLog::ERROR, module: "api".into(),  message: "Timeout 504".into() },
        EntreeLog { timestamp: 1004, niveau: NiveauLog::INFO,  module: "main".into(), message: "Requête traitée".into() },
    ];

    for entree in &entrees {
        ecrire_log(&chemin, entree).unwrap();
        println!("  Écrit : [{} {}] {}", entree.niveau, entree.module, entree.message);
    }

    // --- Lecture des 3 derniers logs ---
    println!("\n[ Lecture des 3 derniers logs ]");
    match lire_derniers_logs(&chemin, 3) {
        Ok(lignes) => {
            for l in &lignes {
                println!("  {}", l);
            }
        }
        Err(e) => println!("  Erreur : {}", e),
    }

    // --- Vérification de rotation ---
    println!("\n[ Vérification rotation (seuil=1 Ko) ]");
    verifier_rotation(&config).unwrap();

    // --- Portabilité des chemins ---
    println!("\n[ Chemins portables ]");
    let c1 = config.repertoire.join("app.log");
    let c2 = config.repertoire.join("subdir").join("config.toml");
    println!("  Path::join : {}", c1.display());
    println!("  Jointure imbriquée : {}", c2.display());

    // Nettoyage
    let _ = fs::remove_dir_all(&tmp_dir);
}

// ============================================================
// Tests unitaires
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn dir_temp() -> PathBuf {
        let p = PathBuf::from(format!("/tmp/test_logs_{}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn config_test(dir: &Path) -> RotationConfig {
        RotationConfig {
            repertoire:    dir.to_path_buf(),
            prefixe:       "test".to_string(),
            taille_max_ko: 1,
            max_fichiers:  2,
        }
    }

    fn entree_test(n: u64) -> EntreeLog {
        EntreeLog { timestamp: n, niveau: NiveauLog::INFO, module: "test".into(), message: format!("msg {}", n) }
    }

    #[test]
    fn test_ecrire_log_cree_fichier() {
        let dir = dir_temp();
        let chemin = dir.join("app.log");
        ecrire_log(&chemin, &entree_test(1)).unwrap();
        assert!(chemin.exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_ecrire_log_format() {
        let dir = dir_temp();
        let chemin = dir.join("app.log");
        let e = EntreeLog { timestamp: 42, niveau: NiveauLog::ERROR, module: "auth".into(), message: "test".into() };
        ecrire_log(&chemin, &e).unwrap();
        let contenu = fs::read_to_string(&chemin).unwrap();
        assert!(contenu.contains("[42]"));
        assert!(contenu.contains("ERROR"));
        assert!(contenu.contains("auth"));
        assert!(contenu.contains("test"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_ecrire_log_append() {
        let dir = dir_temp();
        let chemin = dir.join("app.log");
        ecrire_log(&chemin, &entree_test(1)).unwrap();
        ecrire_log(&chemin, &entree_test(2)).unwrap();
        let lignes: Vec<_> = fs::read_to_string(&chemin).unwrap()
            .lines().map(|s| s.to_string()).collect();
        assert_eq!(lignes.len(), 2);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_lire_derniers_logs_borne() {
        let dir = dir_temp();
        let chemin = dir.join("app.log");
        for i in 0..10 {
            ecrire_log(&chemin, &entree_test(i)).unwrap();
        }
        let derniers = lire_derniers_logs(&chemin, 3).unwrap();
        assert_eq!(derniers.len(), 3);
        // Les 3 derniers doivent correspondre aux timestamps 7, 8, 9
        assert!(derniers[2].contains("msg 9"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_lire_derniers_logs_moins_que_n() {
        let dir = dir_temp();
        let chemin = dir.join("app.log");
        ecrire_log(&chemin, &entree_test(1)).unwrap();
        ecrire_log(&chemin, &entree_test(2)).unwrap();
        let derniers = lire_derniers_logs(&chemin, 10).unwrap();
        assert_eq!(derniers.len(), 2); // seulement 2 lignes disponibles
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_chemin_log_courant() {
        let dir = dir_temp();
        let config = config_test(&dir);
        let chemin = chemin_log_courant(&config);
        assert_eq!(chemin, dir.join("test.log"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_rotation_pas_declenchee_si_petite() {
        let dir = dir_temp();
        let config = config_test(&dir);
        let chemin = chemin_log_courant(&config);
        // Écrire très peu (bien en dessous de 1 Ko)
        ecrire_log(&chemin, &entree_test(1)).unwrap();
        verifier_rotation(&config).unwrap();
        // Le fichier courant doit encore exister (pas de rotation)
        assert!(chemin.exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_rotation_renomme_si_grande() {
        let dir = dir_temp();
        let config = RotationConfig {
            repertoire: dir.clone(),
            prefixe: "test".to_string(),
            taille_max_ko: 0, // 0 Ko = rotation immédiate
            max_fichiers: 5,
        };
        let chemin = chemin_log_courant(&config);
        ecrire_log(&chemin, &entree_test(1)).unwrap();
        verifier_rotation(&config).unwrap();
        // Le fichier courant ne doit plus exister (renommé en archive)
        assert!(!chemin.exists());
        // Une archive doit exister
        let archives = lister_archives_triees(&config).unwrap();
        assert!(!archives.is_empty());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_lister_archives_triees_par_date() {
        let dir = dir_temp();
        let config = config_test(&dir);
        // Créer quelques faux fichiers d'archive
        fs::write(dir.join("test_100.log"), "a").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::write(dir.join("test_200.log"), "b").unwrap();
        let archives = lister_archives_triees(&config).unwrap();
        assert_eq!(archives.len(), 2);
        // La plus récente (test_200) doit être en premier
        assert!(archives[0].to_string_lossy().contains("200"));
        fs::remove_dir_all(&dir).unwrap();
    }
}

