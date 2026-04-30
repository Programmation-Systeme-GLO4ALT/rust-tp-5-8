use std::io::{self, BufRead};

use crate::error::ReadError;
use crate::readable::Readable;

/// Lit toutes les lignes depuis stdin jusqu'à EOF (Ctrl+D / Ctrl+Z).
pub struct StdinSource {
    exhausted: bool,
}

impl StdinSource {
    pub fn new() -> Self {
        Self { exhausted: false }
    }
}

impl Readable for StdinSource {
    fn source_name(&self) -> &str {
        "<stdin>"
    }

    fn read_content(&mut self) -> Result<String, ReadError> {
        if self.exhausted {
            return Err(ReadError::EmptySource);
        }

        let stdin = io::stdin();
        let mut lines = Vec::new();

        for line_result in stdin.lock().lines() {
            lines.push(line_result?);
        }

        self.exhausted = true;

        if lines.is_empty() {
            return Err(ReadError::EmptySource);
        }

        Ok(lines.join("\n"))
    }

    fn is_exhausted(&self) -> bool {
        self.exhausted
    }
}
