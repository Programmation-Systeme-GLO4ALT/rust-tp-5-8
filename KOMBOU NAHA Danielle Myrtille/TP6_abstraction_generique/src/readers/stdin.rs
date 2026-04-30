use crate::readable::Readable;
use std::io::{self, Read};

pub struct StdinReader;

impl Readable for StdinReader {
    fn read(&mut self) -> String {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();
        input
    }
}