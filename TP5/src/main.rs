use std::collections::HashMap;
use std::fs;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Erreur de lecture du fichier")]
    Io(#[from] io::Error),

    #[error("Ligne invalide: {0}")]
    InvalidLine(String),
}

fn parse_file(path: &str) -> Result<HashMap<String, String>, ParseError> {
    let content = fs::read_to_string(path)?; 

    let mut map = HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split('=').collect();

        if parts.len() != 2 {
            return Err(ParseError::InvalidLine(line.to_string()));
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();

        map.insert(key, value);
    }

    Ok(map)
}

fn main() -> Result<(), ParseError> {
    let result = parse_file("data.txt")?;

    for (key, value) in result {
        println!("{} = {}", key, value);
    }

    Ok(())
}