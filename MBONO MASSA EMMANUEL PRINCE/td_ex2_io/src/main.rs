// ============================================================
// TD GL4 — Exercice 2 : Abstraction générique d'entrées/sorties
// Séance 6 : Traits & Génériques
// Programmation Système avec Rust — ENSPD GL4 2025-2026
// ============================================================
//
// Parties :
//   2.1 — Définition de traits + implémentation CanalFichier
//   2.2 — Fonctions génériques et trait bounds
//   2.3 — Traits standards (Display, Debug)
// ============================================================

use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::{self, BufReader, BufWriter, Read, Write};

// ============================================================
// PARTIE 2.1 — Définition des traits
// ============================================================

/// Capacité à envoyer des données vers un canal.
pub trait Emetteur {
    type Erreur: fmt::Debug;
    fn envoyer(&mut self, donnees: &[u8]) -> Result<usize, Self::Erreur>;
    fn vider(&mut self) -> Result<(), Self::Erreur>;
}

/// Capacité à recevoir des données depuis un canal.
pub trait Recepteur {
    type Erreur: fmt::Debug;
    fn recevoir(&mut self, tampon: &mut [u8]) -> Result<usize, Self::Erreur>;
}

/// Un canal bidirectionnel est à la fois Emetteur et Recepteur.
/// Q2 — Blanket implementation : tout T qui impl Emetteur + Recepteur
///       impl automatiquement CanalBidirectionnel, sans écrire de code supplémentaire.
pub trait CanalBidirectionnel: Emetteur + Recepteur {}

// Implémentation blanket : automatique pour tout T qui satisfait les deux traits
impl<T> CanalBidirectionnel for T where T: Emetteur + Recepteur {}

// ============================================================
// CanalFichier — canal basé sur deux fichiers (lecture + écriture)
// ============================================================

/// Q1 — Implémentation de Emetteur et Recepteur pour CanalFichier.
/// Utilise BufWriter pour l'émission (écriture groupée) et
/// BufReader pour la réception (lecture efficace avec buffer).
pub struct CanalFichier {
    lecteur: BufReader<File>,
    ecrivain: BufWriter<File>,
}

impl CanalFichier {
    /// Crée un canal fichier depuis deux chemins distincts (lecture / écriture).
    pub fn nouveau(chemin_lecture: &str, chemin_ecriture: &str) -> io::Result<Self> {
        let fichier_lecture = File::open(chemin_lecture)?;
        let fichier_ecriture = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(chemin_ecriture)?;
        Ok(CanalFichier {
            lecteur:   BufReader::new(fichier_lecture),
            ecrivain:  BufWriter::new(fichier_ecriture),
        })
    }
}

impl Emetteur for CanalFichier {
    type Erreur = io::Error;

    fn envoyer(&mut self, donnees: &[u8]) -> Result<usize, io::Error> {
        self.ecrivain.write(donnees)
    }

    fn vider(&mut self) -> Result<(), io::Error> {
        self.ecrivain.flush()
    }
}

impl Recepteur for CanalFichier {
    type Erreur = io::Error;

    fn recevoir(&mut self, tampon: &mut [u8]) -> Result<usize, io::Error> {
        self.lecteur.read(tampon)
    }
}

// ============================================================
// CanalMemoire — canal en mémoire pour les tests
// ============================================================
pub struct CanalMemoire {
    tampon_entrant:  Vec<u8>,   // données disponibles à la réception
    tampon_sortant:  Vec<u8>,   // données envoyées
    position_lecture: usize,
}

impl CanalMemoire {
    pub fn nouveau(donnees_initiales: Vec<u8>) -> Self {
        CanalMemoire {
            tampon_entrant: donnees_initiales,
            tampon_sortant: Vec::new(),
            position_lecture: 0,
        }
    }

    pub fn donnees_envoyees(&self) -> &[u8] {
        &self.tampon_sortant
    }
}

impl Emetteur for CanalMemoire {
    type Erreur = io::Error;

    fn envoyer(&mut self, donnees: &[u8]) -> Result<usize, io::Error> {
        self.tampon_sortant.extend_from_slice(donnees);
        Ok(donnees.len())
    }

    fn vider(&mut self) -> Result<(), io::Error> {
        Ok(()) // pas de buffer à flush en mémoire
    }
}

impl Recepteur for CanalMemoire {
    type Erreur = io::Error;

    fn recevoir(&mut self, tampon: &mut [u8]) -> Result<usize, io::Error> {
        let disponible = &self.tampon_entrant[self.position_lecture..];
        let n = tampon.len().min(disponible.len());
        tampon[..n].copy_from_slice(&disponible[..n]);
        self.position_lecture += n;
        Ok(n)
    }
}

// ============================================================
// PARTIE 2.2 — Fonctions génériques et trait bounds
// ============================================================

/// Q3 — Fonction générique transmettre<C: CanalBidirectionnel>
/// Envoie "PING", attend la réponse et l'affiche.
/// Utilise la syntaxe where pour les trait bounds.
pub fn transmettre<C>(canal: &mut C) -> Result<(), <C as Emetteur>::Erreur>
where
    C: CanalBidirectionnel,
    <C as Emetteur>::Erreur: fmt::Debug,
    <C as Recepteur>::Erreur: Into<<C as Emetteur>::Erreur>,
{
    // Envoyer PING
    let message = b"PING";
    let n = canal.envoyer(message)?;
    canal.vider()?;
    println!("  [TX] Envoyé {} octets : {:?}", n, std::str::from_utf8(message).unwrap_or("?"));

    // Recevoir la réponse
    let mut reponse = [0u8; 64];
    let n_recu = canal.recevoir(&mut reponse).map_err(Into::into)?;
    if n_recu > 0 {
        let texte = std::str::from_utf8(&reponse[..n_recu]).unwrap_or("<binaire>");
        println!("  [RX] Reçu {} octets : {:?}", n_recu, texte);
    } else {
        println!("  [RX] Aucune réponse");
    }
    Ok(())
}

/// Q4 — Fonction diagnostiquer avec trait objects Box<dyn Emetteur<Erreur=io::Error>>
///
/// Pourquoi dyn Trait ici plutôt qu'un générique ?
/// → Parce qu'on veut stocker des types DIFFÉRENTS dans le même Vec.
///   Un Vec<Box<dyn Emetteur<...>>> peut contenir un CanalFichier ET un CanalMemoire.
///   Avec un générique <E: Emetteur>, le Vec ne pourrait contenir qu'un seul type concret.
///   Le dispatch dynamique (vtable) est ici nécessaire car la liste est hétérogène.
pub fn diagnostiquer(canaux: &mut Vec<Box<dyn Emetteur<Erreur = io::Error>>>) {
    let message = b"DIAG";
    for (i, canal) in canaux.iter_mut().enumerate() {
        match canal.envoyer(message) {
            Ok(n)  => println!("  Canal {} : DIAG envoyé ({} octets)", i, n),
            Err(e) => println!("  Canal {} : erreur — {:?}", i, e),
        }
        let _ = canal.vider();
    }
}

/// Q5 — Comparaison static dispatch vs dynamic dispatch :
///
/// Version A — static dispatch (monomorphisation à la compilation) :
///   fn logger_a<E: Emetteur>(canal: &mut E, msg: &str) -> Result<(), E::Erreur>
///   → Le compilateur génère une version spécialisée pour chaque type concret.
///   → Zéro overhead à l'exécution (inlining possible).
///   → Préférable dans les bibliothèques hautes performances où le type est connu.
///
/// Version B — dynamic dispatch (vtable à l'exécution) :
///   fn logger_b(canal: &mut dyn Emetteur<Erreur=io::Error>, msg: &str) -> Result<(), io::Error>
///   → Un seul code compilé, mais lookup vtable à chaque appel.
///   → Préférable quand le type n'est pas connu à la compilation (Vec hétérogène,
///     retour de fonction, configuration runtime).
///   → Aussi utile pour réduire la taille du binaire (pas de duplication de code).
pub fn logger_a<E: Emetteur>(canal: &mut E, msg: &str) -> Result<(), E::Erreur> {
    canal.envoyer(msg.as_bytes())?;
    canal.vider()
}

pub fn logger_b(canal: &mut dyn Emetteur<Erreur = io::Error>, msg: &str) -> Result<(), io::Error> {
    canal.envoyer(msg.as_bytes())?;
    canal.vider()
}

// ============================================================
// PARTIE 2.3 — Traits standards Display et Debug
// ============================================================

/// Q6 — Struct Statistiques avec Display et Debug personnalisés.
#[derive(Clone)]
pub struct Statistiques {
    pub octets_envoyes: u64,
    pub octets_recus:   u64,
    pub erreurs:        u32,
}

/// Debug : format technique pour les développeurs (ex: logs, assertions)
impl fmt::Debug for Statistiques {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Statistiques")
            .field("octets_envoyes", &self.octets_envoyes)
            .field("octets_recus",   &self.octets_recus)
            .field("erreurs",        &self.erreurs)
            .finish()
    }
}

/// Display : format lisible par un humain (ex: interface utilisateur, rapports)
impl fmt::Display for Statistiques {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[STATS] TX={} B / RX={} B / ERR={}",
            self.octets_envoyes, self.octets_recus, self.erreurs
        )
    }
}

// ============================================================
// Programme principal — démonstration
// ============================================================
fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  TD GL4 — Ex2 : Abstraction I/O (Traits/Génériques) ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // --- Canal mémoire bidirectionnel ---
    println!("[ CanalMemoire bidirectionnel ]");
    let mut canal = CanalMemoire::nouveau(b"PONG-REPONSE".to_vec());
    transmettre(&mut canal).unwrap();

    // --- diagnostiquer avec trait objects ---
    println!("\n[ diagnostiquer() — trait objects ]");
    let mut canal2: Box<dyn Emetteur<Erreur = io::Error>> =
        Box::new(CanalMemoire::nouveau(vec![]));
    let mut canal3: Box<dyn Emetteur<Erreur = io::Error>> =
        Box::new(CanalMemoire::nouveau(vec![]));
    let mut liste: Vec<Box<dyn Emetteur<Erreur = io::Error>>> = vec![canal2, canal3];
    diagnostiquer(&mut liste);

    // --- logger_a (static) vs logger_b (dynamic) ---
    println!("\n[ logger_a (static dispatch) ]");
    let mut c = CanalMemoire::nouveau(vec![]);
    logger_a(&mut c, "Message via static dispatch\n").unwrap();
    println!("  Envoyé : {:?}", std::str::from_utf8(c.donnees_envoyees()).unwrap());

    println!("\n[ logger_b (dynamic dispatch) ]");
    let mut c2 = CanalMemoire::nouveau(vec![]);
    logger_b(&mut c2, "Message via dynamic dispatch\n").unwrap();
    println!("  Envoyé : {:?}", std::str::from_utf8(c2.donnees_envoyees()).unwrap());

    // --- Statistiques : Display et Debug ---
    println!("\n[ Statistiques — Display & Debug ]");
    let stats = Statistiques { octets_envoyes: 1024, octets_recus: 2048, erreurs: 0 };
    println!("  Display : {}", stats);
    println!("  Debug   : {:?}", stats);
}

// ============================================================
// Tests unitaires
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canal_memoire_envoyer() {
        let mut canal = CanalMemoire::nouveau(vec![]);
        let n = canal.envoyer(b"hello").unwrap();
        assert_eq!(n, 5);
        assert_eq!(canal.donnees_envoyees(), b"hello");
    }

    #[test]
    fn test_canal_memoire_recevoir() {
        let mut canal = CanalMemoire::nouveau(b"PONG".to_vec());
        let mut buf = [0u8; 10];
        let n = canal.recevoir(&mut buf).unwrap();
        assert_eq!(n, 4);
        assert_eq!(&buf[..4], b"PONG");
    }

    #[test]
    fn test_canal_bidirectionnel_blanket() {
        // CanalMemoire impl Emetteur + Recepteur → impl automatiquement CanalBidirectionnel
        fn accepte_bidir<C: CanalBidirectionnel>(_: &mut C) {}
        let mut canal = CanalMemoire::nouveau(b"test".to_vec());
        accepte_bidir(&mut canal); // doit compiler
    }

    #[test]
    fn test_transmettre() {
        let mut canal = CanalMemoire::nouveau(b"PONG".to_vec());
        assert!(transmettre(&mut canal).is_ok());
        assert_eq!(canal.donnees_envoyees(), b"PING");
    }

    #[test]
    fn test_logger_a_static() {
        let mut canal = CanalMemoire::nouveau(vec![]);
        logger_a(&mut canal, "test").unwrap();
        assert_eq!(canal.donnees_envoyees(), b"test");
    }

    #[test]
    fn test_logger_b_dynamic() {
        let mut canal = CanalMemoire::nouveau(vec![]);
        logger_b(&mut canal, "dyn").unwrap();
        assert_eq!(canal.donnees_envoyees(), b"dyn");
    }

    #[test]
    fn test_statistiques_display() {
        let s = Statistiques { octets_envoyes: 1024, octets_recus: 2048, erreurs: 3 };
        assert_eq!(format!("{}", s), "[STATS] TX=1024 B / RX=2048 B / ERR=3");
    }

    #[test]
    fn test_statistiques_debug() {
        let s = Statistiques { octets_envoyes: 0, octets_recus: 0, erreurs: 0 };
        let d = format!("{:?}", s);
        assert!(d.contains("octets_envoyes"));
        assert!(d.contains("erreurs"));
    }

    #[test]
    fn test_diagnostiquer_envoie_diag() {
        let mut c: Box<dyn Emetteur<Erreur = io::Error>> =
            Box::new(CanalMemoire::nouveau(vec![]));
        let ptr = &*c as *const dyn Emetteur<Erreur = io::Error>;
        let mut liste = vec![c];
        diagnostiquer(&mut liste);
        // Si on arrive ici sans panique, le test passe
    }
}

