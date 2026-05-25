
use thiserror::Error;
use std::collections::HashMap;

// Définition propre des erreurs (Contrainte : Zéro unwrap)
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Ligne malformée (manque '=') : {0}")]
    FormatError(String),
    #[error("Erreur d'entrée/sortie")]
    IoError(#[from] std::io::Error),
}

// Fonction qui parse le texte
fn parser_donnees(contenu: &str) -> Result<HashMap<String, String>, ParseError> {
    let mut dictionnaire = HashMap::new();

    for ligne in contenu.lines() {
        let ligne_propre = ligne.trim();
        if ligne_propre.is_empty() { continue; }

        // On cherche le '=' sans faire planter le programme
        let parts: Vec<&str> = ligne_propre.splitn(2, '=').collect();
        
        if parts.len() != 2 {
            // Au lieu de unwrap(), on renvoie une erreur proprement
            return Err(ParseError::FormatError(ligne_propre.to_string()));
        }

        dictionnaire.insert(
            parts[0].trim().to_string(), 
            parts[1].trim().to_string()
        );
    }
    Ok(dictionnaire)
}

fn main() {
    // Simulation d'un fichier clé=valeur
    let contenu_fichier = "universite = ecole Nationale Polytechnique\nniveau = 4\nville = Douala";

    match parser_donnees(contenu_fichier) {
        Ok(map) => {
            for (cle, valeur) in map {
                println!("Clé: {} ; Valeur: {}", cle, valeur);
            }
        },
        Err(e) => eprintln!("Erreur de parsing : {}", e),
    }
}
