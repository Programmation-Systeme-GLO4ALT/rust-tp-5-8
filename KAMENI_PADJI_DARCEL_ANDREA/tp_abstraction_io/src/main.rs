use std::fs::File;
use std::io::{self, Cursor};

trait Readable {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
}

impl Readable for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        std::io::Read::read(self, buf)
    }
}

impl Readable for Cursor<Vec<u8>> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        std::io::Read::read(self, buf)
    }
}

impl Readable for io::Stdin {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        std::io::Read::read(self, buf)
    }
}

fn read_all<R: Readable>(reader: &mut R) -> io::Result<String> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];

    loop {
        let count = reader.read(&mut temp)?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..count]);
    }

    String::from_utf8(buffer)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn read_from_dynamic(reader: &mut dyn Readable) -> io::Result<String> {
    let mut buffer = Vec::new();
    let mut temp = [0u8; 1024];

    loop {
        let count = reader.read(&mut temp)?;
        if count == 0 {
            break;
        }
        buffer.extend_from_slice(&temp[..count]);
    }

    String::from_utf8(buffer)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
}

fn main() -> io::Result<()> {
    let sample_path = "sample.txt";
    let file = File::open(sample_path)?;

    let memory_buffer = Cursor::new("Lecture depuis un buffer mémoire.\n".as_bytes().to_vec());
    let mut generic_reader = Cursor::new("Lecture générique statique.\n".as_bytes().to_vec());

    let static_content = read_all(&mut generic_reader)?;
    println!("Contenu lu par la fonction statique generic:\n{}", static_content);

    let mut readers: Vec<Box<dyn Readable>> = Vec::new();
    readers.push(Box::new(file));
    readers.push(Box::new(memory_buffer));

    if std::env::args().any(|arg| arg == "--stdin") {
        let stdin = io::stdin();
        readers.push(Box::new(stdin));
    }

    for (index, reader) in readers.iter_mut().enumerate() {
        let content = read_from_dynamic(reader.as_mut())?;
        println!("--- Lecteur dynamique {} ---\n{}", index + 1, content);
    }

    Ok(())
}
