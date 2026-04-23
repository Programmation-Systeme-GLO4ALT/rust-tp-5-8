use std::fmt;
use std::fs;
use std::io::{self, BufRead};

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    Empty,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReadError::Io(e) => write!(f, "erreur I/O : {}", e),
            ReadError::Empty => write!(f, "source vide"),
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(e: io::Error) -> Self {
        ReadError::Io(e)
    }
}

pub trait Readable {
    fn read_content(&self) -> Result<String, ReadError>;
    fn describe(&self) -> &str {
        "source inconnue"
    }
}

pub struct FileReader {
    path: String,
}

impl FileReader {
    pub fn new(path: &str) -> Self {
        FileReader { path: path.to_string() }
    }
}

impl Readable for FileReader {
    fn read_content(&self) -> Result<String, ReadError> {
        let content = fs::read_to_string(&self.path)?;
        if content.is_empty() {
            return Err(ReadError::Empty);
        }
        Ok(content)
    }
    fn describe(&self) -> &str { "fichier disque" }
}

pub struct MemoryReader {
    content: String,
}

impl MemoryReader {
    pub fn new(content: &str) -> Self {
        MemoryReader { content: content.to_string() }
    }
}

impl Readable for MemoryReader {
    fn read_content(&self) -> Result<String, ReadError> {
        if self.content.is_empty() {
            return Err(ReadError::Empty);
        }
        Ok(self.content.clone())
    }
    fn describe(&self) -> &str { "buffer mémoire" }
}

pub struct StdinReader;

impl Readable for StdinReader {
    fn read_content(&self) -> Result<String, ReadError> {
        let stdin = io::stdin();
        let mut lines = Vec::new();
        for line in stdin.lock().lines() {
            lines.push(line?);
        }
        let content = lines.join("\n");
        if content.is_empty() {
            return Err(ReadError::Empty);
        }
        Ok(content)
    }
    fn describe(&self) -> &str { "stdin" }
}

fn process_static<R: Readable>(reader: &R) {
    println!("*** [statique] Source : {} ***", reader.describe());
    match reader.read_content() {
        Ok(content) => println!("{}", content),
        Err(e) => eprintln!("Erreur : {}", e),
    }
}

fn process_all(readers: &[Box<dyn Readable>]) {
    for (i, reader) in readers.iter().enumerate() {
        println!("*** [dynamique] Reader #{} : {} ***", i + 1, reader.describe());
        match reader.read_content() {
            Ok(content) => println!("{}", content),
            Err(e) => eprintln!("Erreur : {}", e),
        }
    }
}

fn main() {
    let mem = MemoryReader::new("Bonjour depuis la mémoire !");
    process_static(&mem);

    let readers: Vec<Box<dyn Readable>> = vec![
        Box::new(MemoryReader::new("Ligne depuis la mémoire")),
        Box::new(MemoryReader::new("Deuxième buffer")),
    ];
    process_all(&readers);
}
