use std::env; // Pour récupérer les arguments de la ligne de commande

fn main() {
    // 1. Récupération des arguments
    // args[0] est le nom du programme, args[1..3] sont nos données
    let args: Vec<String> = env::args().collect();

    // Vérification du nombre d'arguments
    if args.len() != 4 {
        println!("Usage: cargo run -- <nombre1> <opérateur> <nombre2>");
        println!("Exemple: cargo run -- 10 + 5");
        return;
    }

    // 2. Parsing (conversion du texte en nombres)
    let num1: f64 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Erreur : '{}' n'est pas un nombre valide.", args[1]);
            return;
        }
    };

    let operateur = &args[2];

    let num2: f64 = match args[3].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Erreur : '{}' n'est pas un nombre valide.", args[3]);
            return;
        }
    };

    // 3. Calcul avec Pattern Matching
    let resultat = match operateur.as_str() {
        "+" => Ok(num1 + num2),
        "-" => Ok(num1 - num2),
        "*" => Ok(num1 * num2),
        "/" => {
            if num2 == 0.0 {
                Err("Division par zéro impossible !")
            } else {
                Ok(num1 / num2)
            }
        }
        _ => Err("Opérateur non supporté (utilisez +, -, * ou /)"),
    };

    // 4. Affichage du résultat ou de l'erreur
    match resultat {
        Ok(valeur) => println!("Résultat : {} {} {} = {}", num1, operateur, num2, valeur),
        Err(message) => println!("Erreur : {}", message),
    }
}