mod error;
mod parser;

use std::process;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config.txt".to_owned());

    match parser::parse_file(&path) {
        Ok(map) => {
            println!("✓ {} entrée(s) trouvée(s) dans {:?}\n", map.len(), path);

            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for key in keys {
                println!("  {:20} = {}", key, map[key]);
            }
        }
        Err(e) => {
            eprintln!("Erreur : {e}");
            process::exit(1);
        }
    }
}
