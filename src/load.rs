use std::path::Path;
use std::fs::File;
use std::io::{self, Read};

/// Returns a program, loaded from the file at the given path
pub fn load_program<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    Ok(buf)
}
