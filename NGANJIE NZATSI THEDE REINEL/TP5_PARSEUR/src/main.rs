use std::collections::HashMap;
use std::env;
use std::fs;
use thiserror::Error;

#[derive(Debug, Error)]
enum ParseError {
    #[error("I/O error while reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid line {line}: expected key=value, got '{content}'")]
    MissingSeparator { line: usize, content: String },
    #[error("invalid line {line}: empty key")]
    EmptyKey { line: usize },
    #[error("duplicate key '{key}' at line {line}")]
    DuplicateKey { line: usize, key: String },
    #[error("usage: tp5_parseur <path-to-file>")]
    MissingPath,
}

fn parse_config(content: &str) -> Result<HashMap<String, String>, ParseError> {
    let mut map = HashMap::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (raw_key, raw_value) = line
            .split_once('=')
            .ok_or_else(|| ParseError::MissingSeparator {
                line: line_no,
                content: line.to_string(),
            })?;

        let key = raw_key.trim();
        if key.is_empty() {
            return Err(ParseError::EmptyKey { line: line_no });
        }

        let value = raw_value.trim();

        if map.insert(key.to_string(), value.to_string()).is_some() {
            return Err(ParseError::DuplicateKey {
                line: line_no,
                key: key.to_string(),
            });
        }
    }

    Ok(map)
}

fn read_and_parse(path: &str) -> Result<HashMap<String, String>, ParseError> {
    let content = fs::read_to_string(path).map_err(|source| ParseError::Io {
        path: path.to_string(),
        source,
    })?;
    parse_config(&content)
}

fn run() -> Result<(), ParseError> {
    let path = env::args().nth(1).ok_or(ParseError::MissingPath)?;
    let parsed = read_and_parse(&path)?;

    for (key, value) in parsed {
        println!("{key}={value}");
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseError, parse_config};

    #[test]
    fn parses_valid_key_value_lines() {
        let src = "host=localhost\nport=8080\n";
        let parsed = parse_config(src);

        match parsed {
            Ok(map) => {
                assert_eq!(map.get("host"), Some(&"localhost".to_string()));
                assert_eq!(map.get("port"), Some(&"8080".to_string()));
            }
            Err(err) => panic!("expected success, got: {err}"),
        }
    }

    #[test]
    fn rejects_lines_without_separator() {
        let src = "host:localhost\n";
        let parsed = parse_config(src);

        match parsed {
            Err(ParseError::MissingSeparator { line, .. }) => assert_eq!(line, 1),
            Err(err) => panic!("unexpected error variant: {err}"),
            Ok(map) => panic!("expected error, got success: {map:?}"),
        }
    }

    #[test]
    fn rejects_empty_key() {
        let src = "=value\n";
        let parsed = parse_config(src);

        match parsed {
            Err(ParseError::EmptyKey { line }) => assert_eq!(line, 1),
            Err(err) => panic!("unexpected error variant: {err}"),
            Ok(map) => panic!("expected error, got success: {map:?}"),
        }
    }

    #[test]
    fn rejects_duplicate_keys() {
        let src = "a=1\na=2\n";
        let parsed = parse_config(src);

        match parsed {
            Err(ParseError::DuplicateKey { line, key }) => {
                assert_eq!(line, 2);
                assert_eq!(key, "a");
            }
            Err(err) => panic!("unexpected error variant: {err}"),
            Ok(map) => panic!("expected error, got success: {map:?}"),
        }
    }
}
