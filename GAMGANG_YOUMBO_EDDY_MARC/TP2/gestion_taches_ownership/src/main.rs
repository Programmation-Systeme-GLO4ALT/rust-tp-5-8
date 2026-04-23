type Tache = (String, u8, bool);

fn creer_tache(titre: String, priorite: u8) -> Tache {
    (titre, priorite, false)
}

fn afficher_tache(tache: Tache) {
    let (titre, priorite, complete) = tache;
    let statut = if complete { "terminee" } else { "a faire" };

    println!("Titre    : {}", titre);
    println!("Priorite : {}", priorite);
    println!("Statut   : {}", statut);
}

fn marquer_complete(tache: Tache) -> Tache {
    let (titre, priorite, _) = tache;
    (titre, priorite, true)
}

fn extraire_titre(tache: Tache) -> String {
    let (titre, _, _) = tache;
    titre
}

// Correction Programme 1
// Le probleme vient du move de `v` vers `v2`.
// `Vec<T>` n'est pas `Copy`, donc `v` devient invalide apres l'affectation.
fn programme1_corrige() {
    let v = vec![1, 2, 3];
    let v2 = v;

    println!("Longueur via v2 : {}", v2.len());
}

// Correction Programme 2 sans clone()
// On prend une reference immuable pour calculer la somme et garder la possession du vecteur dans main.
fn somme(v: &[i32]) -> i32 {
    v.iter().sum()
}

fn programme2_corrige() {
    let nombres = vec![1, 2, 3, 4, 5];
    let s = somme(&nombres);
    println!("Somme : {}, Vecteur : {:?}", s, nombres);
}

fn demo_gestion_taches() {
    let tache1 = creer_tache(String::from("Reviser ownership"), 1);
    let tache1 = marquer_complete(tache1);
    afficher_tache(tache1);

    let tache2 = creer_tache(String::from("Lire le borrow checker"), 2);
    let titre = extraire_titre(tache2);
    println!("Titre extrait : {}", titre);
}

fn main() {
    println!("=== TP2 - Partie A : corrections ===");
    programme1_corrige();
    programme2_corrige();

    println!();
    println!("=== TP2 - Partie B : gestionnaire de taches ===");
    demo_gestion_taches();

    println!();
    println!("Rappel : dans l'exercice principal, les fonctions du gestionnaire de taches");
    println!("travaillent volontairement avec transfert d'ownership, sans references dans les signatures.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_tache() {
        let tache = creer_tache(String::from("Tester"), 3);
        assert_eq!(tache.0, "Tester");
        assert_eq!(tache.1, 3);
        assert!(!tache.2);
    }

    #[test]
    fn marquage_termine() {
        let tache = creer_tache(String::from("Finir TP2"), 1);
        let tache = marquer_complete(tache);
        assert!(tache.2);
    }

    #[test]
    fn extraction_titre() {
        let tache = creer_tache(String::from("Ownership"), 2);
        let titre = extraire_titre(tache);
        assert_eq!(titre, "Ownership");
    }

    #[test]
    fn somme_reference() {
        let nombres = vec![1, 2, 3, 4, 5];
        assert_eq!(somme(&nombres), 15);
    }
}
