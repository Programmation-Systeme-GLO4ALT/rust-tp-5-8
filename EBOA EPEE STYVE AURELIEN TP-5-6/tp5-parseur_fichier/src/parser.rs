use std::collections::HashMap;
use std::fs;

use crate::error::ParseError;

pub fn parse_file(path: &str) -> Result<HashMap<String, String>, ParseError> {
    let content = fs::read_to_string(path)?; //  propagation erreur

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