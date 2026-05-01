use std::io::{self, BufRead, BufReader, Read};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Erreur I/O : {0}")]
    Io(#[from] io::Error),
    #[error("Ligne mal formée : {0}")]
    MalformedLine(String),
}

// Utilisation d'une abstraction générique avec le trait Read
// Cela permet de lire depuis un fichier, le réseau, ou la mémoire !
fn parse_generic<R: Read>(source: R) -> Result<Vec<(String, String)>, ParserError> {
    let reader = BufReader::new(source);
    let mut items = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        
        if let Some((key, value)) = line.split_once('=') {
            items.push((key.trim().to_string(), value.trim().to_string()));
        } else {
            return Err(ParserError::MalformedLine(line));
        }
    }
    
    Ok(items)
}

fn main() {
    println!("--- Exécution du TP6 : Abstraction I/O ---");
    
    // Test avec une chaîne de caractères directement en mémoire
    let mock_data = "host = localhost\nport = 8080\n# Ceci est un commentaire\nuser = admin";
    
    match parse_generic(mock_data.as_bytes()) {
        Ok(data) => {
            println!("Données extraites avec succès depuis le buffer mémoire :");
            for (k, v) in data {
                println!("Clé: {}, Valeur: {}", k, v);
            }
        }
        Err(e) => eprintln!("Erreur lors du parsing : {}", e),
    }
}