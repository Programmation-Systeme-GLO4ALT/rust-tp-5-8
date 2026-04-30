use std::fs;
use std::path::PathBuf;

use crate::error::ReadError;
use crate::readable::Readable;

/// Lit un fichier depuis le disque.
pub struct FileSource {
    path: PathBuf,
    exhausted: bool,
}

impl FileSource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            exhausted: false,
        }
    }
}

impl Readable for FileSource {
    fn source_name(&self) -> &str {
        self.path.to_str().unwrap_or("<chemin non-UTF8>")
    }

    fn read_content(&mut self) -> Result<String, ReadError> {
        if self.exhausted {
            return Err(ReadError::EmptySource);
        }
        let content = fs::read_to_string(&self.path)?;
        self.exhausted = true;
        Ok(content)
    }

    fn is_exhausted(&self) -> bool {
        self.exhausted
    }
}
