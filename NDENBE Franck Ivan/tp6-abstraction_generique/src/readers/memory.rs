use crate::readable::Readable;

pub struct MemoryBuffer {
    pub data: String,
}

impl Readable for MemoryBuffer {
    fn read(&mut self) -> String {
        self.data.clone()
    }
}