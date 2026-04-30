mod error;
mod readable;
mod sources;
mod processor;

use sources::{FileSource, MemorySource, StdinSource};
use processor::{process_static, process_all};

fn main() {
    // ── 1. Dispatch STATIQUE ─────────────────────────────────────────────
    println!("=== Dispatch statique (générique) ===\n");

    let mut mem = MemorySource::new(
        "buffer mémoire",
        "Bonjour depuis la mémoire !\nLigne deux.\nLigne trois.",
    );
    if let Err(e) = process_static(&mut mem) {
        eprintln!("Erreur : {e}");
    }

    let mut file = FileSource::new("sample.txt");
    if let Err(e) = process_static(&mut file) {
        eprintln!("Erreur : {e}");
    }

    // ── 2. Collection DYNAMIQUE hétérogène ──────────────────────────────
    println!("\n=== Collection dynamique (Box<dyn Readable>) ===\n");

    let mut sources: Vec<Box<dyn Readable>> = vec![
        Box::new(MemorySource::new("source A", "Alpha\nBeta\nGamma")),
        Box::new(MemorySource::new("source B", "Un seul bloc de texte.")),
        Box::new(FileSource::new("sample.txt")),
        // Source déjà épuisée → doit retourner EmptySource
        Box::new({
            let mut s = MemorySource::new("source épuisée", "données");
            let _ = s.read_content(); // consommer
            s
        }),
    ];

    process_all(&mut sources);

    // ── 3. Stdin (optionnel, activé si argument --stdin passé) ──────────
    if std::env::args().any(|a| a == "--stdin") {
        println!("\n=== Lecture depuis stdin (Ctrl+D pour terminer) ===\n");
        let mut stdin_src = StdinSource::new();
        if let Err(e) = process_static(&mut stdin_src) {
            eprintln!("Erreur stdin : {e}");
        }
    }
}
