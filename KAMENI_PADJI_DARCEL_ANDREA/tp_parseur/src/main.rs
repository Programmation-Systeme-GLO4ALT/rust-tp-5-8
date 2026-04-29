use std::{collections::BTreeMap, env, fs, io, path::Path};

use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("erreur d'E/S: {0}")]
    Io(#[from] io::Error),

    #[error("ligne {line_number} invalide: {line}")]
    MalformedLine { line_number: usize, line: String },

    #[error("clé dupliquée: {0}")]
    DuplicateKey(String),

    #[error("clé vide à la ligne {line_number}")]
    EmptyKey { line_number: usize },
}

fn parse_key_value_file<P: AsRef<Path>>(path: P) -> Result<BTreeMap<String, String>, ParseError> {
    let content = fs::read_to_string(path)?;
    let mut result = BTreeMap::new();

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.splitn(2, '=');
        let key_part = match parts.next() {
            Some(value) => value.trim(),
            None => {
                return Err(ParseError::MalformedLine {
                    line_number,
                    line: line.to_string(),
                })
            }
        };

        let value_part = match parts.next() {
            Some(value) => value.trim(),
            None => {
                return Err(ParseError::MalformedLine {
                    line_number,
                    line: line.to_string(),
                })
            }
        };

        if key_part.is_empty() {
            return Err(ParseError::EmptyKey { line_number });
        }

        if result.insert(key_part.to_string(), value_part.to_string()).is_some() {
            return Err(ParseError::DuplicateKey(key_part.to_string()));
        }
    }

    Ok(result)
}

fn main() -> Result<(), ParseError> {
    let path = match env::args().nth(1) {
        Some(value) => value,
        None => "data.txt".to_string(),
    };
    let pairs = parse_key_value_file(&path)?;

    if pairs.is_empty() {
        println!("Aucune paire clé=valeur trouvée dans {}.", path);
        return Ok(());
    }

    println!("Paires clés=valeurs lues depuis {}:", path);
    for (key, value) in pairs {
        println!("{} = {}", key, value);
    }

    Ok(())
}
