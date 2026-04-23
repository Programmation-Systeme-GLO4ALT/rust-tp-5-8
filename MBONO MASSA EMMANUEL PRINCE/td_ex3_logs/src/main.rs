// ============================================================
// TD GL4 — Exercice 3 : Pipeline de traitement de journaux
// Séance 7 : Closures & Itérateurs
// Programmation Système avec Rust — ENSPD GL4 2025-2026
// ============================================================
//
// Parties :
//   3.1 — Closures et capture d'environnement
//   3.2 — Adaptateurs d'itérateurs
//   3.3 — Itérateur custom FenetreGlissante
// ============================================================

use std::collections::HashMap;

// ============================================================
// Types de base
// ============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum NiveauLog { DEBUG, INFO, WARN, ERROR }

impl NiveauLog {
    /// Valeur numérique pour comparer les niveaux (DEBUG < INFO < WARN < ERROR)
    fn valeur(&self) -> u8 {
        match self {
            NiveauLog::DEBUG => 0,
            NiveauLog::INFO  => 1,
            NiveauLog::WARN  => 2,
            NiveauLog::ERROR => 3,
        }
    }
}

impl std::fmt::Display for NiveauLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NiveauLog::DEBUG => write!(f, "DEBUG"),
            NiveauLog::INFO  => write!(f, "INFO"),
            NiveauLog::WARN  => write!(f, "WARN"),
            NiveauLog::ERROR => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EntreeLog {
    pub timestamp: u64,     // secondes depuis epoch
    pub niveau:    NiveauLog,
    pub module:    String,
    pub message:   String,
}

// ============================================================
// PARTIE 3.1 — Closures et capture d'environnement
// ============================================================

/// Q1 — Retourne une closure impl Fn(&EntreeLog) -> bool
/// filtrant par niveau minimum.
///
/// La closure capture `niveau_min` par valeur (move) :
/// → Nécessaire car la closure peut vivre plus longtemps que la frame d'appel.
/// → `move` transfère la propriété de niveau_min dans la closure.
pub fn creer_filtre_niveau(niveau_min: NiveauLog) -> impl Fn(&EntreeLog) -> bool {
    move |entree| entree.niveau.valeur() >= niveau_min.valeur()
}

/// Q2 — Différence entre Fn, FnMut et FnOnce :
///
///   Fn      : peut être appelée plusieurs fois, ne modifie pas son environnement capturé.
///             Exemple : |x: &i32| *x > seuil  (seuil est emprunté ou copié)
///
///   FnMut   : peut être appelée plusieurs fois, PEUT modifier son environnement.
///             Exemple : compteur (ci-dessous) — modifie un entier capturé.
///
///   FnOnce  : ne peut être appelée qu'UNE SEULE FOIS, consomme son environnement.
///             Exemple : |_| drop(valeur_ownable)  — valeur est moved dans la closure.
///
/// Un FnOnce n'est pas Fn ni FnMut.
/// Un FnMut est FnOnce mais pas forcément Fn.
/// Un Fn est FnMut et FnOnce (hiérarchie de sous-traits).

/// Q3 — Closure compteur de type FnMut() -> usize
/// Ne peut pas être Fn car elle MODIFIE l'état interne (compteur).
pub fn creer_compteur() -> impl FnMut() -> usize {
    let mut compte = 0usize;
    move || {
        compte += 1; // modification de l'environnement → FnMut obligatoire
        compte
    }
}

// ============================================================
// PARTIE 3.2 — Adaptateurs d'itérateurs
// ============================================================

/// Q4 — Retourne le nombre d'erreurs par module.
/// N'utilise pas de boucle for explicite : uniquement filter + fold.
pub fn analyser_logs(logs: &[EntreeLog]) -> HashMap<String, usize> {
    logs.iter()
        .filter(|e| e.niveau == NiveauLog::ERROR)
        .fold(HashMap::new(), |mut acc, entree| {
            *acc.entry(entree.module.clone()).or_insert(0) += 1;
            acc
        })
}

/// Q5 — 5 messages d'erreur les plus récents, formatés "[timestamp] module: message"
pub fn cinq_erreurs_recentes(logs: &[EntreeLog]) -> Vec<String> {
    let mut erreurs: Vec<&EntreeLog> = logs.iter()
        .filter(|e| e.niveau == NiveauLog::ERROR)
        .collect();

    // Trier par timestamp décroissant (les plus récents en premier)
    erreurs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    erreurs.iter()
        .take(5)
        .map(|e| format!("[{}] {}: {}", e.timestamp, e.module, e.message))
        .collect()
}

// ============================================================
// PARTIE 3.3 — Itérateur custom FenetreGlissante
// ============================================================

/// Q6 — Itérateur qui produit des fenêtres glissantes de taille fixe.
/// Chaque appel à next() retourne une tranche de `taille` éléments consécutifs.
///
/// Exemple : [1,2,3,4,5] avec taille=3 → [1,2,3], [2,3,4], [3,4,5]
pub struct FenetreGlissante<'a> {
    donnees:  &'a [EntreeLog],
    taille:   usize,
    position: usize,
}

impl<'a> FenetreGlissante<'a> {
    pub fn nouvelle(donnees: &'a [EntreeLog], taille: usize) -> Self {
        FenetreGlissante { donnees, taille, position: 0 }
    }
}

impl<'a> Iterator for FenetreGlissante<'a> {
    type Item = &'a [EntreeLog];

    fn next(&mut self) -> Option<Self::Item> {
        // Il reste assez d'éléments pour former une fenêtre complète ?
        if self.position + self.taille <= self.donnees.len() {
            let fenetre = &self.donnees[self.position..self.position + self.taille];
            self.position += 1; // avancer d'un élément à la fois
            Some(fenetre)
        } else {
            None // plus assez d'éléments : fin de l'itération
        }
    }
}

// ============================================================
// Données de démonstration
// ============================================================
fn jeu_de_donnees() -> Vec<EntreeLog> {
    vec![
        EntreeLog { timestamp: 1700000001, niveau: NiveauLog::INFO,  module: "auth".into(),    message: "Connexion réussie".into() },
        EntreeLog { timestamp: 1700000002, niveau: NiveauLog::DEBUG, module: "db".into(),      message: "Requête SQL exécutée".into() },
        EntreeLog { timestamp: 1700000003, niveau: NiveauLog::ERROR, module: "auth".into(),    message: "Mot de passe incorrect".into() },
        EntreeLog { timestamp: 1700000004, niveau: NiveauLog::WARN,  module: "cache".into(),   message: "Cache presque plein".into() },
        EntreeLog { timestamp: 1700000005, niveau: NiveauLog::ERROR, module: "db".into(),      message: "Connexion perdue".into() },
        EntreeLog { timestamp: 1700000006, niveau: NiveauLog::ERROR, module: "auth".into(),    message: "Token expiré".into() },
        EntreeLog { timestamp: 1700000007, niveau: NiveauLog::INFO,  module: "api".into(),     message: "Requête GET /users".into() },
        EntreeLog { timestamp: 1700000008, niveau: NiveauLog::ERROR, module: "db".into(),      message: "Timeout requête".into() },
        EntreeLog { timestamp: 1700000009, niveau: NiveauLog::ERROR, module: "cache".into(),   message: "Eviction forcée".into() },
        EntreeLog { timestamp: 1700000010, niveau: NiveauLog::ERROR, module: "auth".into(),    message: "Brute force détecté".into() },
    ]
}

// ============================================================
// Programme principal
// ============================================================
fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  TD GL4 — Ex3 : Pipeline de Logs (Closures/Iter.)   ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    let logs = jeu_de_donnees();

    // --- 3.1 : Filtres par niveau ---
    println!("[ 3.1 — Filtres par niveau ]");
    let filtre_warn = creer_filtre_niveau(NiveauLog::WARN);
    let nb_warn_plus = logs.iter().filter(|e| filtre_warn(e)).count();
    println!("  Entrées WARN et plus : {}", nb_warn_plus);

    let filtre_error = creer_filtre_niveau(NiveauLog::ERROR);
    let nb_errors = logs.iter().filter(|e| filtre_error(e)).count();
    println!("  Entrées ERROR        : {}", nb_errors);

    // --- 3.1 : Compteur FnMut ---
    println!("\n[ 3.1 — Compteur FnMut ]");
    let mut compter = creer_compteur();
    println!("  Appel 1 : {}", compter());
    println!("  Appel 2 : {}", compter());
    println!("  Appel 3 : {}", compter());

    // --- 3.2 : Erreurs par module ---
    println!("\n[ 3.2 — Erreurs par module (analyser_logs) ]");
    let stats = analyser_logs(&logs);
    let mut stats_triees: Vec<_> = stats.iter().collect();
    stats_triees.sort_by(|a, b| b.1.cmp(a.1));
    for (module, count) in &stats_triees {
        println!("  {:<12} : {} erreur(s)", module, count);
    }

    // --- 3.2 : 5 erreurs récentes ---
    println!("\n[ 3.2 — 5 erreurs les plus récentes ]");
    for msg in cinq_erreurs_recentes(&logs) {
        println!("  {}", msg);
    }

    // --- 3.3 : Fenêtre glissante ---
    println!("\n[ 3.3 — FenetreGlissante (taille=3) ]");
    let fenetre = FenetreGlissante::nouvelle(&logs, 3);
    for (i, tranche) in fenetre.enumerate() {
        let niveaux: Vec<_> = tranche.iter().map(|e| format!("{}", e.niveau)).collect();
        println!("  Fenêtre {} : [{}]", i, niveaux.join(", "));
    }
}

// ============================================================
// Tests unitaires
// ============================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn logs_test() -> Vec<EntreeLog> {
        vec![
            EntreeLog { timestamp: 100, niveau: NiveauLog::DEBUG, module: "a".into(), message: "d1".into() },
            EntreeLog { timestamp: 200, niveau: NiveauLog::INFO,  module: "b".into(), message: "i1".into() },
            EntreeLog { timestamp: 300, niveau: NiveauLog::WARN,  module: "a".into(), message: "w1".into() },
            EntreeLog { timestamp: 400, niveau: NiveauLog::ERROR, module: "a".into(), message: "e1".into() },
            EntreeLog { timestamp: 500, niveau: NiveauLog::ERROR, module: "b".into(), message: "e2".into() },
        ]
    }

    #[test]
    fn test_filtre_niveau_error_only() {
        let logs = logs_test();
        let filtre = creer_filtre_niveau(NiveauLog::ERROR);
        let count = logs.iter().filter(|e| filtre(e)).count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_filtre_niveau_tous() {
        let logs = logs_test();
        let filtre = creer_filtre_niveau(NiveauLog::DEBUG);
        let count = logs.iter().filter(|e| filtre(e)).count();
        assert_eq!(count, 5);
    }

    #[test]
    fn test_compteur_incremente() {
        let mut c = creer_compteur();
        assert_eq!(c(), 1);
        assert_eq!(c(), 2);
        assert_eq!(c(), 3);
    }

    #[test]
    fn test_analyser_logs_count_par_module() {
        let logs = logs_test();
        let stats = analyser_logs(&logs);
        assert_eq!(stats["a"], 1);
        assert_eq!(stats["b"], 1);
        assert!(!stats.contains_key("nope"));
    }

    #[test]
    fn test_cinq_erreurs_recentes_ordre() {
        let logs = logs_test();
        let erreurs = cinq_erreurs_recentes(&logs);
        // Les plus récentes en premier : timestamp 500 avant 400
        assert!(erreurs[0].contains("500"));
        assert!(erreurs[1].contains("400"));
    }

    #[test]
    fn test_cinq_erreurs_recentes_max_5() {
        // 7 erreurs dans les données → ne retourne que 5
        let mut logs = logs_test();
        for i in 0..5 {
            logs.push(EntreeLog {
                timestamp: 600 + i,
                niveau: NiveauLog::ERROR,
                module: "x".into(),
                message: format!("extra {}", i),
            });
        }
        let erreurs = cinq_erreurs_recentes(&logs);
        assert_eq!(erreurs.len(), 5);
    }

    #[test]
    fn test_fenetre_glissante_nombre_fenetres() {
        let logs = logs_test(); // 5 éléments
        let fenetres: Vec<_> = FenetreGlissante::nouvelle(&logs, 3).collect();
        // 5 - 3 + 1 = 3 fenêtres
        assert_eq!(fenetres.len(), 3);
    }

    #[test]
    fn test_fenetre_glissante_contenu() {
        let logs = logs_test();
        let mut iter = FenetreGlissante::nouvelle(&logs, 2);
        let f1 = iter.next().unwrap();
        assert_eq!(f1[0].timestamp, 100);
        assert_eq!(f1[1].timestamp, 200);
        let f2 = iter.next().unwrap();
        assert_eq!(f2[0].timestamp, 200);
        assert_eq!(f2[1].timestamp, 300);
    }

    #[test]
    fn test_fenetre_glissante_trop_grande() {
        let logs = logs_test(); // 5 éléments
        let fenetres: Vec<_> = FenetreGlissante::nouvelle(&logs, 10).collect();
        assert_eq!(fenetres.len(), 0); // taille > longueur → aucune fenêtre
    }

    #[test]
    fn test_fenetre_egalite_windows_std() {
        // Vérifier que notre implémentation donne le même résultat que la méthode standard
        let logs = logs_test();
        let notre: Vec<_> = FenetreGlissante::nouvelle(&logs, 2).collect();
        let std_windows: Vec<_> = logs.windows(2).collect();
        assert_eq!(notre.len(), std_windows.len());
        for (a, b) in notre.iter().zip(std_windows.iter()) {
            assert_eq!(a.len(), b.len());
        }
    }
}

