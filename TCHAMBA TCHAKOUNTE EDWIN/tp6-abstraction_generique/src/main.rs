// TP6 — Abstraction I/O via traits
// Démonstration : polymorphisme statique (génériques) et dynamique (Box<dyn>)

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

// =============================================================================
// PARTIE 1 — Définition des traits (interfaces abstraites)
// =============================================================================
//
// En C++ ce serait une classe abstraite avec méthodes virtuelles.
// En Rust, on définit un Trait qui décrit le comportement attendu,
// sans imposer la représentation interne.

/// Source de données : tout ce dont on peut lire des lignes.
pub trait Source {
    /// Renvoie la prochaine ligne, ou `Ok(None)` si la source est épuisée.
    fn lire_ligne(&mut self) -> io::Result<Option<String>>;

    /// Nom human-readable, utile pour les logs.
    /// Implémentation par défaut, qu'on peut surcharger.
    fn nom(&self) -> &str {
        "source anonyme"
    }
}

/// Destination de données : tout ce sur quoi on peut écrire des lignes.
pub trait Destination {
    fn ecrire_ligne(&mut self, ligne: &str) -> io::Result<()>;

    fn nom(&self) -> &str {
        "destination anonyme"
    }
}

// =============================================================================
// PARTIE 2 — Implémentations concrètes
// =============================================================================

// --- Source : fichier ---------------------------------------------------------
pub struct SourceFichier {
    chemin: String,
    lecteur: std::io::Lines<BufReader<File>>,
}

impl SourceFichier {
    pub fn ouvrir<P: AsRef<Path>>(chemin: P) -> io::Result<Self> {
        let p = chemin.as_ref();
        let fichier = File::open(p)?;
        Ok(Self {
            chemin: p.display().to_string(),
            lecteur: BufReader::new(fichier).lines(),
        })
    }
}

impl Source for SourceFichier {
    fn lire_ligne(&mut self) -> io::Result<Option<String>> {
        // `Lines` est un Iterator<Item = io::Result<String>>.
        // On transpose : Option<Result<T,E>> → Result<Option<T>,E>.
        match self.lecteur.next() {
            Some(Ok(s)) => Ok(Some(s)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }

    fn nom(&self) -> &str {
        &self.chemin
    }
}

// --- Source : chaîne en mémoire -----------------------------------------------
pub struct SourceMemoire {
    lignes: std::vec::IntoIter<String>,
}

impl SourceMemoire {
    pub fn nouveau(contenu: &str) -> Self {
        let lignes: Vec<String> = contenu.lines().map(String::from).collect();
        Self {
            lignes: lignes.into_iter(),
        }
    }
}

impl Source for SourceMemoire {
    fn lire_ligne(&mut self) -> io::Result<Option<String>> {
        Ok(self.lignes.next())
    }

    fn nom(&self) -> &str {
        "mémoire"
    }
}

// --- Destination : fichier ----------------------------------------------------
pub struct DestinationFichier {
    chemin: String,
    fichier: File,
}

impl DestinationFichier {
    pub fn creer<P: AsRef<Path>>(chemin: P) -> io::Result<Self> {
        let p = chemin.as_ref();
        let fichier = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(p)?;
        Ok(Self {
            chemin: p.display().to_string(),
            fichier,
        })
    }
}

impl Destination for DestinationFichier {
    fn ecrire_ligne(&mut self, ligne: &str) -> io::Result<()> {
        writeln!(self.fichier, "{}", ligne)
    }

    fn nom(&self) -> &str {
        &self.chemin
    }
}

// --- Destination : tampon mémoire ---------------------------------------------
pub struct DestinationBuffer {
    pub tampon: Vec<String>,
}

impl DestinationBuffer {
    pub fn nouveau() -> Self {
        Self { tampon: Vec::new() }
    }
}

impl Destination for DestinationBuffer {
    fn ecrire_ligne(&mut self, ligne: &str) -> io::Result<()> {
        self.tampon.push(ligne.to_string());
        Ok(())
    }

    fn nom(&self) -> &str {
        "buffer mémoire"
    }
}

// --- Destination : stdout -----------------------------------------------------
pub struct DestinationStdout;

impl Destination for DestinationStdout {
    fn ecrire_ligne(&mut self, ligne: &str) -> io::Result<()> {
        println!("{}", ligne);
        Ok(())
    }

    fn nom(&self) -> &str {
        "stdout"
    }
}

// =============================================================================
// PARTIE 3 — Polymorphisme statique (génériques + bornes de traits)
// =============================================================================
//
// Le compilateur génère une version spécialisée de cette fonction pour chaque
// combinaison concrète (S, D) utilisée dans le code (monomorphisation).
// → Aucun coût à l'exécution, mais le binaire grossit.
// Équivalent C++ : un template avec deux paramètres de type.

pub fn copier_statique<S: Source, D: Destination>(
    source: &mut S,
    dest: &mut D,
) -> io::Result<usize> {
    let mut compteur = 0;
    while let Some(ligne) = source.lire_ligne()? {
        dest.ecrire_ligne(&ligne)?;
        compteur += 1;
    }
    Ok(compteur)
}

// =============================================================================
// PARTIE 4 — Polymorphisme dynamique (objets-traits / dyn Trait)
// =============================================================================
//
// Ici, le type concret derrière `&mut dyn Source` est résolu à l'exécution
// via une vtable. Une seule version de la fonction est compilée.
// Équivalent C++ : passer un `AbstractSource*` (méthodes virtuelles).
// Coût : indirection à chaque appel, mais binaire plus compact et possibilité
// de stocker des types hétérogènes dans une même collection.

pub fn copier_dynamique(
    source: &mut dyn Source,
    dest: &mut dyn Destination,
) -> io::Result<usize> {
    let mut compteur = 0;
    while let Some(ligne) = source.lire_ligne()? {
        dest.ecrire_ligne(&ligne)?;
        compteur += 1;
    }
    Ok(compteur)
}

// =============================================================================
// PARTIE 5 — Collection hétérogène de sources (Box<dyn Trait>)
// =============================================================================
//
// On ne peut pas faire `Vec<dyn Source>` parce que la taille n'est pas connue
// à la compilation. On boxe chaque élément : `Box<dyn Source>` est un pointeur
// (taille fixe) vers un objet-trait (taille variable, sur le tas).

pub fn fusionner(
    sources: &mut [Box<dyn Source>],
    dest: &mut dyn Destination,
) -> io::Result<usize> {
    let mut total = 0;
    for src in sources.iter_mut() {
        total += copier_dynamique(src.as_mut(), dest)?;
    }
    Ok(total)
}

// =============================================================================
// PARTIE 6 — Programme principal : démonstration
// =============================================================================
fn main() -> io::Result<()> {
    println!("=== TP6 — Abstraction I/O via traits ===\n");

    // --- Démo 1 : polymorphisme statique (générique) -------------------------
    println!("[1] Statique : SourceMemoire → DestinationStdout");
    let mut src1 = SourceMemoire::nouveau("alpha\nbeta\ngamma");
    let mut dst1 = DestinationStdout;
    let n = copier_statique(&mut src1, &mut dst1)?;
    println!("    → {} lignes copiées\n", n);

    // --- Démo 2 : polymorphisme dynamique (dyn Trait) ------------------------
    println!("[2] Dynamique : SourceMemoire → DestinationBuffer");
    let mut src2 = SourceMemoire::nouveau("delta\nepsilon");
    let mut dst2 = DestinationBuffer::nouveau();
    let n = copier_dynamique(&mut src2, &mut dst2)?;
    println!("    → {} lignes copiées dans '{}'", n, dst2.nom());
    for (i, l) in dst2.tampon.iter().enumerate() {
        println!("      [{}] {}", i, l);
    }
    println!();

    // --- Démo 3 : fichier réel (TP6 test.txt) --------------------------------
    let chemin_in = "test.txt";
    if !Path::new(chemin_in).exists() {
        let mut f = File::create(chemin_in)?;
        writeln!(f, "première ligne du fichier")?;
        writeln!(f, "deuxième ligne du fichier")?;
        writeln!(f, "troisième ligne du fichier")?;
    }

    println!("[3] Fichier → Buffer (statique)");
    let mut src3 = SourceFichier::ouvrir(chemin_in)?;
    let mut dst3 = DestinationBuffer::nouveau();
    let n = copier_statique(&mut src3, &mut dst3)?;
    println!("    → {} lignes copiées depuis '{}'\n", n, chemin_in);

    // --- Démo 4 : collection hétérogène (Vec<Box<dyn Source>>) ---------------
    println!("[4] Fusion de sources hétérogènes → DestinationFichier");
    let mut sources: Vec<Box<dyn Source>> = vec![
        Box::new(SourceMemoire::nouveau("ligne_de_memoire_1\nligne_de_memoire_2")),
        Box::new(SourceFichier::ouvrir(chemin_in)?),
        Box::new(SourceMemoire::nouveau("ligne_de_memoire_3")),
    ];
    let mut dst4 = DestinationFichier::creer("sortie_fusion.txt")?;
    let total = fusionner(&mut sources, &mut dst4)?;
    println!(
        "    → {} lignes fusionnées dans '{}'",
        total,
        dst4.nom()
    );

    println!("\n✓ Toutes les démos ont réussi.");
    Ok(())
}

// =============================================================================
// PARTIE 7 — Tests unitaires
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statique_memoire_vers_buffer() {
        let mut src = SourceMemoire::nouveau("a\nb\nc");
        let mut dst = DestinationBuffer::nouveau();
        let n = copier_statique(&mut src, &mut dst).unwrap();
        assert_eq!(n, 3);
        assert_eq!(dst.tampon, vec!["a", "b", "c"]);
    }

    #[test]
    fn dynamique_memoire_vers_buffer() {
        let mut src = SourceMemoire::nouveau("x\ny");
        let mut dst = DestinationBuffer::nouveau();
        let n = copier_dynamique(&mut src, &mut dst).unwrap();
        assert_eq!(n, 2);
        assert_eq!(dst.tampon, vec!["x", "y"]);
    }

    #[test]
    fn fusion_sources_heterogenes() {
        let mut sources: Vec<Box<dyn Source>> = vec![
            Box::new(SourceMemoire::nouveau("1\n2")),
            Box::new(SourceMemoire::nouveau("3")),
        ];
        let mut dst = DestinationBuffer::nouveau();
        let total = fusionner(&mut sources, &mut dst).unwrap();
        assert_eq!(total, 3);
        assert_eq!(dst.tampon, vec!["1", "2", "3"]);
    }

    #[test]
    fn source_vide() {
        let mut src = SourceMemoire::nouveau("");
        let mut dst = DestinationBuffer::nouveau();
        let n = copier_statique(&mut src, &mut dst).unwrap();
        assert_eq!(n, 0);
        assert!(dst.tampon.is_empty());
    }

    #[test]
    fn nom_par_defaut_et_surcharge() {
        let src = SourceMemoire::nouveau("x");
        assert_eq!(src.nom(), "mémoire");
        let dst = DestinationStdout;
        assert_eq!(dst.nom(), "stdout");
    }
}
