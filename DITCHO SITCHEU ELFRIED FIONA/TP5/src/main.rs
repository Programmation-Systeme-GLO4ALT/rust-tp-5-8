use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseurError {
    #[error("Impossible de lire le fichier '{chemin}' : {source}")]
    LectureFichier {
        chemin: String,
        #[source]
        source: io::Error,
    },
    #[error("Ligne {numero} mal formée (séparateur '=' manquant) : «{contenu}»")]
    LigneMalFormee { numero: usize, contenu: String },
    #[error("Ligne {numero} : la clé ne peut pas être vide")]
    CleVide { numero: usize },
    #[error("Ligne {numero} : la valeur de '{cle}' n'est pas un entier valide : {source}")]
    ValeurEntierInvalide {
        numero: usize,
        cle: String,
        #[source]
        source: ParseIntError,
    },
    #[error("Clé obligatoire '{cle}' absente du fichier")]
    CleObligatoireAbsente { cle: String },
}

#[derive(Debug)]
pub struct Config {
    pub paires: HashMap<String, String>,
}

impl Config {
    pub fn get(&self, cle: &str) -> Result<&str, ParseurError> {
        self.paires
            .get(cle)
            .map(|v| v.as_str())
            .ok_or_else(|| ParseurError::CleObligatoireAbsente {
                cle: cle.to_string(),
            })
    }

    pub fn get_i64(&self, cle: &str) -> Result<i64, ParseurError> {
        let valeur = self.get(cle)?;
        valeur.parse::<i64>().map_err(|e| ParseurError::ValeurEntierInvalide {
            numero: 0,
            cle: cle.to_string(),
            source: e,
        })
    }
}

fn parser_ligne(ligne: &str, numero: usize) -> Result<Option<(String, String)>, ParseurError> {
    let ligne = ligne.trim();
    if ligne.is_empty() || ligne.starts_with('#') {
        return Ok(None);
    }
    let mut parties = ligne.splitn(2, '=');
    let cle_brute = parties.next().ok_or_else(|| ParseurError::LigneMalFormee {
        numero,
        contenu: ligne.to_string(),
    })?;
    let valeur_brute = parties.next().ok_or_else(|| ParseurError::LigneMalFormee {
        numero,
        contenu: ligne.to_string(),
    })?;
    let cle = cle_brute.trim().to_string();
    let valeur = valeur_brute.trim().to_string();
    if cle.is_empty() {
        return Err(ParseurError::CleVide { numero });
    }
    Ok(Some((cle, valeur)))
}

pub fn parser_contenu(contenu: &str) -> Result<Config, ParseurError> {
    let mut paires = HashMap::new();
    for (idx, ligne) in contenu.lines().enumerate() {
        let numero = idx + 1;
        if let Some((cle, valeur)) = parser_ligne(ligne, numero)? {
            paires.insert(cle, valeur);
        }
    }
    Ok(Config { paires })
}

pub fn parser_fichier(chemin: &str) -> Result<Config, ParseurError> {
    let contenu = fs::read_to_string(chemin).map_err(|e| ParseurError::LectureFichier {
        chemin: chemin.to_string(),
        source: e,
    })?;
    parser_contenu(&contenu)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════");
    println!("  TP5 — Parseur clé=valeur avec thiserror  ");
    println!("═══════════════════════════════════════════\n");

    println!("▶ Test 1 : parsing d'un contenu valide");
    let contenu_valide = "# Config\nhote = 127.0.0.1\nport = 8080\nmax_cnx = 100\n";
    let config = parser_contenu(contenu_valide)?;
    println!("  Config parsée : {} entrées", config.paires.len());
    let hote = config.get("hote")?;
    let port = config.get_i64("port")?;
    println!("  hote = {}", hote);
    println!("  port = {}", port);

    println!("\n▶ Test 2 : écriture puis lecture d'un fichier disque");
    let chemin = "tp5_config.conf";
    {
        let mut fichier = fs::File::create(chemin)?;
        writeln!(fichier, "# Config générée par TP5")?;
        writeln!(fichier, "version = 2")?;
        writeln!(fichier, "url = http://localhost:8080/api")?;
    }
    let config_fichier = parser_fichier(chemin)?;
    let version = config_fichier.get_i64("version")?;
    let url = config_fichier.get("url")?;
    println!("  version = {}", version);
    println!("  url     = {}", url);

    println!("\n▶ Test 3 : démonstration des cas d'erreur\n");

    match parser_contenu("cle_sans_egal\n") {
        Err(e) => println!("  [OK] Erreur capturée → {}", e),
        Ok(_)  => println!("  [INATTENDU]"),
    }
    match parser_contenu("= valeur_orpheline\n") {
        Err(e) => println!("  [OK] Erreur capturée → {}", e),
        Ok(_)  => println!("  [INATTENDU]"),
    }
    let config_petite = parser_contenu("a = 1\n")?;
    match config_petite.get("b") {
        Err(e) => println!("  [OK] Erreur capturée → {}", e),
        Ok(_)  => println!("  [INATTENDU]"),
    }
    let config_bad = parser_contenu("port = not_a_number\n")?;
    match config_bad.get_i64("port") {
        Err(e) => {
            println!("  [OK] Erreur capturée → {}", e);
            if let Some(cause) = std::error::Error::source(&e) {
                println!("       Cause : {}", cause);
            }
        }
        Ok(_) => println!("  [INATTENDU]"),
    }
    match parser_fichier("fichier_inexistant_tp5.conf") {
        Err(e) => println!("  [OK] Erreur capturée → {}", e),
        Ok(_)  => println!("  [INATTENDU]"),
    }

    println!("\n▶ Test 4 : valeur contenant '='");
    let contenu_complex = "url = http://example.com/path?a=1&b=2\nnom =  Mon App  \n";
    let cfg = parser_contenu(contenu_complex)?;
    for (k, v) in &cfg.paires {
        println!("  {:25} => «{}»", k, v);
    }

    println!("\n✅  TP5 terminé — zéro unwrap() !");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ligne_normale() {
        let res = parser_ligne("cle = valeur", 1).unwrap();
        assert_eq!(res, Some(("cle".to_string(), "valeur".to_string())));
    }
    #[test]
    fn test_ligne_commentaire() {
        assert_eq!(parser_ligne("# commentaire", 1).unwrap(), None);
    }
    #[test]
    fn test_ligne_vide() {
        assert_eq!(parser_ligne("   ", 1).unwrap(), None);
    }
    #[test]
    fn test_ligne_sans_egal() {
        let res = parser_ligne("pas_de_separateur", 3);
        assert!(matches!(res, Err(ParseurError::LigneMalFormee { numero: 3, .. })));
    }
    #[test]
    fn test_cle_vide() {
        assert!(matches!(parser_ligne("= valeur", 5), Err(ParseurError::CleVide { numero: 5 })));
    }
    #[test]
    fn test_valeur_avec_egal() {
        let res = parser_ligne("url=http://x.com/p?a=1", 1).unwrap();
        assert_eq!(res, Some(("url".to_string(), "http://x.com/p?a=1".to_string())));
    }
    #[test]
    fn test_get_i64_valide() {
        let cfg = parser_contenu("port = 8080\n").unwrap();
        assert_eq!(cfg.get_i64("port").unwrap(), 8080i64);
    }
    #[test]
    fn test_get_i64_invalide() {
        let cfg = parser_contenu("port = abc\n").unwrap();
        assert!(cfg.get_i64("port").is_err());
    }
    #[test]
    fn test_cle_absente() {
        let cfg = parser_contenu("a = 1\n").unwrap();
        assert!(matches!(cfg.get("b"), Err(ParseurError::CleObligatoireAbsente { .. })));
    }
    #[test]
    fn test_multi_lignes() {
        let cfg = parser_contenu("a=1\nb=2\n# commentaire\nc=3\n").unwrap();
        assert_eq!(cfg.paires.len(), 3);
    }
}
