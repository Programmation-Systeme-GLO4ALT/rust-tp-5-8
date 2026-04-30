use crate::error::ReadError;

/// Trait principal : toute source de données lisible doit l'implémenter.
pub trait Readable {
    /// Nom descriptif de la source (pour les logs, l'affichage).
    fn source_name(&self) -> &str;

    /// Lit l'intégralité du contenu et le retourne en `String`.
    fn read_content(&mut self) -> Result<String, ReadError>;

    /// Indique si la source a déjà été consommée / est épuisée.
    fn is_exhausted(&self) -> bool;
}
