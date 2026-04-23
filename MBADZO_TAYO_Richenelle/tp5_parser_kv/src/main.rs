use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("séparateur '=' manquant à la ligne : '{0}'")]
    MissingSeparator(String),

    #[error("clé vide à la ligne : '{0}'")]
    EmptyKey(String),

    #[error("erreur de lecture fichier : {0}")]
    IoError(#[from] std::io::Error),
}

fn parse_line(line: &str) -> Result<(&str, &str), ParseError> {
    let pos = line
        .find('=')
        .ok_or_else(|| ParseError::MissingSeparator(line.to_string()))?;

    let key = &line[..pos];
    let value = &line[pos + 1..];

    if key.is_empty() {
        return Err(ParseError::EmptyKey(line.to_string()));
    }

    Ok((key, value))
}

fn parse_file(path: &str) -> Result<HashMap<String, String>, ParseError> {
    let content = fs::read_to_string(path)?;
    let mut map = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) = parse_line(line)?;
        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
}

const CONFIG_PATH: &str = "src/config.txt";

fn main() {
    match parse_file(CONFIG_PATH) {
        Ok(map) => {
            println!("Parsing réussi ! {} entrées trouvées :", map.len());
            for (key, value) in &map {
                println!("  {} = {}", key, value);
            }
        }
        Err(e) => eprintln!("Erreur : {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ligne_valide() {
        let (k, v) = parse_line("nom=Alice").unwrap();
        assert_eq!(k, "nom");
        assert_eq!(v, "Alice");
    }

    #[test]
    fn test_sans_separateur() {
        let result = parse_line("pas-de-egal");
        assert!(matches!(result, Err(ParseError::MissingSeparator(_))));
    }

    #[test]
    fn test_cle_vide() {
        let result = parse_line("=valeur");
        assert!(matches!(result, Err(ParseError::EmptyKey(_))));
    }
}
