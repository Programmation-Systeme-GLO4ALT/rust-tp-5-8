use std::collections::BTreeMap;
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("usage: cargo run -- <fichier>")]
    MissingPath,
    #[error("impossible de lire le fichier `{path}`: {source}")]
    ReadFile {
        path: String,
        source: std::io::Error,
    },
    #[error("ligne {line}: format invalide, attendu `cle=valeur`")]
    InvalidLine { line: usize },
    #[error("ligne {line}: cle vide")]
    EmptyKey { line: usize },
    #[error("ligne {line}: cle en double `{key}`")]
    DuplicateKey { line: usize, key: String },
}

fn parse_kv_content(content: &str) -> Result<BTreeMap<String, String>, ParseError> {
    let mut values = BTreeMap::new();

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or(ParseError::InvalidLine { line: line_number })?;

        let key = key.trim();
        let value = value.trim();

        if key.is_empty() {
            return Err(ParseError::EmptyKey { line: line_number });
        }

        if values
            .insert(key.to_string(), value.to_string())
            .is_some()
        {
            return Err(ParseError::DuplicateKey {
                line: line_number,
                key: key.to_string(),
            });
        }
    }

    Ok(values)
}

fn parse_kv_file(path: &str) -> Result<BTreeMap<String, String>, ParseError> {
    let content = fs::read_to_string(path).map_err(|source| ParseError::ReadFile {
        path: path.to_string(),
        source,
    })?;

    parse_kv_content(&content)
}

fn main() {
    let path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("{}", ParseError::MissingPath);
            std::process::exit(1);
        }
    };

    match parse_kv_file(&path) {
        Ok(values) => {
            for (key, value) in values {
                println!("{key} = {value}");
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ok() {
        let content = "nom=Rust\nversion = 1.0\n# commentaire\nlangue = FR\n";
        let values = parse_kv_content(content).expect("le parse doit reussir");

        assert_eq!(values.get("nom"), Some(&"Rust".to_string()));
        assert_eq!(values.get("version"), Some(&"1.0".to_string()));
        assert_eq!(values.get("langue"), Some(&"FR".to_string()));
    }

    #[test]
    fn parse_invalid_line() {
        let error = parse_kv_content("ligne_sans_egal").expect_err("une erreur est attendue");

        match error {
            ParseError::InvalidLine { line } => assert_eq!(line, 1),
            _ => panic!("mauvaise erreur"),
        }
    }

    #[test]
    fn parse_empty_key() {
        let error = parse_kv_content(" = valeur").expect_err("une erreur est attendue");

        match error {
            ParseError::EmptyKey { line } => assert_eq!(line, 1),
            _ => panic!("mauvaise erreur"),
        }
    }

    #[test]
    fn parse_duplicate_key() {
        let error = parse_kv_content("nom=Rust\nnom=Langage").expect_err("une erreur est attendue");

        match error {
            ParseError::DuplicateKey { line, key } => {
                assert_eq!(line, 2);
                assert_eq!(key, "nom");
            }
            _ => panic!("mauvaise erreur"),
        }
    }
}
