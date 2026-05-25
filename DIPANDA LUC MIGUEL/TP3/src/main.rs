struct Statistiques {
    nb_mots: usize,
    nb_caracteres: usize,
    mot_le_plus_long: String,
}

fn main() {
    let texte_utilisateur = String::from("Rust est un langage de programmation moderne et sûr.");

    println!("--- ANALYSE DE TEXTE ---");
    println!("Texte : \"{}\"", texte_utilisateur);

  
    let nombre = compter_mots(&texte_utilisateur);
    println!("Nombre de mots : {}", nombre);

   
    let plus_long = trouver_mot_le_plus_long(&texte_utilisateur);
    println!("Le mot le plus long est : \"{}\"", plus_long);

    
    let test_pali = "radar";
    println!("Est-ce que '{}' est un palindrome ? {}", test_pali, est_palindrome(test_pali));

   
    let stats = generer_stats(&texte_utilisateur);
    println!("\n--- RÉSULTAT DES STATS ---");
    println!("Total caractères : {}", stats.nb_caracteres);
    println!("Total mots : {}", stats.nb_mots);
    println!("Copie du mot le plus long : {}", stats.mot_le_plus_long);
}


fn compter_mots(s: &str) -> usize {
    s.split_whitespace().count()
}

fn trouver_mot_le_plus_long<'a>(s: &'a str) -> &'a str {
    let mut mot_max = "";
    for mot in s.split_whitespace() {
        if mot.len() > mot_max.len() {
            mot_max = mot;
        }
    }
    mot_max
}

fn est_palindrome(s: &str) -> bool {
    let s_clean = s.to_lowercase();
    s_clean.chars().eq(s_clean.chars().rev())
}

fn generer_stats(s: &str) -> Statistiques {
    Statistiques {
        nb_mots: compter_mots(s),
        nb_caracteres: s.len(),
        mot_le_plus_long: trouver_mot_le_plus_long(s).to_string(), 
    }
}