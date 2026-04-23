use std::fs;
use std::io::{self, Read};

/// 1. Le trait (contrat)
trait Readable {
    fn read(&mut self) -> Result<String, io::Error>;
}

/// 2. Implémentation pour un fichier
struct FileReader {
    path: String,
}

impl Readable for FileReader {
    fn read(&mut self) -> Result<String, io::Error> {
        fs::read_to_string(&self.path)
    }
}

/// 3. Implémentation pour un buffer mémoire
struct MemoryReader {
    content: String,
}

impl Readable for MemoryReader {
    fn read(&mut self) -> Result<String, io::Error> {
        Ok(self.content.clone())
    }
}

/// 4. Implémentation pour stdin (clavier)
struct StdinReader;

impl Readable for StdinReader {
    fn read(&mut self) -> Result<String, io::Error> {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        Ok(input)
    }
}

/// 5. Fonction générique (statique)
fn read_static<T: Readable>(mut source: T) {
    match source.read() {
        Ok(content) => println!("(statique)\n{}", content),
        Err(e) => println!("Erreur: {}", e),
    }
}

/// 6. Fonction dynamique
fn read_dynamic(sources: Vec<Box<dyn Readable>>) {
    for mut source in sources {
        match source.read() {
            Ok(content) => println!("(dynamique)\n{}", content),
            Err(e) => println!("Erreur: {}", e),
        }
    }
}

fn main() {
    // ---- STATIQUE ----
    let file = FileReader {
        path: "data.txt".to_string(),
    };

    read_static(file);

    // ---- DYNAMIQUE ----
    let sources: Vec<Box<dyn Readable>> = vec![
        Box::new(FileReader {
            path: "data.txt".to_string(),
        }),
        Box::new(MemoryReader {
            content: "Bonjour depuis la mémoire".to_string(),
        }),
    ];

    read_dynamic(sources);

    
}