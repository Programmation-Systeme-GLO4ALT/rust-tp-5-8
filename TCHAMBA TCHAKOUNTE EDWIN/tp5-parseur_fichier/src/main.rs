// TP5 — Parseur de fichier de configuration
// Lecture d'un fichier "clé=valeur" avec gestion d'erreurs typée (thiserror)

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;
use std::str::ParseBoolError;

use thiserror::Error;

// --- 1) Définition des erreurs métier -----------------------------------------
//
// On définit un enum d'erreurs propre au parseur. `thiserror` génère
// automatiquement l'implémentation de `std::error::Error` et `Display`
// à partir des attributs `#[error(...)]`.
//
// `#[from]` génère la conversion automatique d'une erreur source vers notre
// enum, ce qui permet d'utiliser l'opérateur `?` directement.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("erreur d'E/S sur le fichier : {0}")]
    Io(#[from] std::io::Error),

    #[error("ligne {ligne} mal formée : '{contenu}' (format attendu : clé=valeur)")]
    LigneMalFormee { ligne: usize, contenu: String },

    #[error("champ obligatoire manquant : '{0}'")]
    ChampManquant(String),

    #[error("valeur invalide pour '{cle}' : '{valeur}' — {raison}")]
    ValeurInvalide {
        cle: String,
        valeur: String,
        raison: String,
    },
}

// Conversions explicites depuis les erreurs de parsing standard.
// On ne peut pas utiliser `#[from]` ici car on veut enrichir l'erreur
// avec le nom du champ (`cle`) et la valeur fautive.
impl ConfigError {
    fn entier(cle: &str, valeur: &str, e: ParseIntError) -> Self {
        ConfigError::ValeurInvalide {
            cle: cle.to_string(),
            valeur: valeur.to_string(),
            raison: format!("entier attendu ({})", e),
        }
    }

    fn booleen(cle: &str, valeur: &str, e: ParseBoolError) -> Self {
        ConfigError::ValeurInvalide {
            cle: cle.to_string(),
            valeur: valeur.to_string(),
            raison: format!("booléen attendu ({})", e),
        }
    }
}

// --- 2) Structure de configuration --------------------------------------------
//
// Représentation typée du fichier de config. C'est notre cible de parsing.
#[derive(Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub debug: bool,
    pub max_connections: u32,
}

// --- 3) Lecture du fichier en HashMap (clé → valeur) --------------------------
use std::collections::HashMap;

fn lire_paires<P: AsRef<Path>>(chemin: P) -> Result<HashMap<String, String>, ConfigError> {
    let fichier = File::open(&chemin)?; // ? convertit io::Error → ConfigError::Io
    let lecteur = BufReader::new(fichier);

    let mut paires = HashMap::new();

    for (idx, ligne) in lecteur.lines().enumerate() {
        let ligne = ligne?; // io::Error possible sur la lecture d'une ligne
        let numero = idx + 1;

        // On ignore les lignes vides et les commentaires.
        let trim = ligne.trim();
        if trim.is_empty() || trim.starts_with('#') {
            continue;
        }

        // split_once renvoie Option<(&str, &str)> : très pratique ici.
        let (cle, valeur) = match trim.split_once('=') {
            Some((c, v)) => (c.trim().to_string(), v.trim().to_string()),
            None => {
                return Err(ConfigError::LigneMalFormee {
                    ligne: numero,
                    contenu: ligne,
                });
            }
        };

        if cle.is_empty() {
            return Err(ConfigError::LigneMalFormee {
                ligne: numero,
                contenu: ligne,
            });
        }

        paires.insert(cle, valeur);
    }

    Ok(paires)
}

// --- 4) Conversion HashMap → Config (typée) -----------------------------------
fn extraire<'a>(paires: &'a HashMap<String, String>, cle: &str) -> Result<&'a str, ConfigError> {
    paires
        .get(cle)
        .map(|s| s.as_str())
        .ok_or_else(|| ConfigError::ChampManquant(cle.to_string()))
}

impl Config {
    pub fn depuis_fichier<P: AsRef<Path>>(chemin: P) -> Result<Self, ConfigError> {
        let paires = lire_paires(chemin)?;

        let host = extraire(&paires, "host")?.to_string();

        let port_str = extraire(&paires, "port")?;
        let port: u16 = port_str
            .parse()
            .map_err(|e| ConfigError::entier("port", port_str, e))?;

        let debug_str = extraire(&paires, "debug")?;
        let debug: bool = debug_str
            .parse()
            .map_err(|e| ConfigError::booleen("debug", debug_str, e))?;

        let max_str = extraire(&paires, "max_connections")?;
        let max_connections: u32 = max_str
            .parse()
            .map_err(|e| ConfigError::entier("max_connections", max_str, e))?;

        Ok(Config {
            host,
            port,
            debug,
            max_connections,
        })
    }
}

// --- 5) Programme principal ---------------------------------------------------
fn main() {
    let chemin = "config.txt";

    match Config::depuis_fichier(chemin) {
        Ok(config) => {
            println!("✓ Configuration chargée depuis '{}' :", chemin);
            println!("  host            = {}", config.host);
            println!("  port            = {}", config.port);
            println!("  debug           = {}", config.debug);
            println!("  max_connections = {}", config.max_connections);
        }
        Err(e) => {
            eprintln!("✗ Erreur : {}", e);
            std::process::exit(1);
        }
    }
}

// --- 6) Tests unitaires -------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn ecrire_temp(nom: &str, contenu: &str) -> std::path::PathBuf {
        let mut chemin = std::env::temp_dir();
        chemin.push(nom);
        let mut f = File::create(&chemin).unwrap();
        f.write_all(contenu.as_bytes()).unwrap();
        chemin
    }

    #[test]
    fn parse_ok() {
        let p = ecrire_temp(
            "tp5_ok.txt",
            "host=example.com\nport=80\ndebug=false\nmax_connections=10\n",
        );
        let cfg = Config::depuis_fichier(&p).unwrap();
        assert_eq!(cfg.host, "example.com");
        assert_eq!(cfg.port, 80);
        assert!(!cfg.debug);
        assert_eq!(cfg.max_connections, 10);
    }

    #[test]
    fn ligne_mal_formee() {
        let p = ecrire_temp("tp5_malforme.txt", "host=localhost\nportSANS_EGAL\n");
        let err = Config::depuis_fichier(&p).unwrap_err();
        assert!(matches!(err, ConfigError::LigneMalFormee { ligne: 2, .. }));
    }

    #[test]
    fn champ_manquant() {
        let p = ecrire_temp("tp5_manquant.txt", "host=h\nport=1\ndebug=true\n");
        let err = Config::depuis_fichier(&p).unwrap_err();
        assert!(matches!(err, ConfigError::ChampManquant(ref c) if c == "max_connections"));
    }

    #[test]
    fn valeur_invalide() {
        let p = ecrire_temp(
            "tp5_invalide.txt",
            "host=h\nport=abc\ndebug=true\nmax_connections=10\n",
        );
        let err = Config::depuis_fichier(&p).unwrap_err();
        assert!(matches!(
            err,
            ConfigError::ValeurInvalide { ref cle, .. } if cle == "port"
        ));
    }

    #[test]
    fn ignore_vide_et_commentaires() {
        let p = ecrire_temp(
            "tp5_commentaires.txt",
            "# commentaire\n\nhost=h\nport=1\ndebug=true\nmax_connections=10\n",
        );
        let cfg = Config::depuis_fichier(&p).unwrap();
        assert_eq!(cfg.host, "h");
    }
}
