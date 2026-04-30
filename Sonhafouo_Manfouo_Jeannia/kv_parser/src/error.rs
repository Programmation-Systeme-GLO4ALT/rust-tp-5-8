use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Erreur d'entrée/sortie : {0}")]
    IoError(#[from] std::io::Error),

    #[error("Ligne {line_number} vide ou composée uniquement d'espaces")]
    EmptyLine { line_number: usize },

    #[error("Séparateur '=' manquant à la ligne {line_number} : {content:?}")]
    MissingSeparator { line_number: usize, content: String },

    #[error("Clé vide à la ligne {line_number}")]
    EmptyKey { line_number: usize },
}
