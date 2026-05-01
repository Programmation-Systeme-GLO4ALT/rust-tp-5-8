use std::fs::File;
use std::io::{self, BufRead, BufReader};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Erreur de lecture du fichier : {0}")]
    Io(#[from] io::Error),
    #[error("Format invalide à la ligne : {0}")]
    InvalidFormat(String),
}

fn parse_config(path: &str) -> Result<Vec<(String, String)>, ConfigError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut config = Vec::new();

    for line in reader.lines() {
        let line = line?;
        // On ignore les lignes vides et les commentaires
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(ConfigError::InvalidFormat(line));
        }
        
        config.push((
            parts[0].trim().to_string(),
            parts[1].trim().to_string(),
        ));
    }
    
    Ok(config)
}

fn main() {
    println!("--- Exécution du TP5 : Parseur Clé-Valeur ---");
    // Remarque : Il faudra un fichier "config.txt" réel pour ne pas avoir d'erreur I/O
    match parse_config("config.txt") {
        Ok(data) => println!("Configuration parsée avec succès : {:?}", data),
        Err(e) => eprintln!("Erreur détectée : {}", e),
    }
}