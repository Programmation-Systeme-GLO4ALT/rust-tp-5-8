use crate::readable::Readable;
use std::fs;

pub struct FileReader {
    pub path: String,
}

impl Readable for FileReader {
    fn read(&mut self) -> String {
        fs::read_to_string(&self.path).unwrap_or_else(|_| "Erreur lecture fichier".into())
    }
}