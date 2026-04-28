mod readable;
mod readers;

use readable::Readable;
use readers::memory::MemoryBuffer;
use readers::file::FileReader;
use readers::stdin::StdinReader;


fn process_readable<T: Readable>(reader: &mut T) {
    println!("Contenu : {}", reader.read());
}

fn main() {
    // Statique
    let mut mem = MemoryBuffer {
        data: "Hello depuis mémoire".into(),
    };

    process_readable(&mut mem);

    // Dynamique
    let mut readers: Vec<Box<dyn Readable>> = Vec::new();

    readers.push(Box::new(MemoryBuffer {
        data: "Buffer dynamique".into(),
    }));

    readers.push(Box::new(FileReader {
        path: "test.txt".into(),
    }));

    readers.push(Box::new(StdinReader));

    for reader in readers.iter_mut() {
        println!("Lecture dynamique : {}", reader.read());
    }
}