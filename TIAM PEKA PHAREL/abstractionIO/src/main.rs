use std::fs::File;
use std::io::{self, BufRead, BufReader};

// 1. Définition du trait
pub trait Readable {
    fn read_line(&mut self) -> Option<String>;
}

// 2. Implémentation pour Fichier
pub struct FileReader {
    reader: BufReader<File>,
}

impl FileReader {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(FileReader {
            reader: BufReader::new(file),
        })
    }
}

impl Readable for FileReader {
    fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                // On enlève le \n ou \r\n final
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(line)
            }
            Err(_) => None, // En cas d'erreur I/O, on arrête la lecture
        }
    }
}

// 3. Implémentation pour Mémoire
pub struct MemoryReader {
    lines: Vec<String>,
    index: usize,
}

impl MemoryReader {
    pub fn new(lines: Vec<String>) -> Self {
        MemoryReader { lines, index: 0 }
    }
}

impl Readable for MemoryReader {
    fn read_line(&mut self) -> Option<String> {
        if self.index < self.lines.len() {
            let line = self.lines[self.index].clone();
            self.index += 1;
            Some(line)
        } else {
            None
        }
    }
}

// 4. Implémentation pour Stdin
pub struct StdinReader {
    reader: BufReader<io::Stdin>,
}

impl StdinReader {
    pub fn new() -> Self {
        StdinReader {
            reader: BufReader::new(io::stdin()),
        }
    }
}

impl Readable for StdinReader {
    fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                if line.ends_with('\n') {
                    line.pop();
                    if line.ends_with('\r') {
                        line.pop();
                    }
                }
                Some(line)
            }
            Err(_) => None,
        }
    }
}

// 5. Fonction générique statique (Dispatch statique)
fn count_lines_and_chars<R: Readable + ?Sized>(source: &mut R) -> (usize, usize) {
    let mut line_count = 0;
    let mut char_count = 0;

    while let Some(line) = source.read_line() {
        line_count += 1;
        char_count += line.chars().count();
    }

    (line_count, char_count)
}

// 6. Programme principal
fn main() -> io::Result<()> {
    println!("=== TRAITEMENT STATIQUE ===");
    
    // Test avec fichier
    let mut file_reader = FileReader::new("Cargo.toml")?;
    let (lines, chars) = count_lines_and_chars(&mut file_reader);
    println!("Fichier Cargo.toml : {} lignes, {} caractères", lines, chars);

    // Test avec mémoire
    let mem_data = vec![
        "Hello".to_string(),
        "World".to_string(),
        "Rust".to_string(),
    ];
    let mut mem_reader = MemoryReader::new(mem_data);
    let (lines, chars) = count_lines_and_chars(&mut mem_reader);
    println!("Buffer Mémoire : {} lignes, {} caractères", lines, chars);

    println!("\n=== TRAITEMENT DYNAMIQUE (COLLECTION HÉTÉROGÈNE) ===");
    
    // Création d'une collection hétérogène de sources
    let mut sources: Vec<Box<dyn Readable>> = vec![
        Box::new(MemoryReader::new(vec!["Ligne dyn 1".to_string(), "Ligne dyn 2".to_string()])),
        Box::new(FileReader::new("Cargo.toml")?),
        // Box::new(StdinReader::new()), // Décommenter pour tester l'entrée standard
    ];

    // Pour chaque source, on utilise la même logique générique
    for (i, source) in sources.iter_mut().enumerate() {
        let (lines, chars) = count_lines_and_chars(source.as_mut());
        println!("Source #{} : {} lignes, {} caractères", i, lines, chars);
    }

    Ok(())
}