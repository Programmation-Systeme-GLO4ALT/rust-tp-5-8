use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Erreur I/O : {0}")]
    Io(#[from] std::io::Error),

    #[error("Source vide : aucune donnée disponible")]
    EmptySource,

    #[error("Encodage UTF-8 invalide : {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}
