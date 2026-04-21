use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read};
use thiserror::Error;

// TP 5 : Parseur Clé=Valeur avec thiserror

// Définition de l'erreur personnalisée
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Erreur d'entrée/sortie : {0}")] // Génère l'implémentation Display 
    Io(#[from] io::Error),                   // Conversion automatique depuis io::Error 
    
    #[error("Format invalide à la ligne : {0}")]
    FormatInvalide(String),
}

pub fn parser_fichier(chemin: &str) -> Result<HashMap<String, String>, ParseError> {
    let fichier = File::open(chemin)?; // L'opérateur ? propage l'erreur si le fichier est absent 
    // Utilisation d'un buffer pour optimiser les lectures I/O 
    let lecteur = BufReader::new(fichier); 
    let mut map = HashMap::new();

    for ligne_result in lecteur.lines() {
        let ligne = ligne_result?; // Propage l'erreur de lecture ligne par ligne 
        
        if ligne.trim().is_empty() { 
            continue; 
        }

        let parties: Vec<&str> = ligne.splitn(2, '=').collect();
        if parties.len() != 2 {
            return Err(ParseError::FormatInvalide(ligne));
        }

        map.insert(parties[0].trim().to_string(), parties[1].trim().to_string());
    }

    Ok(map)
}



// TP 6 : Abstraction I/O générique

// Un trait définit un ensemble de méthodes qu'un type doit implémenter 
pub trait Readable {
    fn lire_donnees(&self) -> io::Result<String>;
}

//  Fichier Disque
pub struct FichierDisque {
    pub chemin: String,
}
impl Readable for FichierDisque {
    fn lire_donnees(&self) -> io::Result<String> {
        fs::read_to_string(&self.chemin) // Lecture complète du fichier 
    }
}

// Buffer Mémoire
pub struct BufferMemoire {
    pub contenu: String,
}
impl Readable for BufferMemoire {
    fn lire_donnees(&self) -> io::Result<String> {
        Ok(self.contenu.clone())
    }
}

//  Entrée Standard (stdin)
pub struct EntreeStandard;
impl Readable for EntreeStandard {
    fn lire_donnees(&self) -> io::Result<String> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

// Fonction générique STATIQUE 
pub fn lire_statique<T: Readable>(source: &T) -> io::Result<()> {
    let donnees = source.lire_donnees()?;
    println!("=> Lecture statique : {}", donnees.trim());
    Ok(())
}


// Retourne Box<dyn Error> pour unifier les erreurs possibles (IoError et ParseError) [cite: 24, 25]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- DÉBUT TP5 ---");
    // Création dynamique d'un fichier de test pour éviter de crash
    fs::write("config.ini", "utilisateur=admin\nport=8080")?;
    
    // Appel du parseur
    let config = parser_fichier("config.ini")?;
    println!("Configuration parsée : {:?}", config);


    println!("\n--- DÉBUT TP6 ---");
    let source_memoire = BufferMemoire { contenu: "Ceci est en mémoire vive".to_string() };
    let source_fichier = FichierDisque { chemin: "config.ini".to_string() };

    // 1 Test du dispatch statique
    lire_statique(&source_memoire)?;

    // 2 Test du dispatch DYNAMIQUE (vtable) avec une collection hétérogène [cite: 65, 66]
    let collection_dynamique: Vec<Box<dyn Readable>> = vec![
        Box::new(source_memoire),
        Box::new(source_fichier),
    ];

    println!("=> Lecture dynamique depuis la collection :");
    for source in collection_dynamique {
        let contenu = source.lire_donnees()?;
        println!("Contenu trouvé : {}", contenu.trim());
    }

    // Nettoyage du fichier de test
    fs::remove_file("config.ini")?;

    Ok(())
}