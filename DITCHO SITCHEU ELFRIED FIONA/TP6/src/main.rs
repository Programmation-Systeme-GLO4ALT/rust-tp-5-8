// ============================================================
// TP6 — Abstraction I/O générique avec le trait Readable
// ============================================================
//
// Objectifs pédagogiques :
//   1. Définir un trait Readable avec méthodes obligatoires et par défaut
//   2. Implémenter Readable sur 3 types concrets :
//        • FichierLecteur   — lit depuis le disque (File + BufReader)
//        • BufferLecteur    — lit depuis un Vec<u8> en mémoire (Cursor)
//        • StdinLecteur     — lit depuis l'entrée standard
//   3. Utiliser une fonction générique STATIQUE   process<R: Readable>
//   4. Utiliser une collection DYNAMIQUE          Vec<Box<dyn Readable>>
//   5. Comparer dispatch statique vs dynamique

use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Cursor, Read};

// ─────────────────────────────────────────────────────────
// 1. LE TRAIT Readable
// ─────────────────────────────────────────────────────────
//
// Un trait est un contrat : tout type qui implémente Readable
// garantit de fournir ces comportements.
//
// Règles pour être "object-safe" (utilisable comme dyn Readable) :
//   • Aucune méthode ne retourne Self
//   • Aucune méthode générique (pas de fn lire<T>(...))
//   • Les méthodes associées sans &self ne sont pas permises
//     sauf avec where Self: Sized

pub trait Readable {
    // ── Méthodes OBLIGATOIRES (pas d'implémentation par défaut) ──

    /// Retourne le nom ou la description de la source
    fn nom(&self) -> &str;

    /// Lit tout le contenu disponible sous forme de chaîne UTF-8
    fn lire_tout(&mut self) -> io::Result<String>;

    // ── Méthodes AVEC implémentation par défaut ──
    //
    // Ces méthodes sont fournies gratuitement à tous les types
    // qui implémentent le trait. Elles peuvent être surchargées.

    /// Retourne le nombre d'octets disponibles (estimation)
    /// Par défaut : lit tout et compte. Les types peuvent optimiser.
    fn taille_estimee(&mut self) -> io::Result<usize> {
        Ok(self.lire_tout()?.len())
    }

    /// Lit le contenu et retourne la liste de toutes les lignes
    fn lire_lignes(&mut self) -> io::Result<Vec<String>> {
        let contenu = self.lire_tout()?;
        Ok(contenu.lines().map(|l| l.to_string()).collect())
    }

    /// Retourne le nombre de lignes
    fn compter_lignes(&mut self) -> io::Result<usize> {
        Ok(self.lire_lignes()?.len())
    }

    /// Retourne une description générique de la source
    fn description(&self) -> String {
        format!("Source[{}]", self.nom())
    }
}

// ─────────────────────────────────────────────────────────
// 2. IMPLÉMENTATION 1 : FichierLecteur (lecture sur disque)
// ─────────────────────────────────────────────────────────
//
// BufReader enveloppe un File et ajoute un tampon de 8 Ko.
// Cela regroupe les appels système (syscall read()) coûteux
// en lectures de blocs, ce qui est beaucoup plus rapide
// pour les fichiers lus ligne par ligne.

pub struct FichierLecteur {
    chemin: String,
    lecteur: BufReader<File>,
}

impl FichierLecteur {
    /// Constructeur — ouvre le fichier ou retourne une erreur io
    pub fn nouveau(chemin: &str) -> io::Result<Self> {
        let fichier = File::open(chemin)?;
        Ok(FichierLecteur {
            chemin: chemin.to_string(),
            lecteur: BufReader::new(fichier),
        })
    }
}

impl Readable for FichierLecteur {
    fn nom(&self) -> &str {
        &self.chemin
    }

    fn lire_tout(&mut self) -> io::Result<String> {
        let mut contenu = String::new();
        // read_to_string remplit le buffer depuis le BufReader
        self.lecteur.read_to_string(&mut contenu)?;
        Ok(contenu)
    }

    // SURCHARGE de taille_estimee : on peut utiliser les métadonnées
    // du fichier pour avoir la taille sans tout lire en mémoire
    // Note : on ne peut pas accéder à self.lecteur.get_ref() facilement
    // après un read_to_string, donc on garde l'implémentation par défaut
    // pour cet exemple. Une vraie implémentation lirait les metadata avant.
}

// ─────────────────────────────────────────────────────────
// 3. IMPLÉMENTATION 2 : BufferLecteur (mémoire in-memory)
// ─────────────────────────────────────────────────────────
//
// Cursor<Vec<u8>> est un type de la std qui enveloppe un Vec<u8>
// et implémente Read/Write/Seek comme si c'était un fichier.
// Parfait pour les tests et le traitement de données en mémoire.

pub struct BufferLecteur {
    nom: String,
    curseur: Cursor<Vec<u8>>,
}

impl BufferLecteur {
    /// Crée un buffer depuis un slice de bytes
    pub fn depuis_bytes(nom: &str, donnees: Vec<u8>) -> Self {
        BufferLecteur {
            nom: nom.to_string(),
            curseur: Cursor::new(donnees),
        }
    }

    /// Crée un buffer depuis une chaîne UTF-8
    pub fn depuis_str(nom: &str, contenu: &str) -> Self {
        Self::depuis_bytes(nom, contenu.as_bytes().to_vec())
    }
}

impl Readable for BufferLecteur {
    fn nom(&self) -> &str {
        &self.nom
    }

    fn lire_tout(&mut self) -> io::Result<String> {
        let mut contenu = String::new();
        self.curseur.read_to_string(&mut contenu)?;
        Ok(contenu)
    }

    // SURCHARGE de taille_estimee : on connaît la taille exacte
    // sans avoir à tout lire grâce à get_ref()
    fn taille_estimee(&mut self) -> io::Result<usize> {
        Ok(self.curseur.get_ref().len())
    }
}

// ─────────────────────────────────────────────────────────
// 4. IMPLÉMENTATION 3 : StdinLecteur (entrée standard)
// ─────────────────────────────────────────────────────────
//
// stdin() retourne un handle vers l'entrée standard.
// BufReader l'enveloppe pour la lecture ligne par ligne.
//
// Note : en mode non-interactif (pipe ou redirection), stdin()
// se comportera comme un flux de données classique.

pub struct StdinLecteur {
    lecteur: BufReader<io::Stdin>,
}

impl StdinLecteur {
    pub fn nouveau() -> Self {
        StdinLecteur {
            lecteur: BufReader::new(io::stdin()),
        }
    }
}

impl Readable for StdinLecteur {
    fn nom(&self) -> &str {
        "stdin"
    }

    fn lire_tout(&mut self) -> io::Result<String> {
        let mut contenu = String::new();
        self.lecteur.read_to_string(&mut contenu)?;
        Ok(contenu)
    }

    // SURCHARGE de lire_lignes pour une lecture ligne par ligne efficace
    // (utile si stdin est interactif et on ne veut pas attendre EOF)
    fn lire_lignes(&mut self) -> io::Result<Vec<String>> {
        let mut lignes = Vec::new();
        let mut ligne = String::new();
        loop {
            ligne.clear();
            let nb = self.lecteur.read_line(&mut ligne)?;
            if nb == 0 {
                break; // EOF
            }
            lignes.push(ligne.trim_end_matches('\n').to_string());
        }
        Ok(lignes)
    }
}

// ─────────────────────────────────────────────────────────
// 5. FONCTION GÉNÉRIQUE STATIQUE : dispatch statique
// ─────────────────────────────────────────────────────────
//
// Syntaxe : fn f<R: Readable>(source: &mut R)
// Équivalent à : fn f<R>(source: &mut R) where R: Readable
//
// Le compilateur génère UNE version de la fonction par type concret
// utilisé à l'appel. C'est la "monomorphisation" :
//   process::<FichierLecteur>(...) → code spécialisé pour FichierLecteur
//   process::<BufferLecteur>(...) → code spécialisé pour BufferLecteur
//
// Avantages : zéro overhead, inlining possible
// Inconvénient : code binaire plus grand, pas de collection hétérogène

pub fn analyser_source<R: Readable>(source: &mut R) -> io::Result<Statistiques> {
    println!("  [statique] Analyse de «{}»", source.description());

    let contenu = source.lire_tout()?;
    let nb_lignes = contenu.lines().count();
    let nb_mots = contenu.split_whitespace().count();
    let nb_octets = contenu.len();

    Ok(Statistiques {
        source: source.nom().to_string(),
        nb_lignes,
        nb_mots,
        nb_octets,
    })
}

#[derive(Debug)]
pub struct Statistiques {
    pub source: String,
    pub nb_lignes: usize,
    pub nb_mots: usize,
    pub nb_octets: usize,
}

impl std::fmt::Display for Statistiques {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  Source: {:30} | Lignes: {:4} | Mots: {:5} | Octets: {:5}",
            self.source, self.nb_lignes, self.nb_mots, self.nb_octets
        )
    }
}

// ─────────────────────────────────────────────────────────
// 6. FONCTION avec COLLECTION DYNAMIQUE : dispatch dynamique
// ─────────────────────────────────────────────────────────
//
// Box<dyn Readable> = pointeur sur le tas vers un objet implémentant Readable
// La vtable (table de pointeurs de fonctions) est utilisée à l'exécution
// pour appeler la bonne implémentation.
//
// Vec<Box<dyn Readable>> = collection hétérogène de sources différentes
// → IMPOSSIBLE avec le dispatch statique (chaque T doit être le même type)
//
// Avantage : flexibilité, collections hétérogènes, injection de dépendance
// Inconvénient : allocation heap, indirection vtable (overhead minimal ~1ns)

pub fn analyser_toutes_les_sources(
    sources: &mut Vec<Box<dyn Readable>>,
) -> io::Result<Vec<Statistiques>> {
    println!("  [dynamique] {} source(s) dans la collection", sources.len());
    let mut resultats = Vec::new();

    for source in sources.iter_mut() {
        // Ici, l'appel est dynamique : la vtable détermine quelle
        // implémentation de lire_tout() appeler au moment de l'exécution
        println!("    → Traitement de «{}»", source.nom());
        let contenu = source.lire_tout()?;
        let nb_lignes = contenu.lines().count();
        let nb_mots = contenu.split_whitespace().count();
        let nb_octets = contenu.len();
        resultats.push(Statistiques {
            source: source.nom().to_string(),
            nb_lignes,
            nb_mots,
            nb_octets,
        });
    }

    Ok(resultats)
}

// ─────────────────────────────────────────────────────────
// 7. POINT D'ENTRÉE
// ─────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════");
    println!("    TP6 — Abstraction I/O générique         ");
    println!("═══════════════════════════════════════════\n");

    // ── Préparer des fichiers de test ──────────────────────
    fs::write(
        "tp6_poeme.txt",
        "Les sanglots longs\nDes violons\nDe l'automne\n\nBlessent mon coeur\n",
    )?;
    fs::write(
        "tp6_data.txt",
        "alpha beta gamma\ndelta epsilon\nzeta\n",
    )?;

    // ══════════════════════════════════════════════════════
    // PARTIE A — DISPATCH STATIQUE
    // ══════════════════════════════════════════════════════
    println!("━━━ PARTIE A : Dispatch statique (monomorphisation) ━━━\n");

    // Le compilateur génère du code spécialisé pour CHAQUE type
    // process::<FichierLecteur> et process::<BufferLecteur> sont
    // deux fonctions distinctes dans le binaire compilé.

    println!("  ▶ Test avec FichierLecteur");
    let mut lecteur_fichier = FichierLecteur::nouveau("tp6_poeme.txt")?;
    let stats1 = analyser_source(&mut lecteur_fichier)?; // type inféré : FichierLecteur
    println!("{}", stats1);

    println!("\n  ▶ Test avec BufferLecteur");
    let contenu_mem = "premiere ligne du buffer\ndeuxieme ligne\ntroisieme ligne avec plus de mots ici\n";
    let mut lecteur_buffer = BufferLecteur::depuis_str("buffer_memoire", contenu_mem);
    let stats2 = analyser_source(&mut lecteur_buffer); // type inféré : BufferLecteur
    println!("{}", stats2?);

    // Démonstration de la méthode par défaut (taille_estimee)
    println!("\n  ▶ Démonstration des méthodes par défaut du trait");
    let mut b2 = BufferLecteur::depuis_str("test_defaut", "un\ndeux\ntrois\nquatre\ncinq\n");
    println!("  taille_estimee()  = {} octets", b2.taille_estimee()?);
    let mut b3 = BufferLecteur::depuis_str("test_lignes", "ligne A\nligne B\nligne C\n");
    println!("  compter_lignes()  = {}", b3.compter_lignes()?);
    println!("  description()     = {}", b3.description());

    // ══════════════════════════════════════════════════════
    // PARTIE B — DISPATCH DYNAMIQUE (collection hétérogène)
    // ══════════════════════════════════════════════════════
    println!("\n━━━ PARTIE B : Dispatch dynamique (Box<dyn Readable>) ━━━\n");

    // Construction d'une collection HÉTÉROGÈNE
    // Box::new(...) alloue l'objet sur le tas et stocke un fat pointer
    // (pointeur données + pointeur vtable) dans le Vec
    let mut sources: Vec<Box<dyn Readable>> = vec![
        Box::new(FichierLecteur::nouveau("tp6_poeme.txt")?),
        Box::new(FichierLecteur::nouveau("tp6_data.txt")?),
        Box::new(BufferLecteur::depuis_str(
            "config_inline",
            "cle1=val1\ncle2=val2\ncle3=val3\n",
        )),
        Box::new(BufferLecteur::depuis_bytes(
            "donnees_binaires_utf8",
            b"Hello\nWorld\nRust\n".to_vec(),
        )),
    ];

    let resultats = analyser_toutes_les_sources(&mut sources)?;
    println!("\n  Résultats :");
    for stat in &resultats {
        println!("{}", stat);
    }

    // ── Statistiques agrégées ──────────────────────────────
    let total_lignes: usize = resultats.iter().map(|s| s.nb_lignes).sum();
    let total_mots: usize   = resultats.iter().map(|s| s.nb_mots).sum();
    let total_octets: usize = resultats.iter().map(|s| s.nb_octets).sum();
    println!("\n  Totaux : {} lignes, {} mots, {} octets", total_lignes, total_mots, total_octets);

    // ══════════════════════════════════════════════════════
    // PARTIE C — Comparaison statique vs dynamique
    // ══════════════════════════════════════════════════════
    println!("\n━━━ PARTIE C : Comparaison des deux approches ━━━\n");

    // Statique : fonction générique, type connu à la compilation
    fn compter_mots_statique<R: Readable>(src: &mut R) -> io::Result<usize> {
        Ok(src.lire_tout()?.split_whitespace().count())
    }

    // Dynamique : trait object, type connu seulement à l'exécution
    fn compter_mots_dynamique(src: &mut dyn Readable) -> io::Result<usize> {
        Ok(src.lire_tout()?.split_whitespace().count())
    }

    let mut buf_a = BufferLecteur::depuis_str("a", "un deux trois");
    let mut buf_b = BufferLecteur::depuis_str("b", "quatre cinq six sept");

    // Appels statiques (monomorphisés)
    println!("  [statique] buf_a : {} mots", compter_mots_statique(&mut buf_a)?);
    println!("  [statique] buf_b : {} mots", compter_mots_statique(&mut buf_b)?);

    // Appels dynamiques (vtable)
    let mut buf_c = BufferLecteur::depuis_str("c", "alpha beta gamma delta epsilon");
    let source_dyn: &mut dyn Readable = &mut buf_c;
    println!("  [dynamique] buf_c : {} mots", compter_mots_dynamique(source_dyn)?);

    // La puissance du dynamique : passer n'importe quelle implémentation
    let mut fichier_dyn = FichierLecteur::nouveau("tp6_data.txt")?;
    let source_fichier: &mut dyn Readable = &mut fichier_dyn;
    println!("  [dynamique] fichier : {} mots", compter_mots_dynamique(source_fichier)?);

    println!("\n✅  TP6 terminé — dispatch statique ET dynamique maîtrisés !");
    Ok(())
}

// ─────────────────────────────────────────────────────────
// 8. TESTS UNITAIRES
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Utilitaire de test
    fn buffer(contenu: &str) -> BufferLecteur {
        BufferLecteur::depuis_str("test", contenu)
    }

    #[test]
    fn test_buffer_lire_tout() {
        let contenu = "ligne1\nligne2\nligne3";
        let mut b = buffer(contenu);
        assert_eq!(b.lire_tout().unwrap(), contenu);
    }

    #[test]
    fn test_buffer_nom() {
        let b = BufferLecteur::depuis_str("mon_buffer", "data");
        assert_eq!(b.nom(), "mon_buffer");
    }

    #[test]
    fn test_methode_defaut_lire_lignes() {
        let mut b = buffer("a\nb\nc\n");
        let lignes = b.lire_lignes().unwrap();
        assert_eq!(lignes, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_methode_defaut_compter_lignes() {
        let mut b = buffer("x\ny\nz\n");
        assert_eq!(b.compter_lignes().unwrap(), 3);
    }

    #[test]
    fn test_taille_estimee_surchargee() {
        let contenu = "hello world";
        let mut b = BufferLecteur::depuis_str("t", contenu);
        // BufferLecteur surcharge taille_estimee pour retourner len() sans lire
        assert_eq!(b.taille_estimee().unwrap(), contenu.len());
    }

    #[test]
    fn test_description_defaut() {
        let b = BufferLecteur::depuis_str("ma_source", "...");
        assert_eq!(b.description(), "Source[ma_source]");
    }

    #[test]
    fn test_analyser_source_statique() {
        let mut b = buffer("mot1 mot2 mot3\nmot4\n");
        let stats = analyser_source(&mut b).unwrap();
        assert_eq!(stats.nb_mots, 4);
        assert_eq!(stats.nb_lignes, 2);
    }

    #[test]
    fn test_collection_dynamique_heterogene() {
        let mut sources: Vec<Box<dyn Readable>> = vec![
            Box::new(BufferLecteur::depuis_str("s1", "a b\nc d\n")),
            Box::new(BufferLecteur::depuis_str("s2", "e f g\n")),
        ];
        let resultats = analyser_toutes_les_sources(&mut sources).unwrap();
        assert_eq!(resultats.len(), 2);
        assert_eq!(resultats[0].nb_mots, 4);
        assert_eq!(resultats[1].nb_mots, 3);
    }

    #[test]
    fn test_fichier_lecteur() {
        std::fs::write("tp6_test_unit.txt", "bonjour\nmonde\n").unwrap();
        let mut f = FichierLecteur::nouveau("tp6_test_unit.txt").unwrap();
        let contenu = f.lire_tout().unwrap();
        assert!(contenu.contains("bonjour"));
        assert!(contenu.contains("monde"));
    }

    #[test]
    fn test_fichier_inexistant() {
        let res = FichierLecteur::nouveau("fichier_qui_nexiste_pas_xyz.txt");
        assert!(res.is_err());
    }

    #[test]
    fn test_buffer_depuis_bytes() {
        let mut b = BufferLecteur::depuis_bytes("raw", b"hello rust".to_vec());
        assert_eq!(b.lire_tout().unwrap(), "hello rust");
    }
}
