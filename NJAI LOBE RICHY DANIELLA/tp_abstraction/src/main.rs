
// Le Trait (Interface)
trait Readable {
    fn lire(&self) -> String;
}

// Implémentation 1 : Fichier simulé
struct Fichier { nom: String }
impl Readable for Fichier {
    fn lire(&self) -> String {
        format!("[Fichier: {}] Contenu lu sur le disque.", self.nom)
    }
}

// Implémentation 2 : Mémoire vive
struct Buffer { texte: String }
impl Readable for Buffer {
    fn lire(&self) -> String {
        format!("[Buffer] Données en mémoire : {}", self.texte)
    }
}

// Fonction générique statique (Utilise T)
fn afficher_statique<T: Readable>(source: T) {
    println!("Lecture Statique : {}", source.lire());
}

fn main() {
    println!("--- Test du TP 2 (Abstraction) ---");

    let mon_fichier = Fichier { nom: String::from("devoir.txt") };
    let ma_memoire = Buffer { texte: String::from("Calcul en cours...") };
    afficher_statique(ma_memoire);
    // 1. Utilisation de la fonction générique
    afficher_statique(mon_fichier);

    // 2. Collection dynamique Box<dyn Readable>
    // On mélange différents types dans une seule liste
    let mut liste_sources: Vec<Box<dyn Readable>> = Vec::new();
    
    liste_sources.push(Box::new(Fichier { nom: String::from("notes.log") }));
    liste_sources.push(Box::new(Buffer { texte: String::from("Alerte système") }));

    for source in liste_sources {
        println!("Lecture Dynamique : {}", source.lire());
    }
}