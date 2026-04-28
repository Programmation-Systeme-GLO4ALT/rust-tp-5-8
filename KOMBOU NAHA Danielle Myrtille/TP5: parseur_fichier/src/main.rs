mod parser;
mod error;

use parser::parse_file;

fn main() {
    match parse_file("config.txt") {
        Ok(data) => {
            println!("Données parsées :");
            for (k, v) in data {
                println!("{} = {}", k, v);
            }
        }
        Err(e) => {
            eprintln!("Erreur : {}", e);
        }
    }
}