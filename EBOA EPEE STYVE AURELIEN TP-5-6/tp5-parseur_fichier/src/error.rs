use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Erreur de lecture du fichier")]
    Io(#[from] std::io::Error),

    #[error("Ligne invalide : {0}")]
    InvalidLine(String),
}