use crate::error::ReadError;
use crate::readable::Readable;

/// Source en mémoire : wraps un `String`.
pub struct MemorySource {
    name: String,
    data: Option<String>,
}

impl MemorySource {
    pub fn new(name: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data: Some(data.into()),
        }
    }
}

impl Readable for MemorySource {
    fn source_name(&self) -> &str {
        &self.name
    }

    fn read_content(&mut self) -> Result<String, ReadError> {
        self.data.take().ok_or(ReadError::EmptySource)
    }

    fn is_exhausted(&self) -> bool {
        self.data.is_none()
    }
}
