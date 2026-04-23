use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

trait Readable {
    fn read_text(&mut self) -> io::Result<String>;
}

struct MemoryReadable {
    content: String,
}

struct FileReadable {
    path: PathBuf,
}

struct StdinReadable;

impl Readable for MemoryReadable {
    fn read_text(&mut self) -> io::Result<String> {
        Ok(self.content.clone())
    }
}

impl Readable for FileReadable {
    fn read_text(&mut self) -> io::Result<String> {
        fs::read_to_string(&self.path)
    }
}

impl Readable for StdinReadable {
    fn read_text(&mut self) -> io::Result<String> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim_end().to_string())
    }
}

fn read_static<R: Readable>(reader: &mut R) -> io::Result<String> {
    reader.read_text()
}

fn read_dynamic(readers: &mut [Box<dyn Readable>]) -> io::Result<Vec<String>> {
    let mut results = Vec::new();

    for reader in readers {
        results.push(reader.read_text()?);
    }

    Ok(results)
}

fn main() -> io::Result<()> {
    let mut memory = MemoryReadable {
        content: String::from("Texte venant de la memoire"),
    };

    let static_result = read_static(&mut memory)?;
    println!("Statique: {static_result}");

    let sample_path = PathBuf::from("exemple.txt");
    fs::write(&sample_path, "Texte venant du fichier")?;

    let mut readers: Vec<Box<dyn Readable>> = vec![
        Box::new(MemoryReadable {
            content: String::from("Lecture dynamique en memoire"),
        }),
        Box::new(FileReadable {
            path: sample_path.clone(),
        }),
        Box::new(StdinReadable),
    ];

    print!("Entrez une ligne pour stdin: ");
    io::stdout().flush()?;

    let dynamic_results = read_dynamic(&mut readers)?;

    for (index, value) in dynamic_results.iter().enumerate() {
        println!("Dynamique {}: {}", index + 1, value);
    }

    Ok(())
}
