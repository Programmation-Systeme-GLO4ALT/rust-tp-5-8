// ============================================================
// TD GL4 — Exercice 1 : Parseur de fichier de configuration
// Séance 5 : Gestion des erreurs
// Programmation Système avec Rust — ENSPD GL4 2025-2026
// ============================================================
//
// Ce fichier répond à toutes les questions des 3 parties :
//   1.1 — Erreurs custom avec thiserror
//   1.2 — Propagation avec l'opérateur ?
//   1.3 — Erreurs dynamiques avec Box<dyn Error>
// ============================================================

use std::collections::HashMap;
use std::io::{self, BufRead};
use std::num::ParseIntError;
use std::path::Path;
use thiserror::Error;

// ============================================================
// PARTIE 1.1 — Erreurs custom avec thiserror
// ============================================================

/// Enum représentant toutes les erreurs possibles du parseur de config.
///
/// Q1 — Différence entre #[source] et #[from] dans thiserror :
///
///   #[source] : marque un champ comme la "cause" de l'erreur.
///     Il est accessible via error.source() mais NE génère PAS
///     d'implémentation From automatique.
///     → On doit convertir manuellement avec .map_err(|e| ConfigError::ValeurNonNumerique { ... })
///
///   #[from] : fait tout ce que #[source] fait + génère automatiquement
///     impl From<TypeErreur> for ConfigError.
///     → L'opérateur ? convertit automatiquement, sans map_err.
///
///   Exemple avec #[source] (manuel) :
///     let val: u16 = s.parse().map_err(|e| ConfigError::ValeurNonNumerique {
///         cle: "port".to_string(),
///         source: e,
///     })?;
///
///   Exemple avec #[from] (automatique) :
///     let _val: u16 = s.parse()?;  // ? suffit si ConfigError impl From<ParseIntError>
///
#[derive(Debug, Error)]
pub enum ConfigError {
    // Variante 1 : fichier introuvable — wrappez std::io::Error
    // On utilise #[from] car on veut que ? convertisse automatiquement
    // les io::Error en ConfigError::FichierIntrouvable.
    #[error("Fichier introuvable : {0}")]
    FichierIntrouvable(#[from] io::Error),

    // Variante 2 : ligne au mauvais format "cle=valeur"
    #[error("Syntaxe invalide à la ligne {ligne} : '{contenu}'")]
    SyntaxeInvalide { ligne: usize, contenu: String },

    // Variante 3 : valeur numérique non parsable
    // #[source] expose la cause via error.source() SANS From automatique
    #[error("Valeur non numérique pour la clé '{cle}' : {source}")]
    ValeurNonNumerique {
        cle: String,
        #[source]
        source: ParseIntError,
    },

    // Variante 4 : clé obligatoire absente
    #[error("Clé obligatoire manquante : '{0}'")]
    CleObligatoire(String),
}

// Q2 — FichierIntrouvable bénéficie de #[from] car :
//   Quand on ouvre un fichier avec File::open(), ça retourne Result<_, io::Error>.
//   Avec #[from], l'opérateur ? convertit automatiquement io::Error →
//   ConfigError::FichierIntrouvable(io_err), sans écrire .map_err() manuellement.
//   C'est la variante la plus pratique quand le type source est unique dans l'enum.

// ============================================================
// PARTIE 1.2 — Propagation avec l'opérateur ?
// ============================================================

/// Lit un fichier de configuration au format clé=valeur.
///
/// Q3 — L'opérateur ? simplifie la propagation d'erreur :
///   Sans ? : match File::open(chemin) { Ok(f) => f, Err(e) => return Err(ConfigError::from(e)) }
///   Avec ?  : File::open(chemin)?   ← beaucoup plus lisible
///
///   Pour que ? fonctionne sur Result<T, io::Error> en retournant ConfigError,
///   ConfigError doit implémenter From<io::Error>. C'est ce que fait #[from] automatiquement.
///   Le trait requis est : impl From<io::Error> for ConfigError
///
pub fn lire_config(chemin: &Path) -> Result<HashMap<String, String>, ConfigError> {
    // ? ici : si le fichier n'existe pas, io::Error est converti en
    // ConfigError::FichierIntrouvable via From<io::Error> (grâce à #[from])
    let fichier = std::fs::File::open(chemin)?;
    let lecteur = io::BufReader::new(fichier);

    let mut map = HashMap::new();

    for (numero, ligne_result) in lecteur.lines().enumerate() {
        let ligne = ligne_result?; // propagation d'erreur I/O de lecture
        let numero_ligne = numero + 1;

        // Ignorer les lignes vides et les commentaires (#)
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }

        // Parser le format cle=valeur
        let mut parties = ligne.splitn(2, '=');
        let cle = parties.next()
            .ok_or_else(|| ConfigError::SyntaxeInvalide {
                ligne: numero_ligne,
                contenu: ligne.to_string(),
            })?
            .trim()
            .to_string();

        let valeur = parties.next()
            .ok_or_else(|| ConfigError::SyntaxeInvalide {
                ligne: numero_ligne,
                contenu: ligne.to_string(),
            })?
            .trim()
            .to_string();

        if cle.is_empty() {
            return Err(ConfigError::SyntaxeInvalide {
                ligne: numero_ligne,
                contenu: ligne.to_string(),
            });
        }

        map.insert(cle, valeur);
    }

    Ok(map)
}

/// Q4 — Lit la clé "port" depuis la map et la convertit en u16.
/// Retourne ConfigError::CleObligatoire si "port" est absent,
/// ou ConfigError::ValeurNonNumerique si la valeur n'est pas un entier valide.
pub fn lire_port(map: &HashMap<String, String>) -> Result<u16, ConfigError> {
    let valeur_str = map
        .get("port")
        .ok_or_else(|| ConfigError::CleObligatoire("port".to_string()))?;

    // parse::<u16>() retourne Result<u16, ParseIntError>
    // .map_err() convertit manuellement ParseIntError → ConfigError::ValeurNonNumerique
    // (car ValeurNonNumerique utilise #[source] et non #[from])
    valeur_str.parse::<u16>().map_err(|e| ConfigError::ValeurNonNumerique {
        cle: "port".to_string(),
        source: e,
    })
}

// ============================================================
// PARTIE 1.3 — Erreurs dynamiques avec Box<dyn Error>
// ============================================================

/// Q5 — Version avec Box<dyn std::error::Error>
///
/// Avantages de Box<dyn Error> :
///   + Très flexible : n'importe quel type d'erreur peut être retourné via ?
///     sans déclarer d'enum. Idéal pour les scripts rapides / prototyping.
///   + Moins de code à écrire (pas besoin de déclarer ConfigError).
///   + Compatible avec toutes les erreurs qui implémentent Error.
///
/// Inconvénients par rapport à une enum typée :
///   - Perd l'information de type à la compilation (type erasure).
///     L'appelant ne peut plus faire match sur le type d'erreur.
///   - Allocation heap supplémentaire (Box).
///   - Moins lisible dans les bibliothèques publiques.
///   - Ne permet pas de distinguer les cas d'erreur pour les gérer différemment.
///
///   → Règle pratique : utiliser Box<dyn Error> dans les binaires/scripts,
///     enum typée dans les bibliothèques partagées.
///
pub fn lire_config_dynamique(
    chemin: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let fichier = std::fs::File::open(chemin)?;
    let lecteur = io::BufReader::new(fichier);
    let mut map = HashMap::new();

    for ligne_result in lecteur.lines() {
        let ligne = ligne_result?;
        let ligne = ligne.trim();
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }
        if let Some((cle, valeur)) = ligne.split_once('=') {
            map.insert(cle.trim().to_string(), valeur.trim().to_string());
        }
    }
    Ok(map)
}

// ============================================================
// Programme principal — démonstration
// ============================================================
fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  TD GL4 — Ex1 : Parseur de Configuration            ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Créer un fichier de config temporaire pour la démo
    let config_valide = "/tmp/config_valide.conf";
    std::fs::write(config_valide,
        "# Configuration du serveur\nhost=localhost\nport=8080\ntimeout=30\nnom=MonApp\n"
    ).unwrap();

    let config_invalide = "/tmp/config_invalide.conf";
    std::fs::write(config_invalide,
        "host=localhost\nligne_sans_egal\nport=abc\n"
    ).unwrap();

    // --- Test 1 : fichier valide ---
    println!("[ Test 1 : Fichier valide ]");
    match lire_config(Path::new(config_valide)) {
        Ok(map) => {
            println!("  Clés lues :");
            let mut cles: Vec<_> = map.keys().collect();
            cles.sort();
            for cle in cles {
                println!("    {} = {}", cle, map[cle]);
            }
            match lire_port(&map) {
                Ok(port) => println!("  Port parsé : {}", port),
                Err(e)   => println!("  Erreur port : {}", e),
            }
        }
        Err(e) => println!("  Erreur : {}", e),
    }

    // --- Test 2 : fichier inexistant ---
    println!("\n[ Test 2 : Fichier inexistant ]");
    match lire_config(Path::new("/tmp/inexistant.conf")) {
        Ok(_)  => println!("  Inattendu : succès"),
        Err(e) => println!("  Erreur attendue : {}", e),
    }

    // --- Test 3 : fichier syntaxe invalide ---
    println!("\n[ Test 3 : Syntaxe invalide ]");
    match lire_config(Path::new(config_invalide)) {
        Ok(_)  => println!("  Inattendu : succès"),
        Err(e) => println!("  Erreur attendue : {}", e),
    }

    // --- Test 4 : port absent ---
    println!("\n[ Test 4 : Clé 'port' absente ]");
    let map_sans_port: HashMap<String, String> =
        [("host".to_string(), "localhost".to_string())].into();
    match lire_port(&map_sans_port) {
        Ok(_)  => println!("  Inattendu : succès"),
        Err(e) => println!("  Erreur attendue : {}", e),
    }

    // --- Test 5 : port non numérique ---
    println!("\n[ Test 5 : Port non numérique ]");
    let map_port_invalide: HashMap<String, String> =
        [("port".to_string(), "abc".to_string())].into();
    match lire_port(&map_port_invalide) {
        Ok(_)  => println!("  Inattendu : succès"),
        Err(e) => {
            println!("  Erreur attendue : {}", e);
            // Accès à la source via #[source]
            if let Some(source) = std::error::Error::source(&e) {
                println!("  Cause (source) : {}", source);
            }
        }
    }

    // --- Test 6 : version Box<dyn Error> ---
    println!("\n[ Test 6 : Box<dyn Error> ]");
    match lire_config_dynamique(Path::new(config_valide)) {
        Ok(map) => println!("  Succès, {} clés lues", map.len()),
        Err(e)  => println!("  Erreur : {}", e),
    }
}

// ============================================================
// Tests unitaires
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fichier_temp(contenu: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        write!(f, "{}", contenu).unwrap();
        f
    }

    #[test]
    fn test_lire_config_ok() {
        let f = fichier_temp("host=localhost\nport=8080\n");
        let map = lire_config(f.path()).unwrap();
        assert_eq!(map["host"], "localhost");
        assert_eq!(map["port"], "8080");
    }

    #[test]
    fn test_lire_config_ignore_commentaires() {
        let f = fichier_temp("# commentaire\nhost=127.0.0.1\n");
        let map = lire_config(f.path()).unwrap();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_lire_config_ignore_lignes_vides() {
        let f = fichier_temp("\n\nhost=ok\n\n");
        let map = lire_config(f.path()).unwrap();
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_lire_config_syntaxe_invalide() {
        let f = fichier_temp("ligne_sans_egal\n");
        let err = lire_config(f.path()).unwrap_err();
        assert!(matches!(err, ConfigError::SyntaxeInvalide { .. }));
    }

    #[test]
    fn test_lire_config_fichier_inexistant() {
        let err = lire_config(Path::new("/tmp/fichier_qui_nexiste_pas_12345.conf"))
            .unwrap_err();
        assert!(matches!(err, ConfigError::FichierIntrouvable(_)));
    }

    #[test]
    fn test_lire_port_ok() {
        let map = [("port".to_string(), "9090".to_string())].into();
        assert_eq!(lire_port(&map).unwrap(), 9090);
    }

    #[test]
    fn test_lire_port_absent() {
        let map = HashMap::new();
        let err = lire_port(&map).unwrap_err();
        assert!(matches!(err, ConfigError::CleObligatoire(_)));
    }

    #[test]
    fn test_lire_port_invalide() {
        let map = [("port".to_string(), "not_a_number".to_string())].into();
        let err = lire_port(&map).unwrap_err();
        assert!(matches!(err, ConfigError::ValeurNonNumerique { .. }));
    }

    #[test]
    fn test_lire_port_hors_plage() {
        // u16 max = 65535, donc 99999 doit échouer
        let map = [("port".to_string(), "99999".to_string())].into();
        assert!(lire_port(&map).is_err());
    }
}

