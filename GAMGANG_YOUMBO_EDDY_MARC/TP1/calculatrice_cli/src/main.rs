use std::env;
use std::io::{self, Write};

fn calculer(a: f64, op: &str, b: f64) -> Result<f64, String> {
    match op {
        "+" => Ok(a + b),
        "-" => Ok(a - b),
        "*" => Ok(a * b),
        "/" => {
            if b == 0.0 {
                Err(String::from("Division par zero"))
            } else {
                Ok(a / b)
            }
        }
        _ => Err(format!("Operateur inconnu : {}", op)),
    }
}

fn parser_expression(expression: &str) -> Result<(f64, String, f64), String> {
    let morceaux: Vec<&str> = expression.split_whitespace().collect();

    if morceaux.len() != 3 {
        return Err(String::from(
            "Format invalide. Exemple attendu : 12 + 5",
        ));
    }

    let a = morceaux[0]
        .parse::<f64>()
        .map_err(|_| format!("Nombre invalide : {}", morceaux[0]))?;
    let op = morceaux[1].to_string();
    let b = morceaux[2]
        .parse::<f64>()
        .map_err(|_| format!("Nombre invalide : {}", morceaux[2]))?;

    Ok((a, op, b))
}

fn afficher_resultat(a: f64, op: &str, b: f64) -> Result<(), String> {
    let resultat = calculer(a, op, b)?;
    println!("{a} {op} {b} = {resultat}");
    Ok(())
}

fn mode_interactif() {
    println!("Mode interactif active.");
    println!("Saisis une expression du type : 12 + 5");
    println!("Tape 'quitter' pour sortir.");

    loop {
        print!("> ");
        io::stdout().flush().expect("Impossible de vider stdout");

        let mut entree = String::new();
        match io::stdin().read_line(&mut entree) {
            Ok(_) => {
                let entree = entree.trim();

                if entree.eq_ignore_ascii_case("quitter") {
                    println!("Fermeture de la calculatrice.");
                    break;
                }

                match parser_expression(entree) {
                    Ok((a, op, b)) => {
                        if let Err(e) = afficher_resultat(a, &op, b) {
                            eprintln!("Erreur : {e}");
                        }
                    }
                    Err(e) => eprintln!("Erreur : {e}"),
                }
            }
            Err(e) => {
                eprintln!("Erreur de lecture : {e}");
            }
        }
    }
}

fn afficher_aide(programme: &str) {
    eprintln!("Usage :");
    eprintln!("  {programme} <nombre1> <operateur> <nombre2>");
    eprintln!("Exemples :");
    eprintln!("  {programme} 10 + 5");
    eprintln!("  {programme} 12 / 4");
    eprintln!("  {programme}      # lance le mode interactif");
    eprintln!("Operateurs supportes : + - * /");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => mode_interactif(),
        4 => {
            let a = match args[1].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("'{}' n'est pas un nombre valide", args[1]);
                    std::process::exit(1);
                }
            };

            let op = &args[2];

            let b = match args[3].parse::<f64>() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("'{}' n'est pas un nombre valide", args[3]);
                    std::process::exit(1);
                }
            };

            if let Err(e) = afficher_resultat(a, op, b) {
                eprintln!("Erreur : {e}");
                std::process::exit(1);
            }
        }
        _ => {
            afficher_aide(&args[0]);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addition() {
        assert_eq!(calculer(2.0, "+", 3.0).unwrap(), 5.0);
    }

    #[test]
    fn division_par_zero() {
        assert!(calculer(8.0, "/", 0.0).is_err());
    }

    #[test]
    fn parsing_expression_valide() {
        let (a, op, b) = parser_expression("12 * 3").unwrap();
        assert_eq!(a, 12.0);
        assert_eq!(op, "*");
        assert_eq!(b, 3.0);
    }
}
