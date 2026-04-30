use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::ParseError;

/// Parse une seule ligne "clé=valeur".
/// Les commentaires (lignes commençant par `#`) sont ignorés.
/// Retourne `None` si la ligne est un commentaire, `Some(Ok(...))` ou `Some(Err(...))` sinon.
pub fn parse_line(
    raw: &str,
    line_number: usize,
) -> Option<Result<(String, String), ParseError>> {
    let trimmed = raw.trim();

    // Ignorer les commentaires
    if trimmed.starts_with('#') {
        return None;
    }

    if trimmed.is_empty() {
        return Some(Err(ParseError::EmptyLine { line_number }));
    }

    // Découper sur le PREMIER '=' uniquement : "url=http://a=b" → ("url", "http://a=b")
    let (key_part, value_part) = match trimmed.split_once('=') {
        Some(pair) => pair,
        None => {
            return Some(Err(ParseError::MissingSeparator {
                line_number,
                content: trimmed.to_owned(),
            }))
        }
    };

    let key = key_part.trim().to_owned();
    let value = value_part.trim().to_owned();

    if key.is_empty() {
        return Some(Err(ParseError::EmptyKey { line_number }));
    }

    Some(Ok((key, value)))
}

/// Lit un fichier et retourne un `HashMap<String, String>`.
/// Toute erreur (I/O ou format) est propagée via `?`.
pub fn parse_file(path: impl AsRef<Path>) -> Result<HashMap<String, String>, ParseError> {
    let content = fs::read_to_string(path)?; // IoError via #[from]

    let mut map = HashMap::new();

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;

        match parse_line(raw_line, line_number) {
            None => continue,
            Some(Ok((key, value))) => {
                map.insert(key, value);
            }
            Some(Err(e)) => return Err(e),
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ligne_normale() {
        let result = parse_line("host = localhost", 1);
        assert!(matches!(result, Some(Ok((ref k, ref v))) if k == "host" && v == "localhost"));
    }

    #[test]
    fn parse_valeur_avec_egal() {
        let result = parse_line("url = http://a.com?x=1", 1);
        assert!(matches!(result, Some(Ok((ref k, ref v))) if k == "url" && v == "http://a.com?x=1"));
    }

    #[test]
    fn commentaire_ignore() {
        let result = parse_line("# je suis un commentaire", 1);
        assert!(result.is_none());
    }

    #[test]
    fn erreur_separateur_manquant() {
        let result = parse_line("sans_egal", 3);
        assert!(matches!(result, Some(Err(ParseError::MissingSeparator { line_number: 3, .. }))));
    }

    #[test]
    fn erreur_ligne_vide() {
        let result = parse_line("   ", 5);
        assert!(matches!(result, Some(Err(ParseError::EmptyLine { line_number: 5 }))));
    }

    #[test]
    fn erreur_cle_vide() {
        let result = parse_line("= valeur_orpheline", 7);
        assert!(matches!(result, Some(Err(ParseError::EmptyKey { line_number: 7 }))));
    }
}
