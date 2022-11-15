use std::{io, fs};

// TODO: This needs a real file picker eventually

pub fn read_file(path: &str) -> Result<String, io::Error> {
    fs::read_to_string(path)
}
