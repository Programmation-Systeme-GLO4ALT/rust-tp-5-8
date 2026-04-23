use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

// 1. DÉFINITION DES ERREURS AVEC THISERROR
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Erreur I/O : {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Ligne {ligne} : format invalide (attendu 'clé=valeur')")]
    FormatInvalide { ligne: usize },
    #[error("Ligne {ligne} : clé vide")]
    CleVide { ligne: usize },
    #[error("Ligne {ligne} : clé '{cle}' en double")]
    CleDupliquee { ligne: usize, cle: String },
}

// 2. FONCTION PRINCIPALE DE PARSING
pub fn parse_config<P: AsRef<Path>>(chemin: P) -> Result<HashMap<String, String>, ConfigError> {
    let contenu = fs::read_to_string(chemin)?;
    let mut config = HashMap::new();

    for (index, ligne) in contenu.lines().enumerate() {
        let numero = index + 1;
        let ligne = ligne.trim();
        
        if ligne.is_empty() || ligne.starts_with('#') {
            continue;
        }
        
        let pos = ligne.find('=').ok_or(ConfigError::FormatInvalide { ligne: numero })?;
        let cle = ligne[..pos].trim();
        let valeur = ligne[pos + 1..].trim();
        
        if cle.is_empty() {
            return Err(ConfigError::CleVide { ligne: numero });
        }
        
        if config.contains_key(cle) {
            return Err(ConfigError::CleDupliquee {
                ligne: numero,
                cle: cle.to_string(),
            });
        }
        
        config.insert(cle.to_string(), valeur.to_string());
    }
    
    Ok(config)
}

// 3. EXEMPLE D'UTILISATION
fn main() -> Result<(), Box<dyn std::error::Error>> {
    fs::write("config.txt","# Mon fichier de config\nnom=monapp\nport=8080\nhost=localhost\n",)?;
    
    let config = parse_config("config.txt")?;
    
    println!("Configuration parsée avec succès :");
    for (cle, valeur) in &config {
        println!("  {} = {}", cle, valeur);
    }
    
    if let Some(port) = config.get("port") {
        println!("\nLe port configuré est : {}", port);
    }
    
    Ok(())
}